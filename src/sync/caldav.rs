use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, Method, StatusCode};

/// Info about a remote calendar discovered via PROPFIND.
#[derive(Debug, Clone)]
pub struct CalendarInfo {
    pub href: String,
    pub display_name: String,
    pub supports_vtodo: bool,
    pub supports_vevent: bool,
}

/// A remote VTODO fetched from the server.
#[derive(Debug, Clone)]
pub struct RemoteVtodo {
    pub href: String,
    pub etag: String,
    pub uid: Option<String>,
    pub ical_body: String,
}

/// Represents a change returned by a sync-collection REPORT.
#[derive(Debug, Clone)]
pub enum SyncChange {
    /// Updated or new resource.
    Changed(RemoteVtodo),
    /// Deleted resource (only href is known).
    Deleted(String),
}

/// Condition for PUT requests.
pub enum PutCondition<'a> {
    /// If-None-Match: * — create only, fail if resource exists.
    CreateOnly,
    /// If-Match: <etag> — update only if etag matches.
    UpdateEtag(&'a str),
    /// No conditional header — unconditional overwrite.
    Unconditional,
}

#[derive(Clone)]
pub struct CalDavClient {
    base_url: String,
    username: String,
    password: String,
    http: Client,
}

impl CalDavClient {
    pub fn new(base_url: &str, username: &str, password: &str) -> Result<Self, String> {
        let http = Client::builder()
            .build()
            .map_err(|e| format!("Failed to build HTTP client: {}", e))?;
        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            username: username.to_string(),
            password: password.to_string(),
            http,
        })
    }

    /// Discover calendars by PROPFINDing the calendar-home-set.
    pub async fn discover_calendars(&self) -> Result<Vec<CalendarInfo>, String> {
        // Step 1: Find the principal URL
        let principal_url = self.find_principal().await?;
        log::info!("Found principal: {}", principal_url);
        // Step 2: Find the calendar-home-set
        let home_set = self.find_calendar_home_set(&principal_url).await?;
        log::info!("Found calendar-home-set: {}", home_set);
        // Step 3: List calendars in the home set
        self.list_calendars(&home_set).await
    }

    /// List all VTODOs in a calendar using a REPORT.
    pub async fn list_vtodos(&self, calendar_href: &str) -> Result<Vec<RemoteVtodo>, String> {
        let url = self.resolve_href(calendar_href);
        let body = r#"<?xml version="1.0" encoding="UTF-8"?>
<c:calendar-query xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:prop>
    <d:getetag/>
    <c:calendar-data/>
  </d:prop>
  <c:filter>
    <c:comp-filter name="VCALENDAR">
      <c:comp-filter name="VTODO"/>
    </c:comp-filter>
  </c:filter>
</c:calendar-query>"#;

        let resp = self
            .request(Method::from_bytes(b"REPORT").unwrap(), &url)
            .header(CONTENT_TYPE, "application/xml; charset=utf-8")
            .header("Depth", "1")
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| format!("REPORT request failed: {}", e))?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read REPORT response: {}", e))?;

        if !status.is_success() && status != StatusCode::MULTI_STATUS {
            return Err(format!("REPORT failed with status {}: {}", status, text));
        }

        parse_multistatus_vtodos(&text)
    }

    /// List all VEVENTs in a calendar using a REPORT.
    pub async fn list_vevents(&self, calendar_href: &str) -> Result<Vec<RemoteVtodo>, String> {
        let url = self.resolve_href(calendar_href);
        let body = r#"<?xml version="1.0" encoding="UTF-8"?>
<c:calendar-query xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:prop>
    <d:getetag/>
    <c:calendar-data/>
  </d:prop>
  <c:filter>
    <c:comp-filter name="VCALENDAR">
      <c:comp-filter name="VEVENT"/>
    </c:comp-filter>
  </c:filter>
</c:calendar-query>"#;

        let resp = self
            .request(Method::from_bytes(b"REPORT").unwrap(), &url)
            .header(CONTENT_TYPE, "application/xml; charset=utf-8")
            .header("Depth", "1")
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| format!("REPORT request failed: {}", e))?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read REPORT response: {}", e))?;

        if !status.is_success() && status != StatusCode::MULTI_STATUS {
            return Err(format!("REPORT failed with status {}: {}", status, text));
        }

        parse_multistatus_vtodos(&text)
    }

    /// Get a single VTODO by href. Returns (etag, ical_body).
    pub async fn get_vtodo(&self, href: &str) -> Result<(String, String), String> {
        let url = self.resolve_href(href);
        let resp = self
            .request(Method::GET, &url)
            .send()
            .await
            .map_err(|e| format!("GET failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("GET {} failed: {}", href, resp.status()));
        }

        let etag = resp
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        let body = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read GET body: {}", e))?;

        Ok((etag, body))
    }

    /// PUT a VTODO to the server. Returns the new etag.
    ///
    /// `condition`:
    /// - `PutCondition::CreateOnly` — If-None-Match: * (fail if exists)
    /// - `PutCondition::UpdateEtag(etag)` — If-Match: etag
    /// - `PutCondition::Unconditional` — no conditional header (overwrite)
    pub async fn put_vtodo(
        &self,
        href: &str,
        condition: PutCondition<'_>,
        ical: &str,
    ) -> Result<String, String> {
        let url = self.resolve_href(href);
        let mut req = self
            .request(Method::PUT, &url)
            .header(CONTENT_TYPE, "text/calendar; charset=utf-8")
            .body(ical.to_string());

        match condition {
            PutCondition::CreateOnly => {
                req = req.header("If-None-Match", "*");
            }
            PutCondition::UpdateEtag(etag) => {
                req = req.header("If-Match", etag);
            }
            PutCondition::Unconditional => {}
        }

        let resp = req
            .send()
            .await
            .map_err(|e| format!("PUT failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("PUT {} failed ({}): {}", href, status, body));
        }

        let new_etag = resp
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        Ok(new_etag)
    }

    /// DELETE a VTODO from the server.
    pub async fn delete_vtodo(&self, href: &str, etag: &str) -> Result<(), String> {
        let url = self.resolve_href(href);
        let mut req = self.request(Method::DELETE, &url);
        if !etag.is_empty() {
            req = req.header("If-Match", etag);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| format!("DELETE failed: {}", e))?;

        if !resp.status().is_success() && resp.status() != StatusCode::NOT_FOUND {
            return Err(format!("DELETE {} failed: {}", href, resp.status()));
        }

        Ok(())
    }

    /// Perform a sync-collection REPORT (RFC 6578) to get changes since last sync.
    /// Returns the changes and a new sync-token.
    pub async fn sync_collection(
        &self,
        calendar_href: &str,
        sync_token: Option<&str>,
    ) -> Result<(Vec<SyncChange>, Option<String>), String> {
        let url = self.resolve_href(calendar_href);
        let token_element = match sync_token {
            Some(token) => format!("<d:sync-token>{}</d:sync-token>", token),
            None => "<d:sync-token/>".to_string(),
        };

        let body = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<d:sync-collection xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  {}
  <d:sync-level>1</d:sync-level>
  <d:prop>
    <d:getetag/>
    <c:calendar-data/>
  </d:prop>
</d:sync-collection>"#,
            token_element
        );

        let resp = self
            .request(Method::from_bytes(b"REPORT").unwrap(), &url)
            .header(CONTENT_TYPE, "application/xml; charset=utf-8")
            .body(body)
            .send()
            .await
            .map_err(|e| format!("sync-collection REPORT failed: {}", e))?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read sync-collection response: {}", e))?;

        // 403/409 with valid-sync-token error means token expired — caller should retry without token
        if status == StatusCode::FORBIDDEN
            || status == StatusCode::CONFLICT
            || status == StatusCode::PRECONDITION_FAILED
        {
            if text.contains("valid-sync-token") {
                return Err("sync-token-expired".to_string());
            }
        }

        if !status.is_success() && status != StatusCode::MULTI_STATUS {
            return Err(format!(
                "sync-collection failed ({}): {}",
                status, text
            ));
        }

        parse_sync_response(&text)
    }

    // --- Private helpers ---

    fn request(&self, method: Method, url: &str) -> reqwest::RequestBuilder {
        self.http
            .request(method, url)
            .basic_auth(&self.username, Some(&self.password))
    }

    fn resolve_href(&self, href: &str) -> String {
        if href.starts_with("http://") || href.starts_with("https://") {
            href.to_string()
        } else {
            // href is a path — combine with base URL's origin
            let origin = url_origin(&self.base_url);
            format!("{}{}", origin, href)
        }
    }

    async fn find_principal(&self) -> Result<String, String> {
        let body = r#"<?xml version="1.0" encoding="UTF-8"?>
<d:propfind xmlns:d="DAV:">
  <d:prop>
    <d:current-user-principal/>
  </d:prop>
</d:propfind>"#;

        // Try .well-known/caldav first (RFC 6764), then root, then constructed URL
        let urls_to_try = [
            format!("{}/.well-known/caldav", self.base_url),
            format!("{}/", self.base_url),
        ];

        for url in &urls_to_try {
            log::info!("PROPFIND principal at: {}", url);

            let resp = match self
                .request(
                    Method::from_bytes(b"PROPFIND").unwrap(),
                    url,
                )
                .header(CONTENT_TYPE, "application/xml; charset=utf-8")
                .header("Depth", "0")
                .body(body.to_string())
                .send()
                .await
            {
                Ok(resp) => resp,
                Err(e) => {
                    log::warn!("PROPFIND at {} failed: {}", url, e);
                    continue;
                }
            };

            let status = resp.status();
            let text = resp
                .text()
                .await
                .map_err(|e| format!("Failed to read principal response: {}", e))?;

            log::info!("Principal PROPFIND at {} status: {}", url, status);
            log::debug!("Principal PROPFIND body: {}", text);

            if !status.is_success() && status != StatusCode::MULTI_STATUS {
                log::warn!("Principal PROPFIND at {} returned {}", url, status);
                continue;
            }

            if let Some(principal) = extract_href_from_xml(&text, "current-user-principal") {
                return Ok(principal);
            }
        }

        // Last resort: construct principal URL from username (works for Fastmail)
        if self.username.contains('@') {
            let constructed = format!("/dav/principals/user/{}/", self.username);
            log::info!("Trying constructed principal URL: {}", constructed);
            return Ok(constructed);
        }

        Err("Could not discover CalDAV principal — check your server URL".to_string())
    }

    async fn find_calendar_home_set(&self, principal_url: &str) -> Result<String, String> {
        let url = self.resolve_href(principal_url);
        log::info!("PROPFIND calendar-home-set at: {}", url);

        let body = r#"<?xml version="1.0" encoding="UTF-8"?>
<d:propfind xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:prop>
    <c:calendar-home-set/>
  </d:prop>
</d:propfind>"#;

        let resp = self
            .request(Method::from_bytes(b"PROPFIND").unwrap(), &url)
            .header(CONTENT_TYPE, "application/xml; charset=utf-8")
            .header("Depth", "0")
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| format!("PROPFIND for calendar-home-set at {} failed: {}", url, e))?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read home-set response: {}", e))?;

        log::info!("Calendar-home-set PROPFIND status: {}", status);

        extract_href_from_xml(&text, "calendar-home-set")
            .ok_or_else(|| format!("Could not find calendar-home-set in response: {}", text))
    }

    async fn list_calendars(&self, home_set_url: &str) -> Result<Vec<CalendarInfo>, String> {
        let url = self.resolve_href(home_set_url);
        log::info!("PROPFIND calendars at: {}", url);

        let body = r#"<?xml version="1.0" encoding="UTF-8"?>
<d:propfind xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:prop>
    <d:displayname/>
    <d:resourcetype/>
    <c:supported-calendar-component-set/>
  </d:prop>
</d:propfind>"#;

        let resp = self
            .request(Method::from_bytes(b"PROPFIND").unwrap(), &url)
            .header(CONTENT_TYPE, "application/xml; charset=utf-8")
            .header("Depth", "1")
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| format!("PROPFIND for calendars at {} failed: {}", url, e))?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read calendar listing: {}", e))?;

        log::info!("Calendar listing PROPFIND status: {}", status);

        let calendars = parse_calendar_listing(&text)?;
        log::info!("Discovered {} calendars", calendars.len());
        for cal in &calendars {
            log::info!("  Calendar: {} (href={}, vtodo={}, vevent={})", cal.display_name, cal.href, cal.supports_vtodo, cal.supports_vevent);
        }
        Ok(calendars)
    }
}

// --- XML parsing helpers (using roxmltree) ---

fn extract_href_from_xml(xml: &str, property_local_name: &str) -> Option<String> {
    let doc = roxmltree::Document::parse(xml).ok()?;
    for node in doc.descendants() {
        if node.tag_name().name() == property_local_name {
            // Look for a child <href> element
            for child in node.children() {
                if child.tag_name().name() == "href" {
                    return child.text().map(|s| s.trim().to_string());
                }
            }
        }
    }
    None
}

fn parse_calendar_listing(xml: &str) -> Result<Vec<CalendarInfo>, String> {
    let doc =
        roxmltree::Document::parse(xml).map_err(|e| format!("Failed to parse XML: {}", e))?;

    let mut calendars = Vec::new();

    for response_node in doc.descendants().filter(|n| n.tag_name().name() == "response") {
        let href = response_node
            .children()
            .find(|n| n.tag_name().name() == "href")
            .and_then(|n| n.text())
            .unwrap_or("")
            .trim()
            .to_string();

        if href.is_empty() {
            continue;
        }

        let mut display_name = String::new();
        let mut is_calendar = false;
        let mut supports_vtodo = false;
        let mut supports_vevent = false;

        for prop_node in response_node
            .descendants()
            .filter(|n| n.tag_name().name() == "prop")
        {
            for child in prop_node.children() {
                match child.tag_name().name() {
                    "displayname" => {
                        display_name = child.text().unwrap_or("").trim().to_string();
                    }
                    "resourcetype" => {
                        is_calendar = child
                            .children()
                            .any(|n| n.tag_name().name() == "calendar");
                    }
                    "supported-calendar-component-set" => {
                        for comp in child.children().filter(|n| n.tag_name().name() == "comp") {
                            if let Some(name) = comp.attribute("name") {
                                if name.eq_ignore_ascii_case("VTODO") {
                                    supports_vtodo = true;
                                }
                                if name.eq_ignore_ascii_case("VEVENT") {
                                    supports_vevent = true;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if is_calendar {
            calendars.push(CalendarInfo {
                href,
                display_name,
                supports_vtodo,
                supports_vevent,
            });
        }
    }

    Ok(calendars)
}

fn parse_multistatus_vtodos(xml: &str) -> Result<Vec<RemoteVtodo>, String> {
    let doc =
        roxmltree::Document::parse(xml).map_err(|e| format!("Failed to parse XML: {}", e))?;

    let mut vtodos = Vec::new();

    for response_node in doc.descendants().filter(|n| n.tag_name().name() == "response") {
        let href = response_node
            .children()
            .find(|n| n.tag_name().name() == "href")
            .and_then(|n| n.text())
            .unwrap_or("")
            .trim()
            .to_string();

        let mut etag = String::new();
        let mut ical_body = String::new();

        for prop_node in response_node
            .descendants()
            .filter(|n| n.tag_name().name() == "prop")
        {
            for child in prop_node.children() {
                match child.tag_name().name() {
                    "getetag" => {
                        etag = child
                            .text()
                            .unwrap_or("")
                            .trim()
                            .trim_matches('"')
                            .to_string();
                    }
                    "calendar-data" => {
                        ical_body = child.text().unwrap_or("").to_string();
                    }
                    _ => {}
                }
            }
        }

        if !href.is_empty() && !ical_body.is_empty() {
            let uid = extract_uid_from_ical(&ical_body);
            vtodos.push(RemoteVtodo {
                href,
                etag,
                uid,
                ical_body,
            });
        }
    }

    Ok(vtodos)
}

fn parse_sync_response(
    xml: &str,
) -> Result<(Vec<SyncChange>, Option<String>), String> {
    let doc =
        roxmltree::Document::parse(xml).map_err(|e| format!("Failed to parse XML: {}", e))?;

    let mut changes = Vec::new();

    // Extract new sync-token
    let new_token = doc
        .descendants()
        .find(|n| n.tag_name().name() == "sync-token")
        .and_then(|n| n.text())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    for response_node in doc.descendants().filter(|n| n.tag_name().name() == "response") {
        let href = response_node
            .children()
            .find(|n| n.tag_name().name() == "href")
            .and_then(|n| n.text())
            .unwrap_or("")
            .trim()
            .to_string();

        if href.is_empty() {
            continue;
        }

        // Check status — 404 means deleted
        let is_deleted = response_node.descendants().any(|n| {
            n.tag_name().name() == "status"
                && n.text()
                    .is_some_and(|t| t.contains("404"))
        });

        if is_deleted {
            changes.push(SyncChange::Deleted(href));
            continue;
        }

        let mut etag = String::new();
        let mut ical_body = String::new();

        for prop_node in response_node
            .descendants()
            .filter(|n| n.tag_name().name() == "prop")
        {
            for child in prop_node.children() {
                match child.tag_name().name() {
                    "getetag" => {
                        etag = child
                            .text()
                            .unwrap_or("")
                            .trim()
                            .trim_matches('"')
                            .to_string();
                    }
                    "calendar-data" => {
                        ical_body = child.text().unwrap_or("").to_string();
                    }
                    _ => {}
                }
            }
        }

        if !ical_body.is_empty() {
            let uid = extract_uid_from_ical(&ical_body);
            changes.push(SyncChange::Changed(RemoteVtodo {
                href,
                etag,
                uid,
                ical_body,
            }));
        }
    }

    Ok((changes, new_token))
}

/// Extract UID from raw iCalendar text (simple line scan).
fn extract_uid_from_ical(ical: &str) -> Option<String> {
    for line in ical.lines() {
        let trimmed = line.trim();
        if let Some(uid) = trimmed.strip_prefix("UID:") {
            return Some(uid.trim().to_string());
        }
    }
    None
}

/// Get the origin (scheme + host + port) from a URL string.
fn url_origin(url: &str) -> String {
    // Find the third slash (end of "https://host")
    if let Some(scheme_end) = url.find("://") {
        let rest = &url[scheme_end + 3..];
        if let Some(slash) = rest.find('/') {
            return url[..scheme_end + 3 + slash].to_string();
        }
    }
    url.to_string()
}

/// Generate a CalDAV href for a new VTODO.
pub fn vtodo_href(calendar_href: &str, task_uid: &uuid::Uuid) -> String {
    let base = calendar_href.trim_end_matches('/');
    format!("{}/{}.ics", base, task_uid)
}

/// Generate a CalDAV href for a new VEVENT.
pub fn vevent_href(calendar_href: &str, event_uid: &uuid::Uuid) -> String {
    let base = calendar_href.trim_end_matches('/');
    format!("{}/{}.ics", base, event_uid)
}
