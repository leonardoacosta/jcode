use regex::Regex;
use serde_json::Value;

const REDACTION_MARKER: &str = "[REDACTED]";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrubOutcome {
    pub text: String,
    pub count: usize,
}

#[derive(Debug, Clone)]
pub struct Redactor {
    telegram_bot_token: Regex,
    slack_token: Regex,
    generic_api_key: Regex,
    github_pat: Regex,
}

impl Redactor {
    pub fn new() -> Self {
        Self {
            telegram_bot_token: Regex::new(r"\b[0-9]{8,10}:[A-Za-z0-9_-]{35}\b")
                .expect("Telegram token regex must compile"),
            slack_token: Regex::new(r"\bxox[baprs]-[A-Za-z0-9-]{10,}\b")
                .expect("Slack token regex must compile"),
            generic_api_key: Regex::new(r"\b(?:sk|pk)-[A-Za-z0-9]{20,}\b")
                .expect("generic API key regex must compile"),
            github_pat: Regex::new(r"\bghp_[A-Za-z0-9]{36}\b")
                .expect("GitHub PAT regex must compile"),
        }
    }

    pub fn scrub(&self, text: &str) -> ScrubOutcome {
        let mut scrubbed = text.to_owned();
        let mut count = 0;

        for pattern in [
            &self.telegram_bot_token,
            &self.slack_token,
            &self.generic_api_key,
            &self.github_pat,
        ] {
            let matches = pattern.find_iter(&scrubbed).count();
            if matches != 0 {
                scrubbed = pattern
                    .replace_all(&scrubbed, REDACTION_MARKER)
                    .into_owned();
                count += matches;
            }
        }

        ScrubOutcome {
            text: scrubbed,
            count,
        }
    }

    pub fn scrub_json(&self, value: &Value) -> (Value, usize) {
        match value {
            Value::String(text) => {
                let outcome = self.scrub(text);
                (Value::String(outcome.text), outcome.count)
            }
            Value::Array(values) => {
                let mut count = 0;
                let scrubbed = values
                    .iter()
                    .map(|value| {
                        let (value, matches) = self.scrub_json(value);
                        count += matches;
                        value
                    })
                    .collect();
                (Value::Array(scrubbed), count)
            }
            Value::Object(values) => {
                let mut count = 0;
                let scrubbed = values
                    .iter()
                    .map(|(key, value)| {
                        let (value, matches) = self.scrub_json(value);
                        count += matches;
                        (key.clone(), value)
                    })
                    .collect();
                (Value::Object(scrubbed), count)
            }
            _ => (value.clone(), 0),
        }
    }
}

impl Default for Redactor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{REDACTION_MARKER, Redactor};

    const TELEGRAM_TOKEN: &str = "123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghi";

    #[test]
    fn replaces_pasted_telegram_token() {
        let outcome = Redactor::new().scrub(&format!("please use {TELEGRAM_TOKEN} now"));

        assert_eq!(outcome.text, format!("please use {REDACTION_MARKER} now"));
        assert_eq!(outcome.count, 1);
    }

    #[test]
    fn original_token_appears_nowhere_in_scrubbed_output() {
        let outcome = Redactor::new().scrub(&format!("token={TELEGRAM_TOKEN}"));

        assert!(!outcome.text.contains(TELEGRAM_TOKEN));
        assert_eq!(outcome.count, 1);
    }

    #[test]
    fn scrubs_tokens_nested_three_levels_deep_in_object_and_array() {
        let input = json!({
            "level_one": {
                "level_two": {
                    "level_three": format!("object token: {TELEGRAM_TOKEN}")
                }
            },
            "array": [
                {
                    "level_two": [
                        {"level_three": format!("array token: {TELEGRAM_TOKEN}")}
                    ]
                }
            ]
        });

        let (scrubbed, count) = Redactor::new().scrub_json(&input);
        let serialized = serde_json::to_string(&scrubbed).expect("scrubbed JSON must serialize");

        assert_eq!(count, 2);
        assert!(!serialized.contains(TELEGRAM_TOKEN));
        assert_eq!(
            scrubbed["level_one"]["level_two"]["level_three"],
            format!("object token: {REDACTION_MARKER}")
        );
        assert_eq!(
            scrubbed["array"][0]["level_two"][0]["level_three"],
            format!("array token: {REDACTION_MARKER}")
        );
    }

    #[test]
    fn ordinary_text_passes_through_byte_identical() {
        let input = "Leo's email is leo@example.com; docs: https://example.com/a?b=c\nUnicode: 🐈";
        let outcome = Redactor::new().scrub(input);

        assert_eq!(outcome.text.as_bytes(), input.as_bytes());
        assert_eq!(outcome.count, 0);
    }

    #[test]
    fn scrubs_multiple_distinct_credentials_in_one_string() {
        let slack = "xoxb-abcdefghij1234567890";
        let api_key = "sk-ABCDEFGHIJKLMNOPQRSTUVWXYZ123456";
        let github = "ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890";
        let input = format!("{TELEGRAM_TOKEN} {slack} {api_key} {github}");

        let outcome = Redactor::new().scrub(&input);

        assert_eq!(outcome.text, [REDACTION_MARKER; 4].join(" "));
        assert_eq!(outcome.count, 4);
        for credential in [TELEGRAM_TOKEN, slack, api_key, github] {
            assert!(!outcome.text.contains(credential));
        }
    }
}
