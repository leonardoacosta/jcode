use std::{fmt, net::TcpStream};

use reqwest::blocking::Client;
use serde_json::{Value, json};
use tungstenite::{Message, WebSocket, connect, stream::MaybeTlsStream};

use crate::{AppToken, BotToken, SlackTransport};

pub struct SlackClient {
    http: Client,
    api_base: String,
    app_token: AppToken,
    bot_token: BotToken,
    socket: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
}

impl SlackClient {
    #[must_use]
    pub fn new(app_token: AppToken, bot_token: BotToken) -> Self {
        Self::with_api_base(app_token, bot_token, "https://slack.com/api")
    }

    #[must_use]
    pub fn with_api_base(
        app_token: AppToken,
        bot_token: BotToken,
        api_base: impl Into<String>,
    ) -> Self {
        Self {
            http: Client::new(),
            api_base: api_base.into().trim_end_matches('/').to_owned(),
            app_token,
            bot_token,
            socket: None,
        }
    }

    fn connect_socket(&mut self) -> Result<(), ApiError> {
        let response = self
            .http
            .post(format!("{}/apps.connections.open", self.api_base))
            .bearer_auth(self.app_token.expose())
            .send()
            .map_err(|_| ApiError::Network)?;
        let status = response.status();
        if !status.is_success() {
            return Err(ApiError::HttpStatus(status.as_u16()));
        }
        let body: Value = response.json().map_err(|_| ApiError::InvalidResponse)?;
        if body.get("ok").and_then(Value::as_bool) != Some(true) {
            return Err(ApiError::Rejected);
        }
        let url = body
            .get("url")
            .and_then(Value::as_str)
            .ok_or(ApiError::InvalidResponse)?;
        let (socket, _) = connect(url).map_err(|_| ApiError::Network)?;
        self.socket = Some(socket);
        Ok(())
    }

    fn socket(&mut self) -> Result<&mut WebSocket<MaybeTlsStream<TcpStream>>, ApiError> {
        if self.socket.is_none() {
            self.connect_socket()?;
        }
        self.socket.as_mut().ok_or(ApiError::Network)
    }
}

impl SlackTransport for SlackClient {
    fn next_envelope(&mut self) -> Result<Option<Value>, ApiError> {
        loop {
            let message = match self.socket()?.read() {
                Ok(message) => message,
                Err(_) => {
                    self.socket = None;
                    return Err(ApiError::Network);
                }
            };
            match message {
                Message::Text(text) => {
                    let value: Value = serde_json::from_str(text.as_ref())
                        .map_err(|_| ApiError::InvalidResponse)?;
                    match value.get("type").and_then(Value::as_str) {
                        Some("hello") => continue,
                        Some("disconnect") => {
                            self.socket = None;
                            return Err(ApiError::Network);
                        }
                        _ => return Ok(Some(value)),
                    }
                }
                Message::Ping(payload) => {
                    self.socket()?
                        .send(Message::Pong(payload))
                        .map_err(|_| ApiError::Network)?;
                }
                Message::Close(_) => {
                    self.socket = None;
                    return Err(ApiError::Network);
                }
                _ => {}
            }
        }
    }

    fn acknowledge(&mut self, envelope_id: &str) -> Result<(), ApiError> {
        self.socket()?
            .send(Message::Text(
                json!({"envelope_id": envelope_id}).to_string().into(),
            ))
            .map_err(|_| {
                self.socket = None;
                ApiError::Network
            })
    }

    fn send_message(&mut self, conversation: &str, text: &str) -> Result<Value, ApiError> {
        let channel = conversation.strip_prefix("sl:").unwrap_or(conversation);
        let response = self
            .http
            .post(format!("{}/chat.postMessage", self.api_base))
            .bearer_auth(self.bot_token.expose())
            .json(&json!({"channel": channel, "text": text}))
            .send()
            .map_err(|_| ApiError::Network)?;
        let status = response.status();
        if !status.is_success() {
            return Err(ApiError::HttpStatus(status.as_u16()));
        }
        let body: Value = response.json().map_err(|_| ApiError::InvalidResponse)?;
        if body.get("ok").and_then(Value::as_bool) != Some(true) {
            return Err(ApiError::Rejected);
        }
        Ok(body)
    }
}

impl fmt::Debug for SlackClient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SlackClient")
            .field("api_base", &self.api_base)
            .field("app_token", &"[REDACTED]")
            .field("bot_token", &"[REDACTED]")
            .field("connected", &self.socket.is_some())
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiError {
    Network,
    HttpStatus(u16),
    Rejected,
    InvalidResponse,
}

impl fmt::Display for ApiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network => formatter.write_str("Slack API connection failed"),
            Self::HttpStatus(status) => write!(formatter, "Slack API returned HTTP {status}"),
            Self::Rejected => formatter.write_str("Slack API rejected the request"),
            Self::InvalidResponse => formatter.write_str("Slack API returned an invalid response"),
        }
    }
}

impl std::error::Error for ApiError {}
