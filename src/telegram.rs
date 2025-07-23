use reqwest::Client;
use serde::Serialize;
use tracing::error;

#[derive(Serialize)]
struct SendMessagePayload<'a> {
    chat_id: &'a str,
    text: &'a str,
    parse_mode: &'a str,
}

#[derive(Clone)]
pub struct Sender {
    client: Client,
    api_key: String,
    chat_id: String,
}

impl Sender {
    pub fn new(api_key: String, chat_id: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            chat_id,
        }
    }

    pub async fn send(&self, message: &str) -> anyhow::Result<()> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.api_key);
        let payload = SendMessagePayload {
            chat_id: &self.chat_id,
            text: message,
            parse_mode: "MarkdownV2",
        };

        let res = self.client.post(&url).json(&payload).send().await?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_else(|_| "Could not read body".to_string());
            error!(status = %status, body = %body, "Telegram API returned non-200 status");
            return Err(anyhow::anyhow!("Telegram API error: {}", status));
        }
        Ok(())
    }
}

pub fn escape_markdown(s: &str) -> String {
    let chars = [
        '_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!',
    ];
    let mut escaped = String::with_capacity(s.len());
    for c in s.chars() {
        if chars.contains(&c) {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    escaped
}