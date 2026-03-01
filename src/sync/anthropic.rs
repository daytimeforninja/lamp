use std::collections::HashMap;

use serde::Deserialize;

const KEYRING_SERVER: &str = "anthropic-api";

/// Structured data extracted from an email by the AI model.
#[derive(Debug, Clone, Deserialize)]
pub struct ExtractedTaskData {
    pub title: String,
    pub priority: Option<String>,
    pub contexts: Option<Vec<String>>,
    pub deadline: Option<String>,
    pub scheduled: Option<String>,
    pub project: Option<String>,
    pub is_duplicate: Option<bool>,
    pub duplicate_of: Option<String>,
}

/// A suggestion produced by batch AI analysis of multiple emails at once.
#[derive(Debug, Clone, Deserialize)]
pub struct BatchEmailSuggestion {
    pub action_needed: bool,
    pub title: Option<String>,
    pub priority: Option<String>,
    pub contexts: Option<Vec<String>>,
    pub deadline: Option<String>,
    pub scheduled: Option<String>,
    pub project: Option<String>,
    pub is_duplicate: Option<bool>,
    pub duplicate_of: Option<String>,
}

/// Call the Anthropic Messages API to extract task data from an email.
pub async fn extract_task_from_email(
    api_key: &str,
    subject: &str,
    from: &str,
    date: Option<&str>,
    body_full: &str,
    user_hint: Option<&str>,
    available_contexts: &[String],
    project_names: &[String],
    existing_task_titles: &[String],
    today: &str,
) -> Result<ExtractedTaskData, String> {
    let system_prompt = build_system_prompt(
        available_contexts,
        project_names,
        existing_task_titles,
        today,
    );

    let mut user_msg = format!(
        "Subject: {}\nFrom: {}\nDate: {}\n\n{}",
        subject,
        from,
        date.unwrap_or("unknown"),
        // Cap body to ~4000 chars to stay within token budget
        &body_full.chars().take(4000).collect::<String>(),
    );
    if let Some(hint) = user_hint {
        if !hint.is_empty() {
            user_msg.push_str(&format!("\n\nUser context: {}", hint));
        }
    }

    let body = serde_json::json!({
        "model": "claude-haiku-4-5-20251001",
        "max_tokens": 400,
        "system": system_prompt,
        "messages": [
            { "role": "user", "content": user_msg }
        ]
    });

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("API request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("API error {}: {}", status, text));
    }

    let api_resp: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse API response: {}", e))?;

    // Extract text from the first content block
    let text = api_resp["content"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|block| block["text"].as_str())
        .ok_or_else(|| "No text in API response".to_string())?;

    // Strip markdown code fences if present
    let json_str = text
        .trim()
        .strip_prefix("```json")
        .or_else(|| text.trim().strip_prefix("```"))
        .unwrap_or(text.trim());
    let json_str = json_str.strip_suffix("```").unwrap_or(json_str).trim();

    serde_json::from_str::<ExtractedTaskData>(json_str)
        .map_err(|e| format!("Failed to parse extracted data: {} — raw: {}", e, text))
}

fn build_system_prompt(
    available_contexts: &[String],
    project_names: &[String],
    existing_task_titles: &[String],
    today: &str,
) -> String {
    let mut prompt = String::from(
        "You extract actionable tasks from emails. Return ONLY a JSON object, no explanation.\n\n\
         Rules:\n\
         - \"title\": concise imperative action (e.g. \"Reply to vendor quote\", \"Schedule dentist appointment\")\n\
         - \"priority\": \"A\" (urgent+important), \"B\" (important), \"C\" (low), or null\n\
         - \"contexts\": array of applicable contexts from the allowed list, or null\n\
         - \"deadline\": \"YYYY-MM-DD\" if a hard deadline is mentioned, else null\n\
         - \"scheduled\": \"YYYY-MM-DD\" if a specific do-date is mentioned, else null\n\
         - \"project\": name of an existing project this relates to, or null\n\
         - \"is_duplicate\": true if an existing task already covers this email's action\n\
         - \"duplicate_of\": title of the existing task it duplicates, or null\n\n",
    );

    prompt.push_str(&format!("Today's date: {}\n\n", today));

    if !available_contexts.is_empty() {
        prompt.push_str("Available contexts (pick from these only): ");
        prompt.push_str(&available_contexts.join(", "));
        prompt.push_str("\n\n");
    }

    if !project_names.is_empty() {
        prompt.push_str("Existing projects: ");
        prompt.push_str(&project_names.join(", "));
        prompt.push_str("\n\n");
    }

    if !existing_task_titles.is_empty() {
        // Limit to most recent 100 titles to avoid huge prompts
        let titles: Vec<&str> = existing_task_titles
            .iter()
            .take(100)
            .map(|s| s.as_str())
            .collect();
        prompt.push_str("Existing task titles (for duplicate detection):\n");
        for t in titles {
            prompt.push_str(&format!("- {}\n", t));
        }
        prompt.push('\n');
    }

    prompt
}

/// Call the Anthropic Messages API to extract tasks from all emails in one batch.
pub async fn extract_tasks_from_emails_batch(
    api_key: &str,
    emails: Vec<(u32, String, String, Option<String>, String)>, // (uid, subject, from, date, body)
    available_contexts: &[String],
    project_names: &[String],
    existing_task_titles: &[String],
    today: &str,
) -> Result<Vec<(u32, BatchEmailSuggestion)>, String> {
    let system_prompt = build_batch_system_prompt(
        available_contexts,
        project_names,
        existing_task_titles,
        today,
    );

    let mut user_msg = String::new();
    let uids: Vec<u32> = emails.iter().map(|(uid, ..)| *uid).collect();

    for (i, (_uid, subject, from, date, body)) in emails.iter().enumerate() {
        let capped_body: String = body.chars().take(2000).collect();
        user_msg.push_str(&format!(
            "--- Email {} ---\nSubject: {}\nFrom: {}\nDate: {}\n\n{}\n\n",
            i + 1,
            subject,
            from,
            date.as_deref().unwrap_or("unknown"),
            capped_body,
        ));
    }

    let max_tokens = std::cmp::min(300 * emails.len(), 4096);

    let body = serde_json::json!({
        "model": "claude-haiku-4-5-20251001",
        "max_tokens": max_tokens,
        "system": system_prompt,
        "messages": [
            { "role": "user", "content": user_msg }
        ]
    });

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("API request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("API error {}: {}", status, text));
    }

    let api_resp: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse API response: {}", e))?;

    let text = api_resp["content"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|block| block["text"].as_str())
        .ok_or_else(|| "No text in API response".to_string())?;

    // Strip markdown code fences if present
    let json_str = text
        .trim()
        .strip_prefix("```json")
        .or_else(|| text.trim().strip_prefix("```"))
        .unwrap_or(text.trim());
    let json_str = json_str.strip_suffix("```").unwrap_or(json_str).trim();

    let suggestions: Vec<BatchEmailSuggestion> = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse batch suggestions: {} — raw: {}", e, text))?;

    // Zip suggestions with UIDs — if the model returned fewer, pair what we have
    let results: Vec<(u32, BatchEmailSuggestion)> = uids
        .into_iter()
        .zip(suggestions)
        .collect();

    Ok(results)
}

fn build_batch_system_prompt(
    available_contexts: &[String],
    project_names: &[String],
    existing_task_titles: &[String],
    today: &str,
) -> String {
    let mut prompt = String::from(
        "You analyze emails and suggest tasks. Return ONLY a JSON array with one object per email, in order. No explanation.\n\n\
         Each object must have:\n\
         - \"action_needed\": boolean — true if the email requires the user to do something\n\
         - \"title\": concise imperative action (e.g. \"Reply to vendor quote\"), or null if no action needed\n\
         - \"priority\": \"A\" (urgent+important), \"B\" (important), \"C\" (low), or null\n\
         - \"contexts\": array of applicable contexts from the allowed list, or null\n\
         - \"deadline\": \"YYYY-MM-DD\" if a hard deadline is mentioned, else null\n\
         - \"scheduled\": \"YYYY-MM-DD\" if a specific do-date is mentioned, else null\n\
         - \"project\": name of an existing project this relates to, or null\n\
         - \"is_duplicate\": true if an existing task already covers this email's action\n\
         - \"duplicate_of\": title of the existing task it duplicates, or null\n\n",
    );

    prompt.push_str(&format!("Today's date: {}\n\n", today));

    if !available_contexts.is_empty() {
        prompt.push_str("Available contexts (pick from these only): ");
        prompt.push_str(&available_contexts.join(", "));
        prompt.push_str("\n\n");
    }

    if !project_names.is_empty() {
        prompt.push_str("Existing projects: ");
        prompt.push_str(&project_names.join(", "));
        prompt.push_str("\n\n");
    }

    if !existing_task_titles.is_empty() {
        let titles: Vec<&str> = existing_task_titles
            .iter()
            .take(100)
            .map(|s| s.as_str())
            .collect();
        prompt.push_str("Existing task titles (for duplicate detection):\n");
        for t in titles {
            prompt.push_str(&format!("- {}\n", t));
        }
        prompt.push('\n');
    }

    prompt
}

/// Verify the API key with a minimal request.
pub async fn test_api_key(api_key: &str) -> Result<String, String> {
    let body = serde_json::json!({
        "model": "claude-haiku-4-5-20251001",
        "max_tokens": 4,
        "messages": [
            { "role": "user", "content": "Reply with OK" }
        ]
    });

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if resp.status().is_success() {
        Ok("API key valid".to_string())
    } else if resp.status().as_u16() == 401 {
        Err("Invalid API key".to_string())
    } else {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        Err(format!("API error {}: {}", status, text))
    }
}

/// Store the Anthropic API key in the system keyring.
pub async fn store_api_key(key: &str) -> Result<(), String> {
    let keyring = oo7::Keyring::new()
        .await
        .map_err(|e| format!("Failed to connect to keyring: {}", e))?;

    let mut attrs = HashMap::new();
    attrs.insert("service", super::keyring::SERVICE_NAME);
    attrs.insert("server", KEYRING_SERVER);

    keyring
        .create_item(
            "Lamp Anthropic API Key",
            &attrs,
            key.as_bytes(),
            true,
        )
        .await
        .map_err(|e| format!("Failed to store API key: {}", e))?;

    Ok(())
}

/// Load the Anthropic API key from the system keyring.
pub async fn load_api_key() -> Result<Option<String>, String> {
    let keyring = oo7::Keyring::new()
        .await
        .map_err(|e| format!("Failed to connect to keyring: {}", e))?;

    let mut attrs = HashMap::new();
    attrs.insert("service", super::keyring::SERVICE_NAME);
    attrs.insert("server", KEYRING_SERVER);

    let items = keyring
        .search_items(&attrs)
        .await
        .map_err(|e| format!("Failed to search keyring: {}", e))?;

    if let Some(item) = items.first() {
        let secret_bytes = item
            .secret()
            .await
            .map_err(|e| format!("Failed to read secret: {}", e))?;
        let key = String::from_utf8(secret_bytes.to_vec())
            .map_err(|e| format!("Invalid UTF-8 in secret: {}", e))?;
        if !key.is_empty() {
            return Ok(Some(key));
        }
    }

    Ok(None)
}
