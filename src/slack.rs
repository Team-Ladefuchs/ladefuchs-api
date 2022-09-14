use std::fmt::Display;

use axum::async_trait;
use axum::http::header::CONTENT_TYPE;
use axum::http::HeaderMap;

pub const MALIK: &str = "<@U028N463G1J>";

#[derive(Clone, Debug)]
pub struct Slack {
    token: String,
    channel_id: String,
    client: reqwest::Client,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Message {
    channel: String,
    text: String,
}

pub enum MessageEmoji {
    ImageFrame,
    Warning,
    Rename,
    Error,
}

impl Display for MessageEmoji {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let emoji = match &self {
            MessageEmoji::Warning => "interrobang",
            MessageEmoji::ImageFrame => "frame_with_picture",
            MessageEmoji::Error => "boom",
            MessageEmoji::Rename => "writing_hand",
        };
        write!(f, ":{}:", emoji)
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
struct SlackResponse {
    ok: bool,
    error: Option<String>,
}

#[async_trait]
pub trait SlackClient {
    async fn send(&self, emoji: Option<MessageEmoji>, text: &str);
}

impl Slack {
    pub fn new(token: String, channel_id: String) -> Result<Self, reqwest::Error> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        Ok(Self {
            token,
            channel_id,
            client,
        })
    }

    async fn call_api(&self, message: &Message) -> Result<SlackResponse, eyre::Error> {
        let json: SlackResponse = self
            .client
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

#[async_trait]
impl SlackClient for &Option<Slack> {
    async fn send(&self, emoji: Option<MessageEmoji>, text: &str) {
        if let Some(me) = &self {
            let text = match emoji {
                Some(emoji) => format!("{} {}", emoji, text),
                None => text.to_owned(),
            };

            let message = Message {
                channel: me.channel_id.clone(),
                text,
            };

            if let Err(err) = me.call_api(&message).await {
                tracing::warn!(location = "Slack API", error = %err);
            }
        }
    }
}
