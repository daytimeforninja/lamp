use std::collections::HashMap;

pub(crate) const SERVICE_NAME: &str = "lamp-sync";

/// Store CalDAV credentials in the system keyring via Secret Service.
pub async fn store_credentials(
    server: &str,
    username: &str,
    password: &str,
) -> Result<(), String> {
    let keyring = oo7::Keyring::new()
        .await
        .map_err(|e| format!("Failed to connect to keyring: {}", e))?;

    let mut attrs = HashMap::new();
    attrs.insert("service", SERVICE_NAME);
    attrs.insert("server", server);

    let secret = format!("{}:{}", username, password);

    keyring
        .create_item(
            &format!("Lamp CalDAV ({})", server),
            &attrs,
            secret.as_bytes(),
            true, // replace existing
        )
        .await
        .map_err(|e| format!("Failed to store credentials: {}", e))?;

    Ok(())
}

/// Load CalDAV credentials from the system keyring.
/// Returns (username, password) if found.
pub async fn load_credentials(server: &str) -> Result<Option<(String, String)>, String> {
    let keyring = oo7::Keyring::new()
        .await
        .map_err(|e| format!("Failed to connect to keyring: {}", e))?;

    let mut attrs = HashMap::new();
    attrs.insert("service", SERVICE_NAME);
    attrs.insert("server", server);

    let items = keyring
        .search_items(&attrs)
        .await
        .map_err(|e| format!("Failed to search keyring: {}", e))?;

    if let Some(item) = items.first() {
        let secret_bytes = item
            .secret()
            .await
            .map_err(|e| format!("Failed to read secret: {}", e))?;
        let secret = String::from_utf8(secret_bytes.to_vec())
            .map_err(|e| format!("Invalid UTF-8 in secret: {}", e))?;
        if let Some((username, password)) = secret.split_once(':') {
            return Ok(Some((username.to_string(), password.to_string())));
        }
    }

    Ok(None)
}

/// Delete CalDAV credentials from the system keyring.
pub async fn delete_credentials(server: &str) -> Result<(), String> {
    let keyring = oo7::Keyring::new()
        .await
        .map_err(|e| format!("Failed to connect to keyring: {}", e))?;

    let mut attrs = HashMap::new();
    attrs.insert("service", SERVICE_NAME);
    attrs.insert("server", server);

    let items = keyring
        .search_items(&attrs)
        .await
        .map_err(|e| format!("Failed to search keyring: {}", e))?;

    for item in items {
        item.delete()
            .await
            .map_err(|e| format!("Failed to delete credential: {}", e))?;
    }

    Ok(())
}
