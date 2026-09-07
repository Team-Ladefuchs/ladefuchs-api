use std::fmt::Display;
use std::path::Path;
// use std::sync::atomic::AtomicU16;

use async_trait::async_trait;
use reqwest::header::{CONTENT_TYPE, HeaderMap};

pub const MALIK: &str = "<@U028N463G1J>";

#[derive(Debug)]
pub struct Slack {
    token: String,
    channel_id: String,
    client: reqwest::Client,
}

mod slack_api {

    #[derive(Clone, Debug, serde::Serialize)]
    pub struct Message {
        pub channel: String,
        pub text: String,
        pub mrkdwn: bool,
    }
}

#[derive(Debug)]
pub enum Emoji {
    ImageFrame,
    ElectricPlug,
    Warning,
    // New,
    Down,
    Dollar,
    Rename,
    Error,
    Ladefuchs,
}

impl Display for Emoji {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let emoji = match &self {
            Emoji::Warning => "interrobang",
            Emoji::ImageFrame => "frame_with_picture",
            Emoji::Error => "boom",
            Emoji::Rename => "writing_hand",
            // Emoji::New => "new",
            Emoji::Dollar => "heavy_dollar_sign",
            Emoji::ElectricPlug => "electric_plug",
            Emoji::Down => "arrow_down",
            Emoji::Ladefuchs => "ladefuchs",
        };
        write!(f, ":{}:", emoji)
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
struct SlackResponse {
    ok: bool,
    error: Option<String>,
}

#[derive(Debug, Default)]
pub struct TextMessage {
    pub emoji: Option<Emoji>,
    pub text: String,
    pub markdown: bool,
}

#[derive(Debug)]
pub struct LinkPreview<'a> {
    pub link: &'a url::Url,
    pub text: &'a str,
}

impl Display for LinkPreview<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}|{}>", self.link, self.text)
    }
}

#[async_trait]
pub trait SlackClient {
    async fn send_message(&self, message: TextMessage);
    async fn send_warning_message(&self, error_text: String);
    async fn send_rename_image(&self, prefix: &str, old_file: &Path, new_file: &Path);
    // fn reset_count(&self);
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
            // send_count: AtomicU16::new(0),
        })
    }

    async fn call_api(&self, message: &slack_api::Message) -> Result<SlackResponse, eyre::Error> {
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

    pub async fn send_error_message(&self, error_text: String) {
        self.send(TextMessage {
            emoji: Some(Emoji::Error),
            text: error_text,
            markdown: false,
        })
        .await
    }

    pub async fn send_warning_message(&self, error_text: String) {
        self.send(TextMessage {
            emoji: Some(Emoji::Warning),
            text: error_text,
            markdown: false,
        })
        .await
    }

    pub async fn send(&self, message: TextMessage) {
        let text = match message.emoji {
            Some(emoji) => format!("{} {}", emoji, message.text),
            None => message.text,
        };

        let message = slack_api::Message {
            channel: self.channel_id.clone(),
            text,
            mrkdwn: message.markdown,
        };

        if let Err(err) = self.call_api(&message).await {
            tracing::warn!(location = "Slack API", error = %err);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::slack_api;

    #[test]
    fn message_uses_slack_post_message_fields() {
        let message = slack_api::Message {
            channel: "C123".to_owned(),
            text: "hello".to_owned(),
            mrkdwn: true,
        };

        assert_eq!(
            serde_json::to_value(message).unwrap(),
            serde_json::json!({
                "channel": "C123",
                "text": "hello",
                "mrkdwn": true,
            })
        );
    }
}

#[async_trait]
impl SlackClient for &Option<Slack> {
    async fn send_message(&self, message: TextMessage) {
        if let Some(me) = &self {
            me.send(message).await;
        }
    }

    async fn send_warning_message(&self, error_text: String) {
        if let Some(me) = &self {
            me.send_warning_message(error_text).await;
        }
    }

    async fn send_rename_image(&self, prefix: &str, old_file: &Path, new_file: &Path) {
        self.send_message(TextMessage {
            emoji: Some(Emoji::Rename),
            text: format!(
                "Renamed {} image\nold name: {}, new name {}",
                prefix,
                old_file.file_name().unwrap_or_default().to_string_lossy(),
                new_file.file_name().unwrap_or_default().to_string_lossy()
            ),
            markdown: false,
        })
        .await;
    }
}
