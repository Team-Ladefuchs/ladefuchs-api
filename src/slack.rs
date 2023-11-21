use std::ffi::OsStr;
use std::fmt::Display;
use std::sync::atomic::{AtomicU16, Ordering};

use axum::async_trait;
use axum::http::header::CONTENT_TYPE;
use axum::http::HeaderMap;

pub const MALIK: &str = "<@U028N463G1J>";

#[derive(Debug)]
pub struct Slack {
    token: String,
    channel_id: String,
    client: reqwest::Client,
    send_count: AtomicU16,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Message {
    channel: String,
    text: String,
}

pub enum Emoji {
    ImageFrame,
    ElectricPlug,
    Art,
    Warning,
    New,
    Rename,
    Error,
}

impl Display for Emoji {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let emoji = match &self {
            Emoji::Warning => "interrobang",
            Emoji::ImageFrame => "frame_with_picture",
            Emoji::Error => "boom",
            Emoji::Art => "art",
            Emoji::Rename => "writing_hand",
            Emoji::New => "new",
            Emoji::ElectricPlug => "electric_plug",
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
    async fn send(&self, emoji: Option<Emoji>, text: &str);
    fn reset_count(&self);
    fn inc_count(&self);
    async fn send_new_image_slack(&self, extra: (&str, Emoji), filename: &OsStr);
    async fn send_rename_image(&self, prefix: &str, old_file: &OsStr, new_file: &OsStr);
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
            send_count: AtomicU16::new(0),
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

    pub async fn send(&self, emoji: Option<Emoji>, text: &str) {
        let text = match emoji {
            Some(emoji) => format!("{} {}", emoji, text),
            None => text.to_owned(),
        };

        let message = Message {
            channel: self.channel_id.clone(),
            text,
        };

        if let Err(err) = self.call_api(&message).await {
            tracing::warn!(location = "Slack API", error = %err);
        }
    }

    pub fn inc_count(&self) {
        self.send_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn count(&self) -> u16 {
        self.send_count.load(Ordering::Relaxed)
    }

    fn reset_count(&self) {
        self.send_count.store(0, Ordering::Relaxed);
    }
}

#[async_trait]
impl SlackClient for &Option<Slack> {
    async fn send(&self, emoji: Option<Emoji>, text: &str) {
        if let Some(me) = &self {
            me.send(emoji, text).await;
        }
    }
    fn reset_count(&self) {
        if let Some(me) = &self {
            me.reset_count();
        }
    }
    fn inc_count(&self) {
        if let Some(me) = &self {
            me.inc_count();
        }
    }

    async fn send_new_image_slack(&self, extra: (&str, Emoji), filename: &OsStr) {
        let (prefix, emoji) = extra;
        self.send(
            Some(emoji),
            &format!(
                "New {} image filename: {}",
                prefix,
                filename.to_string_lossy()
            ),
        )
        .await;
    }

    async fn send_rename_image(&self, prefix: &str, old_file: &OsStr, new_file: &OsStr) {
        self.send(
            Some(Emoji::Rename),
            &format!(
                "Renamed {} image\nold name: {}, new name {}",
                prefix,
                old_file.to_string_lossy(),
                new_file.to_string_lossy()
            ),
        )
        .await;
    }
}
