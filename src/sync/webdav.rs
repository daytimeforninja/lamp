use reqwest::{Client, Method, StatusCode};
use std::path::Path;

use crate::core::note::Note;
use crate::org::{convert, writer::OrgWriter};

/// A file discovered on the remote WebDAV collection.
#[derive(Debug, Clone)]
pub struct RemoteFile {
    pub href: String,
    pub filename: String,
    pub etag: Option<String>,
}

/// Result of a notes sync operation.
#[derive(Debug, Clone)]
pub struct NoteSyncResult {
    pub pulled: Vec<Note>,
    pub pushed: usize,
    pub deleted_remote: usize,
    pub errors: Vec<String>,
}

/// Minimal WebDAV client for note sync.
pub struct WebDavClient {
    base_url: String,
    username: String,
    password: String,
    http: Client,
}

impl WebDavClient {
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

    /// Ensure the remote collection exists (MKCOL, ignore 405 "already exists").
    pub async fn ensure_collection(&self) -> Result<(), String> {
        let resp = self
            .http
            .request(Method::from_bytes(b"MKCOL").unwrap(), &self.base_url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .map_err(|e| format!("MKCOL failed: {}", e))?;

        match resp.status() {
            StatusCode::CREATED | StatusCode::OK => Ok(()),
            StatusCode::METHOD_NOT_ALLOWED => Ok(()), // already exists
            s => Err(format!("MKCOL returned {}", s)),
        }
    }

    /// List files in the collection via PROPFIND Depth:1.
    pub async fn list_files(&self) -> Result<Vec<RemoteFile>, String> {
        let body = r#"<?xml version="1.0" encoding="UTF-8"?>
<d:propfind xmlns:d="DAV:">
  <d:prop>
    <d:getetag/>
  </d:prop>
</d:propfind>"#;

        let resp = self
            .http
            .request(Method::from_bytes(b"PROPFIND").unwrap(), &self.base_url)
            .basic_auth(&self.username, Some(&self.password))
            .header("Depth", "1")
            .header("Content-Type", "application/xml")
            .body(body)
            .send()
            .await
            .map_err(|e| format!("PROPFIND failed: {}", e))?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read PROPFIND response: {}", e))?;

        if status != StatusCode::MULTI_STATUS && !status.is_success() {
            return Err(format!("PROPFIND returned {}: {}", status, text));
        }

        parse_propfind_response(&text, &self.base_url)
    }

    /// GET a file's content and etag.
    pub async fn get_file(&self, filename: &str) -> Result<(String, Option<String>), String> {
        let url = format!("{}/{}", self.base_url, filename);
        let resp = self
            .http
            .get(&url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .map_err(|e| format!("GET failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("GET {} returned {}", filename, resp.status()));
        }

        let etag = resp
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.trim_matches('"').to_string());

        let content = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        Ok((content, etag))
    }

    /// PUT a file, returning the new etag if provided.
    pub async fn put_file(&self, filename: &str, content: &str) -> Result<Option<String>, String> {
        let url = format!("{}/{}", self.base_url, filename);
        let resp = self
            .http
            .put(&url)
            .basic_auth(&self.username, Some(&self.password))
            .header("Content-Type", "text/org; charset=utf-8")
            .body(content.to_string())
            .send()
            .await
            .map_err(|e| format!("PUT failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("PUT {} returned {}", filename, resp.status()));
        }

        let etag = resp
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.trim_matches('"').to_string());

        Ok(etag)
    }

    /// DELETE a file from the collection.
    pub async fn delete_file(&self, filename: &str) -> Result<(), String> {
        let url = format!("{}/{}", self.base_url, filename);
        let resp = self
            .http
            .delete(&url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .map_err(|e| format!("DELETE failed: {}", e))?;

        match resp.status() {
            StatusCode::NO_CONTENT | StatusCode::OK | StatusCode::NOT_FOUND => Ok(()),
            s => Err(format!("DELETE {} returned {}", filename, s)),
        }
    }
}

/// Parse a PROPFIND multistatus response into RemoteFile entries.
fn parse_propfind_response(xml: &str, base_url: &str) -> Result<Vec<RemoteFile>, String> {
    let doc =
        roxmltree::Document::parse(xml).map_err(|e| format!("Failed to parse XML: {}", e))?;

    let base_path = url_path(base_url);
    let mut files = Vec::new();

    for response in doc.descendants().filter(|n| n.has_tag_name("response")) {
        let href = response
            .descendants()
            .find(|n| n.has_tag_name("href"))
            .and_then(|n| n.text())
            .unwrap_or("");

        // Skip the collection itself
        let href_clean = href.trim_end_matches('/');
        let base_clean = base_path.trim_end_matches('/');
        if href_clean == base_clean {
            continue;
        }

        // Only include .org files
        let filename = href
            .rsplit('/')
            .find(|s| !s.is_empty())
            .unwrap_or("")
            .to_string();
        if !filename.ends_with(".org") {
            continue;
        }

        let etag = response
            .descendants()
            .find(|n| n.has_tag_name("getetag"))
            .and_then(|n| n.text())
            .map(|s| s.trim_matches('"').to_string());

        files.push(RemoteFile {
            href: href.to_string(),
            filename,
            etag,
        });
    }

    Ok(files)
}

/// Extract the path component from a URL.
fn url_path(url: &str) -> String {
    // Strip scheme + authority to get path
    if let Some(rest) = url.strip_prefix("https://") {
        if let Some(idx) = rest.find('/') {
            return rest[idx..].to_string();
        }
    }
    if let Some(rest) = url.strip_prefix("http://") {
        if let Some(idx) = rest.find('/') {
            return rest[idx..].to_string();
        }
    }
    url.to_string()
}

/// Sync notes between local files and a WebDAV server.
///
/// Strategy: etag-based. Each note stores its last-known remote etag.
/// - Remote file with different etag → pull (remote wins)
/// - Local note with no etag → push (new note)
/// - Local note with etag matching remote → no action (unchanged)
pub async fn sync_notes(
    client: &WebDavClient,
    local_notes: &[Note],
    notes_dir: &Path,
) -> Result<NoteSyncResult, String> {
    let mut result = NoteSyncResult {
        pulled: Vec::new(),
        pushed: 0,
        deleted_remote: 0,
        errors: Vec::new(),
    };

    // Ensure the remote collection exists
    if let Err(e) = client.ensure_collection().await {
        log::warn!("WebDAV ensure_collection: {}", e);
    }

    // List remote files
    let remote_files = client.list_files().await?;

    log::info!(
        "WebDAV sync: {} local notes, {} remote files",
        local_notes.len(),
        remote_files.len()
    );

    // Index local notes by filename (uuid.org)
    let mut local_by_filename: std::collections::HashMap<String, &Note> =
        std::collections::HashMap::new();
    for note in local_notes {
        let filename = format!("{}.org", note.id);
        local_by_filename.insert(filename, note);
    }

    // Track which local notes were matched to remote files
    let mut matched_filenames: std::collections::HashSet<String> =
        std::collections::HashSet::new();

    // Process remote files
    for remote in &remote_files {
        matched_filenames.insert(remote.filename.clone());

        if let Some(&local_note) = local_by_filename.get(&remote.filename) {
            // Both exist — check etag
            let etag_matches = local_note
                .sync_etag
                .as_deref()
                .zip(remote.etag.as_deref())
                .is_some_and(|(local_etag, remote_etag)| local_etag == remote_etag);

            if !etag_matches {
                // Remote changed (or first sync) → pull
                match client.get_file(&remote.filename).await {
                    Ok((content, etag)) => {
                        let mut notes = convert::parse_notes(&content);
                        if let Some(mut note) = notes.pop() {
                            note.sync_etag = etag.or(remote.etag.clone());
                            result.pulled.push(note);
                        }
                    }
                    Err(e) => {
                        result
                            .errors
                            .push(format!("Failed to GET {}: {}", remote.filename, e));
                    }
                }
            }
        } else {
            // Remote only → pull as new
            match client.get_file(&remote.filename).await {
                Ok((content, etag)) => {
                    let mut notes = convert::parse_notes(&content);
                    if let Some(mut note) = notes.pop() {
                        note.sync_etag = etag.or(remote.etag.clone());
                        result.pulled.push(note);
                    }
                }
                Err(e) => {
                    result
                        .errors
                        .push(format!("Failed to GET {}: {}", remote.filename, e));
                }
            }
        }
    }

    // Push local notes not on remote (or with no etag = never synced)
    for note in local_notes {
        let filename = format!("{}.org", note.id);
        if !matched_filenames.contains(&filename) || note.sync_etag.is_none() {
            // Only push if truly not on remote (avoid double-push for pulled notes)
            if matched_filenames.contains(&filename) && note.sync_etag.is_some() {
                continue;
            }
            let content = OrgWriter::write_note_file(note);
            match client.put_file(&filename, &content).await {
                Ok(etag) => {
                    // Return updated note with etag
                    let mut pushed_note = note.clone();
                    pushed_note.sync_etag = etag;
                    result.pulled.push(pushed_note);
                    result.pushed += 1;
                }
                Err(e) => {
                    result
                        .errors
                        .push(format!("Failed to PUT {}: {}", filename, e));
                }
            }
        }
    }

    // Save pulled notes to local disk
    for note in &result.pulled {
        let filename = format!("{}.org", note.id);
        let path = notes_dir.join(&filename);
        let content = OrgWriter::write_note_file(note);
        if let Err(e) = std::fs::write(&path, &content) {
            log::error!("Failed to write {}: {}", path.display(), e);
        }
    }

    log::info!(
        "WebDAV sync complete: {} pulled, {} pushed, {} errors",
        result.pulled.len(),
        result.pushed,
        result.errors.len()
    );

    Ok(result)
}
