use std::fmt::Display;

use axum::http::header::CONTENT_TYPE;
use axum::http::HeaderMap;
use reqwest::Client;

#[derive(Clone, Debug)]
pub struct Slack {
    token: String,
    channel_id: String,
    client: Option<reqwest::Client>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Message {
    channel: String,
    text: String,
}

pub enum MessageEmoji {
    Success,
    Warning,
    Error,
}

impl Display for MessageEmoji {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let emoji = match &self {
            MessageEmoji::Success => "tada",
            MessageEmoji::Warning => "interrobang",
            MessageEmoji::Error => "boom",
        };
        write!(f, ":{}:", emoji)
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
struct SlackResponse {
    ok: bool,
    error: Option<String>,
}

impl Slack {
    pub fn new(token: String, channel_id: String) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

        let client = if token != "null"
            && !token.is_empty()
            && channel_id != "null"
            && !channel_id.is_empty()
        {
            reqwest::Client::builder()
                .default_headers(headers)
                .build()
                .ok()
        } else {
            None
        };

        Self {
            token,
            channel_id,
            client,
        }
    }

    pub async fn send(&self, emoji: Option<MessageEmoji>, text: &str) {
        if let Some(client) = &self.client {
            let text = match emoji {
                Some(emoji) => format!("{} {}", emoji, text),
                None => text.to_owned(),
            };

            let message = Message {
                channel: self.channel_id.clone(),
                text,
            };

            if let Err(err) = self.call_api(client, &message).await {
                tracing::warn!(location = "Slack API", error = %err);
            }
        }
    }
    async fn call_api(
        &self,
        client: &Client,
        message: &Message,
    ) -> Result<SlackResponse, eyre::Error> {
        let json: SlackResponse = client
            .post("https://slack.com/api/chat.postMessage")
            .bearer_auth(&self.token)
            .json(message)
            .send()
            .await?
            .json()
            .await?;

        match json.error {
            Some(msg) if !json.ok => Err(eyre::Error::msg(msg)),
            _ => Ok(json),
        }
    }
}
