use chrono::NaiveDate;
use reqwest::{Client, Method};
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContactCategory {
    Personal,
    Service,
}

impl std::fmt::Display for ContactCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Personal => write!(f, "Personal"),
            Self::Service => write!(f, "Service"),
        }
    }
}

/// A contact fetched from CardDAV (enriched with local-only fields).
#[derive(Debug, Clone)]
pub struct Contact {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub signal: Option<String>,
    pub preferred_method: Option<String>,
    pub category: ContactCategory,
    pub last_contacted: Option<NaiveDate>,
    /// Server href for this vCard resource (used for DELETE).
    pub sync_href: Option<String>,
}

impl Contact {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            email: None,
            phone: None,
            website: None,
            signal: None,
            preferred_method: None,
            category: ContactCategory::Personal,
            last_contacted: None,
            sync_href: None,
        }
    }
}

pub struct CardDavClient {
    base_url: String,
    username: String,
    password: String,
    http: Client,
}

impl CardDavClient {
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

    /// Fetch all contacts from the CardDAV server.
    pub async fn fetch_contacts(&self) -> Result<Vec<Contact>, String> {
        // Discover the addressbook
        let principal = self.find_principal().await?;
        let home_set = self.find_addressbook_home_set(&principal).await?;
        let addressbooks = self.list_addressbooks(&home_set).await?;

        let mut contacts = Vec::new();
        for ab_href in &addressbooks {
            let mut ab_contacts = self.fetch_addressbook_contacts(ab_href).await?;
            contacts.append(&mut ab_contacts);
        }

        // Deduplicate by name
        contacts.sort_by(|a, b| a.name.cmp(&b.name));
        contacts.dedup_by(|a, b| a.name == b.name);

        Ok(contacts)
    }

    async fn find_principal(&self) -> Result<String, String> {
        let body = r#"<?xml version="1.0" encoding="UTF-8"?>
<d:propfind xmlns:d="DAV:">
  <d:prop>
    <d:current-user-principal/>
  </d:prop>
</d:propfind>"#;

        // Try .well-known/carddav first (RFC 6764), then root
        let urls_to_try = [
            format!("{}/.well-known/carddav", self.base_url),
            format!("{}/", self.base_url),
        ];

        for url in &urls_to_try {
            log::info!("CardDAV PROPFIND principal at: {}", url);

            let resp = match self
                .http
                .request(Method::from_bytes(b"PROPFIND").unwrap(), url)
                .basic_auth(&self.username, Some(&self.password))
                .header("Content-Type", "application/xml; charset=utf-8")
                .header("Depth", "0")
                .body(body.to_string())
                .send()
                .await
            {
                Ok(resp) => resp,
                Err(e) => {
                    log::warn!("CardDAV PROPFIND at {} failed: {}", url, e);
                    continue;
                }
            };

            let status = resp.status();
            let text = resp
                .text()
                .await
                .map_err(|e| format!("Failed to read CardDAV response: {}", e))?;

            log::info!("CardDAV PROPFIND at {} status: {}", url, status);
            log::debug!("CardDAV PROPFIND body: {}", text);

            if !status.is_success() && status.as_u16() != 207 {
                log::warn!("CardDAV PROPFIND at {} returned {}", url, status);
                continue;
            }

            if let Some(principal) = extract_href(&text, "current-user-principal") {
                return Ok(principal);
            }
        }

        // Last resort: construct principal URL from username (works for Fastmail)
        if self.username.contains('@') {
            let constructed = format!("/dav/principals/user/{}/", self.username);
            log::info!("Trying constructed CardDAV principal URL: {}", constructed);
            return Ok(constructed);
        }

        Err("Could not discover CardDAV principal — check your server URL".to_string())
    }

    async fn find_addressbook_home_set(&self, principal: &str) -> Result<String, String> {
        let url = self.resolve_href(principal);
        let body = r#"<?xml version="1.0" encoding="UTF-8"?>
<d:propfind xmlns:d="DAV:" xmlns:card="urn:ietf:params:xml:ns:carddav">
  <d:prop>
    <card:addressbook-home-set/>
  </d:prop>
</d:propfind>"#;

        let resp = self
            .http
            .request(Method::from_bytes(b"PROPFIND").unwrap(), &url)
            .basic_auth(&self.username, Some(&self.password))
            .header("Content-Type", "application/xml; charset=utf-8")
            .header("Depth", "0")
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| format!("CardDAV home-set PROPFIND failed: {}", e))?;

        let text = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read home-set response: {}", e))?;

        extract_href(&text, "addressbook-home-set")
            .ok_or_else(|| "Could not find addressbook-home-set".to_string())
    }

    async fn list_addressbooks(&self, home_set: &str) -> Result<Vec<String>, String> {
        let url = self.resolve_href(home_set);
        let body = r#"<?xml version="1.0" encoding="UTF-8"?>
<d:propfind xmlns:d="DAV:">
  <d:prop>
    <d:resourcetype/>
  </d:prop>
</d:propfind>"#;

        let resp = self
            .http
            .request(Method::from_bytes(b"PROPFIND").unwrap(), &url)
            .basic_auth(&self.username, Some(&self.password))
            .header("Content-Type", "application/xml; charset=utf-8")
            .header("Depth", "1")
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| format!("CardDAV addressbook listing failed: {}", e))?;

        let text = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read addressbook listing: {}", e))?;

        let doc = roxmltree::Document::parse(&text)
            .map_err(|e| format!("Failed to parse XML: {}", e))?;

        let mut addressbooks = Vec::new();
        for response in doc
            .descendants()
            .filter(|n| n.tag_name().name() == "response")
        {
            let href = response
                .children()
                .find(|n| n.tag_name().name() == "href")
                .and_then(|n| n.text())
                .unwrap_or("")
                .trim()
                .to_string();

            let is_addressbook = response.descendants().any(|n| {
                n.tag_name().name() == "resourcetype"
                    && n.children()
                        .any(|c| c.tag_name().name() == "addressbook")
            });

            if is_addressbook && !href.is_empty() {
                addressbooks.push(href);
            }
        }

        Ok(addressbooks)
    }

    async fn fetch_addressbook_contacts(&self, href: &str) -> Result<Vec<Contact>, String> {
        let url = self.resolve_href(href);
        let body = r#"<?xml version="1.0" encoding="UTF-8"?>
<card:addressbook-query xmlns:d="DAV:" xmlns:card="urn:ietf:params:xml:ns:carddav">
  <d:prop>
    <card:address-data/>
  </d:prop>
</card:addressbook-query>"#;

        let resp = self
            .http
            .request(Method::from_bytes(b"REPORT").unwrap(), &url)
            .basic_auth(&self.username, Some(&self.password))
            .header("Content-Type", "application/xml; charset=utf-8")
            .header("Depth", "1")
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| format!("CardDAV REPORT failed: {}", e))?;

        let text = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read CardDAV REPORT: {}", e))?;

        let doc = roxmltree::Document::parse(&text)
            .map_err(|e| format!("Failed to parse XML: {}", e))?;

        let mut contacts = Vec::new();
        for response in doc
            .descendants()
            .filter(|n| n.tag_name().name() == "response")
        {
            let resp_href = response
                .children()
                .find(|n| n.tag_name().name() == "href")
                .and_then(|n| n.text())
                .map(|s| s.trim().to_string());

            for prop in response
                .descendants()
                .filter(|n| n.tag_name().name() == "address-data")
            {
                if let Some(vcard_text) = prop.text() {
                    if let Some(mut contact) = parse_vcard(vcard_text) {
                        contact.sync_href = resp_href.clone();
                        contacts.push(contact);
                    }
                }
            }
        }

        Ok(contacts)
    }

    /// Delete a vCard resource from the server.
    pub async fn delete_contact(&self, href: &str) -> Result<(), String> {
        let url = self.resolve_href(href);
        log::info!("Deleting CardDAV contact: {}", url);
        let resp = self
            .http
            .request(Method::DELETE, &url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .map_err(|e| format!("CardDAV DELETE failed: {}", e))?;

        let status = resp.status();
        if status.is_success() || status.as_u16() == 204 || status.as_u16() == 404 {
            Ok(())
        } else {
            Err(format!("CardDAV DELETE returned {}", status))
        }
    }

    fn resolve_href(&self, href: &str) -> String {
        if href.starts_with("http://") || href.starts_with("https://") {
            href.to_string()
        } else {
            let origin = url_origin(&self.base_url);
            format!("{}{}", origin, href)
        }
    }
}

/// Parse a VCARD string to extract contact fields.
fn parse_vcard(vcard: &str) -> Option<Contact> {
    let mut name: Option<String> = None;
    let mut email: Option<String> = None;
    let mut phone: Option<String> = None;
    let mut website: Option<String> = None;
    let mut signal: Option<String> = None;
    let mut preferred_method: Option<String> = None;
    let mut category = ContactCategory::Personal;

    for line in vcard.lines() {
        let line = line.trim();
        if let Some(value) = strip_ical_prefix(line, "FN") {
            name = Some(value.to_string());
        }
        if let Some(value) = strip_ical_prefix(line, "EMAIL") {
            if email.is_none() {
                email = Some(value.to_string());
            }
        }
        if let Some(value) = strip_ical_prefix(line, "TEL") {
            if phone.is_none() {
                phone = Some(value.to_string());
            }
        }
        if let Some(value) = strip_ical_prefix(line, "URL") {
            if website.is_none() {
                website = Some(value.to_string());
            }
        }
        if let Some(value) = strip_ical_prefix(line, "X-SIGNAL") {
            signal = Some(value.to_string());
        }
        if let Some(value) = strip_ical_prefix(line, "X-PREFERRED-METHOD") {
            preferred_method = Some(value.to_string());
        }
        if let Some(value) = strip_ical_prefix(line, "CATEGORIES") {
            if value.eq_ignore_ascii_case("Service") {
                category = ContactCategory::Service;
            }
        }
    }

    let name = name.filter(|n| !n.is_empty())?;
    Some(Contact {
        id: Uuid::new_v4(),
        name,
        email,
        phone,
        website,
        signal,
        preferred_method,
        category,
        last_contacted: None, // vCard doesn't carry this
        sync_href: None, // set by fetch_addressbook_contacts after parsing
    })
}

/// Strip iCalendar/vCard property prefix, handling parameters.
/// E.g., "EMAIL;TYPE=WORK:alice@example.com" → "alice@example.com"
fn strip_ical_prefix<'a>(line: &'a str, property: &str) -> Option<&'a str> {
    // Match "PROPERTY:" or "PROPERTY;"
    if line.starts_with(property) {
        let rest = &line[property.len()..];
        if let Some(stripped) = rest.strip_prefix(':') {
            return Some(stripped);
        }
        if rest.starts_with(';') {
            // Has parameters — find the colon
            if let Some(colon) = rest.find(':') {
                return Some(&rest[colon + 1..]);
            }
        }
    }
    None
}

// --- contacts.org persistence ---

/// Write contacts to an org file.
pub fn write_contacts_org(contacts: &[Contact]) -> String {
    let mut out = String::new();
    out.push_str("#+TITLE: Contacts\n\n");
    for contact in contacts {
        out.push_str(&format!("* {}\n", contact.name));
        out.push_str("  :PROPERTIES:\n");
        out.push_str(&format!("  :ID: {}\n", contact.id));
        if let Some(ref v) = contact.email {
            out.push_str(&format!("  :EMAIL: {}\n", v));
        }
        if let Some(ref v) = contact.phone {
            out.push_str(&format!("  :PHONE: {}\n", v));
        }
        if let Some(ref v) = contact.website {
            out.push_str(&format!("  :WEBSITE: {}\n", v));
        }
        if let Some(ref v) = contact.signal {
            out.push_str(&format!("  :SIGNAL: {}\n", v));
        }
        if let Some(ref v) = contact.preferred_method {
            out.push_str(&format!("  :PREFERRED_METHOD: {}\n", v));
        }
        if contact.category != ContactCategory::Personal {
            out.push_str(&format!("  :CATEGORY: {}\n", contact.category));
        }
        if let Some(d) = contact.last_contacted {
            out.push_str(&format!("  :LAST_CONTACTED: [{}]\n", d.format("%Y-%m-%d")));
        }
        if let Some(ref v) = contact.sync_href {
            out.push_str(&format!("  :SYNC_HREF: {}\n", v));
        }
        out.push_str("  :END:\n");
    }
    out
}

/// Parse contacts from an org file.
pub fn parse_contacts_org(input: &str) -> Vec<Contact> {
    let mut contacts = Vec::new();
    let mut current: Option<Contact> = None;
    let mut in_properties = false;

    for line in input.lines() {
        if let Some(rest) = line.strip_prefix("* ") {
            if let Some(c) = current.take() {
                contacts.push(c);
            }
            current = Some(Contact::new(rest.trim().to_string()));
            in_properties = false;
        } else if line.trim() == ":PROPERTIES:" {
            in_properties = true;
        } else if line.trim() == ":END:" {
            in_properties = false;
        } else if in_properties {
            if let Some(c) = current.as_mut() {
                let trimmed = line.trim();
                if let Some(v) = trimmed.strip_prefix(":ID:") {
                    if let Ok(id) = Uuid::parse_str(v.trim()) {
                        c.id = id;
                    }
                } else if let Some(v) = trimmed.strip_prefix(":EMAIL:") {
                    c.email = Some(v.trim().to_string());
                } else if let Some(v) = trimmed.strip_prefix(":PHONE:") {
                    c.phone = Some(v.trim().to_string());
                } else if let Some(v) = trimmed.strip_prefix(":WEBSITE:") {
                    c.website = Some(v.trim().to_string());
                } else if let Some(v) = trimmed.strip_prefix(":SIGNAL:") {
                    c.signal = Some(v.trim().to_string());
                } else if let Some(v) = trimmed.strip_prefix(":PREFERRED_METHOD:") {
                    c.preferred_method = Some(v.trim().to_string());
                } else if let Some(v) = trimmed.strip_prefix(":CATEGORY:") {
                    let val = v.trim();
                    if val.eq_ignore_ascii_case("Service") {
                        c.category = ContactCategory::Service;
                    }
                } else if let Some(v) = trimmed.strip_prefix(":LAST_CONTACTED:") {
                    let val = v.trim().trim_start_matches('[').trim_end_matches(']');
                    if let Ok(d) = NaiveDate::parse_from_str(val, "%Y-%m-%d") {
                        c.last_contacted = Some(d);
                    }
                } else if let Some(v) = trimmed.strip_prefix(":SYNC_HREF:") {
                    c.sync_href = Some(v.trim().to_string());
                }
            }
        }
    }

    if let Some(c) = current {
        contacts.push(c);
    }

    contacts
}

/// Load contacts from the contacts.org file.
pub fn load_contacts(path: &Path) -> Vec<Contact> {
    match std::fs::read_to_string(path) {
        Ok(content) => parse_contacts_org(&content),
        Err(_) => Vec::new(),
    }
}

/// Save contacts to the contacts.org file.
pub fn save_contacts(path: &Path, contacts: &[Contact]) -> Result<(), String> {
    let content = write_contacts_org(contacts);
    std::fs::write(path, content).map_err(|e| format!("Failed to save contacts: {}", e))
}

/// Merge remote (CardDAV) contacts into local contacts list.
/// Matches by name. Preserves local-only fields like `last_contacted`.
pub fn merge_contacts(local: &mut Vec<Contact>, remote: Vec<Contact>) {
    for rc in remote {
        if let Some(lc) = local.iter_mut().find(|c| c.name == rc.name) {
            // Update vCard-sourced fields, preserve local-only fields
            lc.email = rc.email.or(lc.email.clone());
            lc.phone = rc.phone.or(lc.phone.clone());
            lc.website = rc.website.or(lc.website.clone());
            lc.signal = rc.signal.or(lc.signal.clone());
            lc.preferred_method = rc.preferred_method.or(lc.preferred_method.clone());
            if rc.category != ContactCategory::Personal {
                lc.category = rc.category;
            }
            // preserve last_contacted (local only)
            lc.sync_href = rc.sync_href.or(lc.sync_href.clone());
        } else {
            local.push(rc);
        }
    }
    local.sort_by(|a, b| a.name.cmp(&b.name));
}

fn extract_href(xml: &str, property_local_name: &str) -> Option<String> {
    let doc = roxmltree::Document::parse(xml).ok()?;
    for node in doc.descendants() {
        if node.tag_name().name() == property_local_name {
            for child in node.children() {
                if child.tag_name().name() == "href" {
                    return child.text().map(|s| s.trim().to_string());
                }
            }
        }
    }
    None
}

fn url_origin(url: &str) -> String {
    if let Some(scheme_end) = url.find("://") {
        let rest = &url[scheme_end + 3..];
        if let Some(slash) = rest.find('/') {
            return url[..scheme_end + 3 + slash].to_string();
        }
    }
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_vcard_basic() {
        let vcard = "BEGIN:VCARD\nVERSION:3.0\nFN:John Doe\nEMAIL:john@example.com\nTEL:+1-555-0123\nURL:https://johndoe.com\nEND:VCARD";
        let contact = parse_vcard(vcard).unwrap();
        assert_eq!(contact.name, "John Doe");
        assert_eq!(contact.email, Some("john@example.com".to_string()));
        assert_eq!(contact.phone, Some("+1-555-0123".to_string()));
        assert_eq!(contact.website, Some("https://johndoe.com".to_string()));
        assert_eq!(contact.category, ContactCategory::Personal);
    }

    #[test]
    fn parse_vcard_typed_email() {
        let vcard =
            "BEGIN:VCARD\nVERSION:3.0\nFN:Jane\nEMAIL;TYPE=WORK:jane@work.com\nEND:VCARD";
        let contact = parse_vcard(vcard).unwrap();
        assert_eq!(contact.email, Some("jane@work.com".to_string()));
    }

    #[test]
    fn parse_vcard_extended_fields() {
        let vcard = "BEGIN:VCARD\nVERSION:3.0\nFN:Alice\nX-SIGNAL:alice.42\nX-PREFERRED-METHOD:Signal\nCATEGORIES:Service\nEND:VCARD";
        let contact = parse_vcard(vcard).unwrap();
        assert_eq!(contact.signal, Some("alice.42".to_string()));
        assert_eq!(contact.preferred_method, Some("Signal".to_string()));
        assert_eq!(contact.category, ContactCategory::Service);
    }

    #[test]
    fn contacts_org_roundtrip() {
        let john_id = Uuid::new_v4();
        let jane_id = Uuid::new_v4();
        let contacts = vec![
            Contact {
                id: john_id,
                name: "John Doe".to_string(),
                email: Some("john@example.com".to_string()),
                phone: Some("+1-555-0123".to_string()),
                website: Some("https://johndoe.com".to_string()),
                signal: Some("john.42".to_string()),
                preferred_method: Some("Email".to_string()),
                category: ContactCategory::Personal,
                last_contacted: Some(NaiveDate::from_ymd_opt(2025, 2, 20).unwrap()),
                sync_href: Some("/dav/addr/john.vcf".to_string()),
            },
            Contact {
                id: jane_id,
                name: "Jane Smith".to_string(),
                email: None,
                phone: None,
                website: None,
                signal: None,
                preferred_method: None,
                category: ContactCategory::Service,
                last_contacted: None,
                sync_href: None,
            },
        ];
        let org = write_contacts_org(&contacts);
        let parsed = parse_contacts_org(&org);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].id, john_id);
        assert_eq!(parsed[0].name, "John Doe");
        assert_eq!(parsed[0].email, Some("john@example.com".to_string()));
        assert_eq!(parsed[0].phone, Some("+1-555-0123".to_string()));
        assert_eq!(parsed[0].website, Some("https://johndoe.com".to_string()));
        assert_eq!(parsed[0].signal, Some("john.42".to_string()));
        assert_eq!(parsed[0].preferred_method, Some("Email".to_string()));
        assert_eq!(parsed[0].category, ContactCategory::Personal);
        assert_eq!(
            parsed[0].last_contacted,
            Some(NaiveDate::from_ymd_opt(2025, 2, 20).unwrap())
        );
        assert_eq!(parsed[0].sync_href, Some("/dav/addr/john.vcf".to_string()));
        assert_eq!(parsed[1].id, jane_id);
        assert_eq!(parsed[1].name, "Jane Smith");
        assert_eq!(parsed[1].email, None);
        assert_eq!(parsed[1].category, ContactCategory::Service);
        assert_eq!(parsed[1].last_contacted, None);
        assert_eq!(parsed[1].sync_href, None);
    }

    #[test]
    fn merge_contacts_adds_new_and_updates_existing() {
        let mut local = vec![Contact {
            id: Uuid::new_v4(),
            name: "Alice".to_string(),
            email: Some("alice@local.com".to_string()),
            phone: None,
            website: None,
            signal: None,
            preferred_method: None,
            category: ContactCategory::Personal,
            last_contacted: Some(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
            sync_href: None,
        }];
        let remote = vec![
            Contact::new("Alice".to_string()),
            Contact::new("Bob".to_string()),
        ];
        merge_contacts(&mut local, remote);
        assert_eq!(local.len(), 2);
        // Alice's local email preserved, last_contacted preserved
        let alice = local.iter().find(|c| c.name == "Alice").unwrap();
        assert_eq!(alice.email, Some("alice@local.com".to_string()));
        assert!(alice.last_contacted.is_some());
        // Bob added
        assert!(local.iter().any(|c| c.name == "Bob"));
    }
}
