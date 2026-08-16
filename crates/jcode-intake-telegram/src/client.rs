//! Minimal Telegram Bot API transport.
//!
//! Telegram requires the bot token in the request path. Consequently no
//! `reqwest::Error` or response body is ever exposed through this public API:
//! both can contain the full request URL. Errors are reduced to safe classes
//! and status codes before crossing the boundary.

use std::fmt;

use reqwest::blocking::Client;
use serde_json::{Value, json};

use crate::BotToken;

pub struct TelegramClient {
    http: Client,
    base_url: String,
    token: BotToken,
}

impl TelegramClient {
    #[must_use]
    pub fn new(token: BotToken) -> Self {
        Self::with_base_url(token, "https://api.telegram.org")
    }

    #[must_use]
    pub fn with_base_url(token: BotToken, base_url: impl Into<String>) -> Self {
        Self {
            http: Client::new(),
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            token,
        }
    }

    /// Long-poll for updates after `offset`.
    pub fn get_updates(
        &self,
        offset: Option<i64>,
        timeout_seconds: u32,
    ) -> Result<Vec<Value>, ApiError> {
        let body = json!({
            "offset": offset,
            "timeout": timeout_seconds,
            "allowed_updates": [],
        });
        let result = self.call("getUpdates", &body)?;
        result.as_array().cloned().ok_or(ApiError::InvalidResponse)
    }

    /// Send a plain-text response to a conversation.
    pub fn send_message(&self, conversation: &str, text: &str) -> Result<Value, ApiError> {
        self.call(
            "sendMessage",
            &json!({"chat_id": conversation, "text": text}),
        )
    }

    fn call(&self, method: &str, body: &Value) -> Result<Value, ApiError> {
        let url = format!("{}/bot{}/{}", self.base_url, self.token.expose(), method);
        let response = self
            .http
            .post(url)
            .json(body)
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
        body.get("result").cloned().ok_or(ApiError::InvalidResponse)
    }
}

impl fmt::Debug for TelegramClient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TelegramClient")
            .field("base_url", &self.base_url)
            .field("token", &"[REDACTED]")
            .finish_non_exhaustive()
    }
}

/// Secret-safe error surface. Deliberately carries neither URLs nor bodies.
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
            Self::Network => formatter.write_str("Telegram API request failed"),
            Self::HttpStatus(status) => write!(formatter, "Telegram API returned HTTP {status}"),
            Self::Rejected => formatter.write_str("Telegram API rejected the request"),
            Self::InvalidResponse => {
                formatter.write_str("Telegram API returned an invalid response")
            }
        }
    }
}

impl std::error::Error for ApiError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };

    const TOKEN: &str = "123456789:AAHfSHFyTvJmL5RkQxWnPzZbCdEfGhIjKlM";

    #[test]
    fn debug_output_redacts_the_token() {
        let client = TelegramClient::new(BotToken::new(TOKEN).unwrap());
        let output = format!("{client:?}");
        assert!(!output.contains(TOKEN));
        assert!(output.contains("[REDACTED]"));
    }

    #[test]
    fn every_public_error_surface_is_token_free() {
        let errors = [
            ApiError::Network,
            ApiError::HttpStatus(401),
            ApiError::Rejected,
            ApiError::InvalidResponse,
        ];
        for error in errors {
            let output = format!("{error:?}: {error}");
            assert!(!output.contains(TOKEN));
            assert!(!output.contains("/bot"));
        }
    }

    #[test]
    fn get_updates_uses_the_public_http_interface_and_parses_results() {
        let (base_url, request) = one_shot_server(
            200,
            r#"{"ok":true,"result":[{"update_id":7,"message":{"text":"hello"}}]}"#,
        );
        let client = TelegramClient::with_base_url(BotToken::new(TOKEN).unwrap(), base_url);
        let updates = client.get_updates(Some(7), 0).expect("updates parse");
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0]["update_id"], 7);

        let request = request.join().expect("request captured");
        assert!(request.starts_with(&format!("POST /bot{TOKEN}/getUpdates")));
        assert!(request.contains("\"offset\":7"));
    }

    #[test]
    fn send_message_uses_post_json_and_returns_the_result() {
        let (base_url, request) = one_shot_server(
            200,
            r#"{"ok":true,"result":{"message_id":9,"chat":{"id":555}}}"#,
        );
        let client = TelegramClient::with_base_url(BotToken::new(TOKEN).unwrap(), base_url);
        let sent = client.send_message("555", "hello").expect("send succeeds");
        assert_eq!(sent["message_id"], 9);

        let request = request.join().expect("request captured");
        assert!(request.starts_with(&format!("POST /bot{TOKEN}/sendMessage")));
        assert!(request.contains("\"chat_id\":\"555\""));
        assert!(request.contains("\"text\":\"hello\""));
    }

    #[test]
    fn http_failure_does_not_expose_the_token_or_response_body() {
        let private_body = format!("provider echoed private credential {TOKEN}");
        let (base_url, request) = one_shot_server(401, &private_body);
        let client = TelegramClient::with_base_url(BotToken::new(TOKEN).unwrap(), base_url);
        let error = client.get_updates(None, 0).expect_err("request fails");
        let output = format!("{error:?}: {error}");
        assert_eq!(error, ApiError::HttpStatus(401));
        assert!(!output.contains(TOKEN));
        assert!(!output.contains(&private_body));
        request.join().expect("request completed");
    }

    fn one_shot_server(status: u16, response_body: &str) -> (String, thread::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let address = listener.local_addr().expect("server address");
        let body = response_body.to_owned();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept request");
            let mut bytes = Vec::new();
            let mut buffer = [0_u8; 4096];
            loop {
                let read = stream.read(&mut buffer).expect("read request");
                if read == 0 {
                    break;
                }
                bytes.extend_from_slice(&buffer[..read]);
                if complete_http_request(&bytes) {
                    break;
                }
            }
            let reason = if status == 200 { "OK" } else { "Unauthorized" };
            write!(
                stream,
                "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .expect("write response");
            String::from_utf8(bytes).expect("request is utf-8")
        });
        (format!("http://{address}"), handle)
    }

    fn complete_http_request(bytes: &[u8]) -> bool {
        let text = String::from_utf8_lossy(bytes);
        let Some((headers, body)) = text.split_once("\r\n\r\n") else {
            return false;
        };
        let content_length = headers
            .lines()
            .find_map(|line| {
                line.strip_prefix("content-length: ")
                    .or_else(|| line.strip_prefix("Content-Length: "))
            })
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(0);
        body.len() >= content_length
    }
}
