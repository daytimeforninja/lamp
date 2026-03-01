use chrono::NaiveDateTime;
use futures::TryStreamExt;

type ImapSession = async_imap::Session<async_native_tls::TlsStream<async_std::net::TcpStream>>;

/// Resolve a folder name case-insensitively against the server's mailbox list.
async fn resolve_folder(
    session: &mut ImapSession,
    folder: &str,
) -> Result<String, String> {
    let folders_stream = session
        .list(Some(""), Some("*"))
        .await
        .map_err(|e| format!("Failed to list folders: {}", e))?;
    let folders: Vec<_> = folders_stream
        .try_collect()
        .await
        .map_err(|e| format!("Failed to collect folders: {}", e))?;

    for f in &folders {
        if f.name().eq_ignore_ascii_case(folder) {
            return Ok(f.name().to_string());
        }
    }

    Err(format!(
        "Folder '{}' not found (available: {})",
        folder,
        folders.iter().map(|f| f.name().to_string()).collect::<Vec<_>>().join(", ")
    ))
}

/// An email fetched from an IMAP folder.
#[derive(Debug, Clone)]
pub struct ImapEmail {
    pub uid: u32,
    pub subject: String,
    pub from: String,
    pub date: Option<NaiveDateTime>,
    pub body_preview: String,
    pub body_full: String,
}

/// Connect to IMAP over TLS and return a logged-in session.
async fn connect_and_login(
    host: &str,
    username: &str,
    password: &str,
) -> Result<async_imap::Session<async_native_tls::TlsStream<async_std::net::TcpStream>>, String> {
    let tls = async_native_tls::TlsConnector::new();
    let tcp = async_std::net::TcpStream::connect((host, 993))
        .await
        .map_err(|e| format!("TCP connect failed: {}", e))?;
    let tls_stream = tls
        .connect(host, tcp)
        .await
        .map_err(|e| format!("TLS connect failed: {}", e))?;

    let client = async_imap::Client::new(tls_stream);
    let session = client
        .login(username, password)
        .await
        .map_err(|e| format!("IMAP login failed: {}", e.0))?;

    Ok(session)
}

/// Fetch all emails from the configured IMAP folder.
pub async fn fetch_emails(
    host: &str,
    username: &str,
    password: &str,
    folder: &str,
) -> Result<Vec<ImapEmail>, String> {
    let mut session = connect_and_login(host, username, password).await?;

    let folder = resolve_folder(&mut session, folder).await?;
    session
        .select(&folder)
        .await
        .map_err(|e| format!("Failed to select folder '{}': {}", folder, e))?;

    let messages_stream = session
        .fetch("1:*", "(UID BODY.PEEK[])")
        .await
        .map_err(|e| format!("IMAP fetch failed: {}", e))?;

    let messages: Vec<_> = messages_stream
        .try_collect()
        .await
        .map_err(|e| format!("IMAP stream error: {}", e))?;

    let mut emails = Vec::new();
    for msg in &messages {
        let uid = msg.uid.unwrap_or(0);
        let body = match msg.body() {
            Some(b) => b,
            None => continue,
        };

        let parsed = match mail_parser::MessageParser::default().parse(body) {
            Some(p) => p,
            None => continue,
        };

        let subject = parsed
            .subject()
            .unwrap_or("(no subject)")
            .to_string();

        let from = parsed
            .from()
            .and_then(|addrs| addrs.first())
            .map(|a| {
                if let Some(name) = a.name() {
                    name.to_string()
                } else {
                    a.address().unwrap_or("unknown").to_string()
                }
            })
            .unwrap_or_else(|| "unknown".to_string());

        let date = parsed.date().and_then(|dt| {
            NaiveDateTime::parse_from_str(
                &format!(
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                    dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second
                ),
                "%Y-%m-%d %H:%M:%S",
            )
            .ok()
        });

        let body_text = parsed
            .body_text(0)
            .unwrap_or_default()
            .to_string();
        let body_preview: String = body_text.chars().take(200).collect();

        emails.push(ImapEmail {
            uid,
            subject,
            from,
            date,
            body_preview,
            body_full: body_text,
        });
    }

    session.logout().await.ok();
    Ok(emails)
}

/// Archive an email by moving it to the Archive folder.
pub async fn archive_email(
    host: &str,
    username: &str,
    password: &str,
    folder: &str,
    uid: u32,
) -> Result<u32, String> {
    let mut session = connect_and_login(host, username, password).await?;

    let folder = resolve_folder(&mut session, folder).await?;
    session
        .select(&folder)
        .await
        .map_err(|e| format!("Failed to select folder '{}': {}", folder, e))?;

    let archive = resolve_folder(&mut session, "Archive").await?;
    let uid_set = format!("{}", uid);
    session
        .uid_mv(&uid_set, &archive)
        .await
        .map_err(|e| format!("Failed to move email to Archive: {}", e))?;

    session.logout().await.ok();
    Ok(uid)
}

/// Test IMAP connection — login, list folders, return folder count.
pub async fn test_connection(
    host: &str,
    username: &str,
    password: &str,
) -> Result<String, String> {
    let mut session = connect_and_login(host, username, password).await?;

    let folders_stream = session
        .list(Some(""), Some("*"))
        .await
        .map_err(|e| format!("Failed to list folders: {}", e))?;

    let folders: Vec<_> = folders_stream
        .try_collect()
        .await
        .map_err(|e| format!("Failed to collect folders: {}", e))?;

    let count = folders.len();
    session.logout().await.ok();
    Ok(format!("Connected ({} folders)", count))
}
