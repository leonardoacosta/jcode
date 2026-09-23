use super::{Tool, ToolContext, ToolOutput};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::collections::HashMap;

/// Maximum questions per request.
const MAX_QUESTIONS: usize = 4;
/// Minimum questions per request.
const MIN_QUESTIONS: usize = 1;
/// Maximum options per question.
const MAX_OPTIONS: usize = 4;
/// Minimum options per question.
const MIN_OPTIONS: usize = 2;
/// Maximum total payload size in bytes (conservative for prompt overhead).
const MAX_PAYLOAD_BYTES: usize = 65_536;
/// String bounds for text fields.
const MAX_QUESTION_CHARS: usize = 1_000;
const MAX_HEADER_CHARS: usize = 12;
const MAX_LABEL_CHARS: usize = 100;
const MAX_DESCRIPTION_CHARS: usize = 500;
const MAX_ANSWER_CHARS: usize = 4_000;

/// A single structured question to present to the user.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Question {
    /// Stable unique id for this question (set by model, reused in answers).
    pub id: String,
    /// The complete question text.
    pub question: String,
    /// Short label displayed as a chip (max 12 chars).
    pub header: String,
    /// The available choices.
    pub options: Vec<QuestionOption>,
    /// Whether multiple selections are allowed (default false).
    #[serde(default)]
    pub multi_select: bool,
}

/// A single option within a question.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QuestionOption {
    /// Stable unique id for this option within the question.
    pub id: String,
    /// Display label (1-5 words).
    pub label: String,
    /// Description explaining what this option means.
    pub description: String,
}

/// User's answer to a single question.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QuestionAnswer {
    /// Selected option ids.
    pub option_ids: Vec<String>,
    /// Optional free-form text entered by the user.
    #[serde(default)]
    pub free_text: Option<String>,
}

fn validate_questions(questions: &[Question]) -> Result<()> {
    // Count bounds
    if questions.is_empty() {
        anyhow::bail!("at least {MIN_QUESTIONS} question required");
    }
    if questions.len() > MAX_QUESTIONS {
        anyhow::bail!("at most {MAX_QUESTIONS} questions allowed, got {}", questions.len());
    }

    let mut seen_ids: HashMap<&str, usize> = HashMap::new();
    for q in questions {
        // Unique question id
        if let Some(&prev_idx) = seen_ids.get(q.id.as_str()) {
            anyhow::bail!("duplicate question id '{}' at index {} (first seen at {})", q.id, seen_ids.len(), prev_idx);
        }
        seen_ids.insert(&q.id, seen_ids.len());

        // String bounds
        if q.question.is_empty() || q.question.len() > MAX_QUESTION_CHARS {
            anyhow::bail!("question text must be 1..{MAX_QUESTION_CHARS} chars");
        }
        if q.header.is_empty() || q.header.len() > MAX_HEADER_CHARS {
            anyhow::bail!("question header must be 1..{MAX_HEADER_CHARS} chars");
        }

        // Option count bounds
        if q.options.len() < MIN_OPTIONS {
            anyhow::bail!("question '{}' needs at least {MIN_OPTIONS} options", q.id);
        }
        if q.options.len() > MAX_OPTIONS {
            anyhow::bail!("question '{}' has too many options (max {MAX_OPTIONS})", q.id);
        }

        // Validate options
        let mut seen_opt_ids: HashMap<&str, usize> = HashMap::new();
        for opt in &q.options {
            if let Some(&prev_idx) = seen_opt_ids.get(opt.id.as_str()) {
                anyhow::bail!("duplicate option id '{}' in question '{}' (first seen at index {})", opt.id, q.id, prev_idx);
            }
            seen_opt_ids.insert(&opt.id, seen_opt_ids.len());
            if opt.label.is_empty() || opt.label.len() > MAX_LABEL_CHARS {
                anyhow::bail!("option label must be 1..{MAX_LABEL_CHARS} chars, question '{}' option '{}'", q.id, opt.id);
            }
            if opt.description.len() > MAX_DESCRIPTION_CHARS {
                anyhow::bail!("option description too long (max {MAX_DESCRIPTION_CHARS} chars), question '{}' option '{}'", q.id, opt.id);
            }
        }
    }

    Ok(())
}

pub struct AskUserQuestionTool;

impl AskUserQuestionTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct AskUserQuestionInput {
    /// 1-4 questions to present.
    questions: Vec<Question>,
}

#[async_trait]
impl Tool for AskUserQuestionTool {
    fn name(&self) -> &str {
        "ask_user_question"
    }

    fn description(&self) -> &str {
        "Ask the user one to four structured questions. Use only for choices you cannot resolve from context or defaults. Each question presents options; the user may select options and provide free text."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["questions"],
            "properties": {
                "intent": super::intent_schema_property(),
                "questions": {
                    "type": "array",
                    "minItems": MIN_QUESTIONS,
                    "maxItems": MAX_QUESTIONS,
                    "description": "Questions to present (1-4).",
                    "items": {
                        "type": "object",
                        "required": ["id", "question", "header", "options"],
                        "properties": {
                            "id": {
                                "type": "string",
                                "description": "Stable unique id for this question. Reused in answers."
                            },
                            "question": {
                                "type": "string",
                                "description": "The complete question text (max 1000 chars)."
                            },
                            "header": {
                                "type": "string",
                                "description": "Short label displayed as a chip (max 12 chars)."
                            },
                            "options": {
                                "type": "array",
                                "minItems": MIN_OPTIONS,
                                "maxItems": MAX_OPTIONS,
                                "description": "Available choices (2-4).",
                                "items": {
                                    "type": "object",
                                    "required": ["id", "label", "description"],
                                    "properties": {
                                        "id": {
                                            "type": "string",
                                            "description": "Stable unique id for this option within the question."
                                        },
                                        "label": {
                                            "type": "string",
                                            "description": "Display text for this option (1-5 words, max 100 chars)."
                                        },
                                        "description": {
                                            "type": "string",
                                            "description": "Explanation of what this option means (max 500 chars)."
                                        }
                                    }
                                }
                            },
                            "multi_select": {
                                "type": "boolean",
                                "description": "Allow multiple selections (default false)."
                            }
                        }
                    }
                }
            }
        })
    }

    async fn execute(&self, input: Value, ctx: ToolContext) -> Result<ToolOutput> {
        let params: AskUserQuestionInput = serde_json::from_value(input)
            .map_err(|e| anyhow::anyhow!("invalid ask_user_question input: {e}"))?;

        // Validate payload bounds
        let payload_bytes = serde_json::to_vec(&params.questions)
            .unwrap_or_default()
            .len();
        if payload_bytes > MAX_PAYLOAD_BYTES {
            anyhow::bail!("question payload too large ({payload_bytes} bytes, max {MAX_PAYLOAD_BYTES})");
        }

        validate_questions(&params.questions)?;

        // Block until user answers via the TUI protocol path.
        // The stdin_request_tx path is for shell stdin, not this structured path.
        // For now, when no client is listening, return unavailable.
        anyhow::bail!("ask_user_question requires an interactive client; no answer channel available in this session. The tool is available only for root interactive sessions with a capable TUI client.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_question_valid() {
        let qs = vec![Question {
            id: "q1".into(),
            question: "Which template?".into(),
            header: "Template".into(),
            options: vec![
                QuestionOption { id: "a".into(), label: "Next.js".into(), description: "App Router".into() },
                QuestionOption { id: "b".into(), label: "Remix".into(), description: "Full stack".into() },
            ],
            multi_select: false,
        }];
        assert!(validate_questions(&qs).is_ok());
    }

    #[test]
    fn test_duplicate_question_id_rejected() {
        let qs = vec![
            Question {
                id: "q1".into(), question: "A".into(), header: "H".into(),
                options: vec![
                    QuestionOption { id: "a".into(), label: "X".into(), description: "d".into() },
                    QuestionOption { id: "b".into(), label: "Y".into(), description: "d".into() },
                ],
                multi_select: false,
            },
            Question {
                id: "q1".into(), question: "B".into(), header: "H2".into(),
                options: vec![
                    QuestionOption { id: "c".into(), label: "X".into(), description: "d".into() },
                    QuestionOption { id: "d".into(), label: "Y".into(), description: "d".into() },
                ],
                multi_select: false,
            },
        ];
        assert!(validate_questions(&qs).is_err());
    }

    #[test]
    fn test_duplicate_option_id_rejected() {
        let qs = vec![Question {
            id: "q1".into(), question: "A".into(), header: "H".into(),
            options: vec![
                QuestionOption { id: "dup".into(), label: "X".into(), description: "d".into() },
                QuestionOption { id: "dup".into(), label: "Y".into(), description: "d".into() },
            ],
            multi_select: false,
        }];
        assert!(validate_questions(&qs).is_err());
    }

    #[test]
    fn test_too_few_options_rejected() {
        let qs = vec![Question {
            id: "q1".into(), question: "A".into(), header: "H".into(),
            options: vec![
                QuestionOption { id: "a".into(), label: "X".into(), description: "d".into() },
            ],
            multi_select: false,
        }];
        assert!(validate_questions(&qs).is_err());
    }

    #[test]
    fn test_too_many_questions_rejected() {
        let qs: Vec<Question> = (0..MAX_QUESTIONS+1).map(|i| Question {
            id: format!("q{i}"), question: "A".into(), header: "H".into(),
            options: vec![
                QuestionOption { id: "a".into(), label: "X".into(), description: "d".into() },
                QuestionOption { id: "b".into(), label: "Y".into(), description: "d".into() },
            ],
            multi_select: false,
        }).collect();
        assert!(validate_questions(&qs).is_err());
    }

    #[test]
    fn test_empty_header_rejected() {
        let qs = vec![Question {
            id: "q1".into(), question: "A".into(), header: "".into(),
            options: vec![
                QuestionOption { id: "a".into(), label: "X".into(), description: "d".into() },
                QuestionOption { id: "b".into(), label: "Y".into(), description: "d".into() },
            ],
            multi_select: false,
        }];
        assert!(validate_questions(&qs).is_err());
    }

    #[test]
    fn test_multi_select_default_false() {
        let qs = vec![Question {
            id: "q1".into(), question: "A".into(), header: "H".into(),
            options: vec![
                QuestionOption { id: "a".into(), label: "X".into(), description: "d".into() },
                QuestionOption { id: "b".into(), label: "Y".into(), description: "d".into() },
            ],
            multi_select: false,
        }];
        assert!(validate_questions(&qs).is_ok());
    }
}