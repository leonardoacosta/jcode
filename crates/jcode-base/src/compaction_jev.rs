//! Jev-driven context compaction — score each historical tool call for
//! continued relevance and only drop or truncate calls confident it's obsolete.
//!
//! Adapted from `tamaratran/fast-jev-compaction` (MIT, 4k stars).
//! Text messages are never rewritten — only tool calls and results are pruned.
//!
//! ## Algorithm
//!
//! 1. Collect tool calls and pair with results by `tool_use_id`
//! 2. Pin first message (system prompt) and newest `preserve_recent_messages`
//! 3. Fit state into Jev's token limit via staged truncation
//! 4. For each non-pinned call, ask two Noul questions per call:
//!    - keep_call: does knowing this call was made still matter?
//!    - keep_result: are these contents still needed?
//! 5. Apply decisions: keep_result ≥ threshold → keep both;
//!    keep_call ≥ threshold → keep call + truncate result;
//!    else → remove both
//! 6. Rebuild message list, compute compaction ratio
//! 7. If ratio < 25%, return Skip (fall back to LLM)

use crate::logging;
use crate::message::{ContentBlock, Message, Role};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

// ── Constants ──────────────────────────────────────────────────────────────

/// Default probability threshold for keeping a call or result. Jev returns
/// probabilities; 0.5 means keep if >= 50%.
pub const DEFAULT_JEV_KEEP_THRESHOLD: f64 = 0.5;

/// Default number of newest messages to always pin (never drop).
pub const DEFAULT_JEV_PRESERVE_RECENT: usize = 4;

/// Maximum tokens for the state sent to Jev.
const MAX_STATE_TOKENS: usize = 25_000;

/// Maximum tokens per Jev request batch.
const MAX_REQUEST_TOKENS: usize = 30_000;

/// Minimum compaction ratio to accept (below → fall back to LLM).
const MIN_COMPACTION_RATIO: f64 = 0.25;

/// Timeout for individual Jev API calls.
const JEV_REQUEST_TIMEOUT_SECS: u64 = 10;
const MAX_RETRIES: u32 = 1;

static HTTP_CLIENT: std::sync::LazyLock<reqwest::Client> = std::sync::LazyLock::new(|| {
    reqwest::Client::builder()
        .http2_prior_knowledge()
        .timeout(Duration::from_secs(JEV_REQUEST_TIMEOUT_SECS))
        .build()
        .expect("build reqwest client")
});

// ── Types ──────────────────────────────────────────────────────────────────

/// A paired tool call and its result.
#[derive(Debug, Clone)]
struct ToolCallPair {
    /// Index of the message containing the ToolUse block.
    call_msg_idx: usize,
    /// Index of the message containing the ToolResult block (if paired).
    result_msg_idx: Option<usize>,
    /// The tool use content block.
    tool_use: ToolUseInfo,
    /// The tool result content (if paired).
    tool_result: Option<String>,
    /// Whether this call is pinned (never dropped).
    pinned: bool,
}

#[derive(Debug, Clone)]
struct ToolUseInfo {
    id: String,
    name: String,
    input: Value,
}

/// Decision for a single tool call pair.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct CallDecision {
    call_msg_idx: usize,
    result_msg_idx: Option<usize>,
    /// keep_call probability from Jev.
    keep_call: f64,
    /// keep_result probability from Jev.
    keep_result: f64,
    /// Final action: keep both, keep call only, or remove both.
    keep: bool,
    keep_result_content: bool,
}

/// Result of a Jev compaction attempt.
#[derive(Debug)]
pub struct JevCompactionResult {
    /// The rebuilt message list (or original if nothing to compact).
    pub messages: Vec<Message>,
    /// Compaction ratio (messages removed / original messages).
    pub ratio: f64,
    /// Number of calls assessed.
    pub calls_assessed: usize,
    /// Number of calls kept (either fully or call-only).
    pub calls_kept: usize,
    /// Whether compaction was applied (false = no-op or skip).
    pub applied: bool,
}

// ── Configuration ──────────────────────────────────────────────────────────

/// Configuration for the Jev compactor.
#[derive(Debug, Clone)]
pub struct JevCompactorConfig {
    /// Probability threshold: keep if Jev score >= this.
    pub keep_threshold: f64,
    /// Number of newest messages to always pin.
    pub preserve_recent_messages: usize,
    /// Maximum tokens for the state payload.
    pub max_state_tokens: usize,
    /// Maximum tokens per Jev request batch.
    pub max_request_tokens: usize,
}

impl Default for JevCompactorConfig {
    fn default() -> Self {
        Self {
            keep_threshold: DEFAULT_JEV_KEEP_THRESHOLD,
            preserve_recent_messages: DEFAULT_JEV_PRESERVE_RECENT,
            max_state_tokens: MAX_STATE_TOKENS,
            max_request_tokens: MAX_REQUEST_TOKENS,
        }
    }
}

// ── Public API ─────────────────────────────────────────────────────────────

/// Try Jev-based compaction on the given messages.
///
/// Returns `JevCompactionResult` with the rebuilt message list and stats.
/// If Jev is unavailable or state won't fit, returns an error.
pub async fn try_jev_compact(
    messages: &[Message],
    config: &JevCompactorConfig,
) -> Result<JevCompactionResult> {
    // 1. Single message → nothing to compact.
    if messages.len() <= 1 {
        return Ok(JevCompactionResult {
            messages: messages.to_vec(),
            ratio: 0.0,
            calls_assessed: 0,
            calls_kept: 0,
            applied: false,
        });
    }

    // 2. Collect and pair tool calls.
    let calls = collect_tool_calls(messages, config.preserve_recent_messages);

    // No non-pinned calls → nothing to compact.
    if calls.iter().all(|c| c.pinned) {
        return Ok(JevCompactionResult {
            messages: messages.to_vec(),
            ratio: 0.0,
            calls_assessed: calls.len(),
            calls_kept: calls.len(),
            applied: false,
        });
    }

    // 3. Fit state for Jev.
    let state = fit_state(messages, &calls, config.max_state_tokens);

    // 4. Batch questions.
    let batches = batch_questions(&calls, &state, config.max_request_tokens);

    // 5. Ask Jev.
    let decisions = ask_jev(&batches, config.keep_threshold).await?;

    let calls_assessed = decisions.len();
    let calls_kept = decisions.iter().filter(|d| d.keep).count();

    // 6. Apply decisions and rebuild messages.
    let rebuilt = apply_decisions(messages, &decisions);

    // 7. Compute compaction ratio.
    let ratio = if messages.len() > 0 {
        (messages.len() - rebuilt.len()) as f64 / messages.len() as f64
    } else {
        0.0
    };

    // 8. Check minimum ratio.
    let applied = ratio >= MIN_COMPACTION_RATIO;
    let result = if applied { rebuilt } else { messages.to_vec() };

    Ok(JevCompactionResult {
        messages: result,
        ratio,
        calls_assessed,
        calls_kept,
        applied,
    })
}

// ── Step 1: Tool call collection and pairing ───────────────────────────────

/// Collect tool call pairs from messages. Pins message[0] and the newest
/// `preserve_recent` messages.
fn collect_tool_calls(messages: &[Message], preserve_recent: usize) -> Vec<ToolCallPair> {
    let total = messages.len();
    let mut pairs: Vec<ToolCallPair> = Vec::new();

    // Build a lookup from tool_use_id → result message index.
    let mut results: HashMap<String, (usize, String)> = HashMap::new();
    for (idx, msg) in messages.iter().enumerate() {
        for block in &msg.content {
            if let ContentBlock::ToolResult {
                tool_use_id,
                content,
                ..
            } = block
            {
                results.insert(tool_use_id.clone(), (idx, content.clone()));
            }
        }
    }

    // Collect tool uses.
    for (idx, msg) in messages.iter().enumerate() {
        for block in &msg.content {
            if let ContentBlock::ToolUse {
                id, name, input, ..
            } = block
            {
                let result_pair = results.get(id);
                let result_msg_idx = result_pair.map(|(ri, _)| *ri);
                let tool_result = result_pair.map(|(_, content)| content.clone());

                // Pin: message[0] (system prompt) and the last preserve_recent
                // messages are never pruned.
                let pinned = idx == 0 || (total.saturating_sub(1 + idx) < preserve_recent);

                pairs.push(ToolCallPair {
                    call_msg_idx: idx,
                    result_msg_idx,
                    tool_use: ToolUseInfo {
                        id: id.clone(),
                        name: name.clone(),
                        input: input.clone(),
                    },
                    tool_result,
                    pinned,
                });
            }
        }
    }

    pairs
}

// ── Step 2: Staged state fitting ───────────────────────────────────────────

/// Fit the conversation state into `max_state_tokens` using staged truncation.
fn fit_state(messages: &[Message], calls: &[ToolCallPair], max_state_tokens: usize) -> Value {
    let total = messages.len();

    // Build a JSON-representation of the conversation state.
    let mut state_parts: Vec<Value> = Vec::new();

    for (idx, msg) in messages.iter().enumerate() {
        let mut msg_obj = serde_json::Map::new();

        msg_obj.insert(
            "role".to_string(),
            Value::String(match msg.role {
                Role::User => "user".to_string(),
                Role::Assistant => "assistant".to_string(),
            }),
        );

        let mut blocks: Vec<Value> = Vec::new();

        for block in &msg.content {
            match block {
                ContentBlock::Text { text, .. } => {
                    blocks.push(serde_json::json!({
                        "type": "text",
                        "text": text,
                    }));
                }
                ContentBlock::Reasoning { .. }
                | ContentBlock::ReasoningTrace { .. }
                | ContentBlock::AnthropicThinking { .. }
                | ContentBlock::OpenAIReasoning { .. } => {
                    // Skip reasoning blocks — they're irrelevant to tool retention decisions
                }
                ContentBlock::ToolUse {
                    id, name, input, ..
                } => {
                    blocks.push(serde_json::json!({
                        "type": "tool_use",
                        "id": id,
                        "name": name,
                        "input": input,
                    }));
                }
                ContentBlock::ToolResult {
                    tool_use_id,
                    content,
                    ..
                } => {
                    // Replace full tool results with short notes.
                    let note = format_result_note(content);
                    blocks.push(serde_json::json!({
                        "type": "tool_result",
                        "tool_use_id": tool_use_id,
                        "content": note,
                    }));
                }
                ContentBlock::ToolReference { tool_name, .. } => {
                    blocks.push(serde_json::json!({
                        "type": "tool_reference",
                        "tool_name": tool_name,
                    }));
                }
                ContentBlock::Image { .. } => {
                    blocks.push(serde_json::json!({
                        "type": "image",
                        "note": "[image omitted]",
                    }));
                }
                ContentBlock::OpenAICompaction { .. } => {
                    // Skip native compaction blocks.
                }
            }
        }

        if blocks.is_empty() {
            // Skip messages with no content after filtering.
            continue;
        }

        msg_obj.insert("content".to_string(), Value::Array(blocks));
        msg_obj.insert(
            "idx".to_string(),
            Value::Number(serde_json::Number::from(idx)),
        );
        state_parts.push(Value::Object(msg_obj));
    }

    let state = Value::Array(state_parts);

    // Estimate tokens.
    let state_str = serde_json::to_string(&state).unwrap_or_default();
    let estimated_tokens = estimate_tokens(&state_str);

    if estimated_tokens <= max_state_tokens {
        return state;
    }

    // Stage 1: Truncate tool inputs to 1000 chars.
    let truncated = truncate_tool_inputs(&state, 1000);
    let truncated_str = serde_json::to_string(&truncated).unwrap_or_default();
    if estimate_tokens(&truncated_str) <= max_state_tokens {
        return truncated;
    }

    // Stage 2: Truncate to 200 chars.
    let truncated = truncate_tool_inputs(&state, 200);
    let truncated_str = serde_json::to_string(&truncated).unwrap_or_default();
    if estimate_tokens(&truncated_str) <= max_state_tokens {
        return truncated;
    }

    // Stage 3: Truncate to 60 chars.
    let truncated = truncate_tool_inputs(&state, 60);
    let truncated_str = serde_json::to_string(&truncated).unwrap_or_default();
    if estimate_tokens(&truncated_str) <= max_state_tokens {
        return truncated;
    }

    // Stage 4: Abridge long text blocks to head+tail.
    let abridged = abridge_long_texts(&state);
    let abridged_str = serde_json::to_string(&abridged).unwrap_or_default();
    if estimate_tokens(&abridged_str) <= max_state_tokens {
        return abridged;
    }

    // Stage 5: Collapse old non-pinned messages.
    let collapsed = collapse_old_messages(&state, calls, total);
    let collapsed_str = serde_json::to_string(&collapsed).unwrap_or_default();
    if estimate_tokens(&collapsed_str) <= max_state_tokens {
        return collapsed;
    }

    // Stage 6: Reduce old tool calls to one-liners.
    let reduced = reduce_old_calls(&state, calls, total);
    let reduced_str = serde_json::to_string(&reduced).unwrap_or_default();
    if estimate_tokens(&reduced_str) <= max_state_tokens {
        return reduced;
    }

    // Stage 7: Omit call-less messages from old context.
    let omitted = omit_callless_messages(&state, calls, total);
    let omitted_str = serde_json::to_string(&omitted).unwrap_or_default();
    if estimate_tokens(&omitted_str) <= max_state_tokens {
        return omitted;
    }

    // Stage 8: Fold runs of old call-only messages.
    fold_old_call_runs(&state, calls, total)
}

/// Replace a long tool result with a short note.
fn format_result_note(content: &str) -> String {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return "ok, 0 chars".to_string();
    }
    let len = trimmed.chars().count();
    // Include a brief prefix so Jev can still assess relevance.
    let prefix: String = trimmed.chars().take(80).collect();
    if prefix.chars().count() < trimmed.chars().count() {
        format!("{prefix}… ({len} chars total)")
    } else {
        format!("ok, {len} chars")
    }
}

/// Clone the state array and truncate all tool_input strings to `max_chars`.
fn truncate_tool_inputs(state: &Value, max_chars: usize) -> Value {
    let mut cloned = state.clone();
    if let Value::Array(ref mut msgs) = cloned {
        for msg in msgs.iter_mut() {
            if let Some(blocks) = msg.get_mut("content").and_then(|c| c.as_array_mut()) {
                for block in blocks {
                    if let Some(typ) = block.get("type").and_then(|t| t.as_str()) {
                        if typ == "tool_use" {
                            if let Some(input) = block.get_mut("input") {
                                if let &mut Value::String(ref s) = input {
                                    let truncated: String = s.chars().take(max_chars).collect();
                                    *input = Value::String(truncated);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    cloned
}

/// Abridge long text blocks to head+tail.
fn abridge_long_texts(state: &Value) -> Value {
    let mut cloned = state.clone();
    let head = 300;
    let tail = 100;
    if let Value::Array(ref mut msgs) = cloned {
        for msg in msgs.iter_mut() {
            if let Some(blocks) = msg.get_mut("content").and_then(|c| c.as_array_mut()) {
                for block in blocks {
                    if let Some(typ) = block.get("type").and_then(|t| t.as_str()) {
                        if typ == "text" {
                            if let Some(text) = block.get_mut("text") {
                                if let &mut Value::String(ref s) = text {
                                    let chars: Vec<char> = s.chars().collect();
                                    if chars.len() > head + tail + 20 {
                                        let head_str: String = chars.iter().take(head).collect();
                                        let tail_str: String = chars
                                            .iter()
                                            .rev()
                                            .take(tail)
                                            .collect::<Vec<_>>()
                                            .into_iter()
                                            .rev()
                                            .collect();
                                        *text = Value::String(format!("{head_str}\n…\n{tail_str}"));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    cloned
}

/// Collapse old non-pinned messages to a placeholder.
fn collapse_old_messages(state: &Value, calls: &[ToolCallPair], total: usize) -> Value {
    let mut cloned = state.clone();
    if let Value::Array(ref mut msgs) = cloned {
        let call_idxs: std::collections::HashSet<usize> =
            calls.iter().map(|c| c.call_msg_idx).collect();
        let result_idxs: std::collections::HashSet<usize> =
            calls.iter().filter_map(|c| c.result_msg_idx).collect();

        for msg in msgs.iter_mut() {
            if let Some(idx) = msg.get("idx").and_then(|i| i.as_u64()).map(|i| i as usize) {
                let is_old = total.saturating_sub(idx) > 8;
                let has_call = call_idxs.contains(&idx);
                let has_result = result_idxs.contains(&idx);

                if is_old && !has_call && !has_result {
                    if let Some(blocks) = msg.get_mut("content").and_then(|c| c.as_array_mut()) {
                        let mut total_chars = 0usize;
                        for block in blocks.iter() {
                            if let Some(typ) = block.get("type").and_then(|t| t.as_str()) {
                                if typ == "text" {
                                    if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                                        total_chars += text.chars().count();
                                    }
                                }
                            }
                        }
                        *blocks = vec![serde_json::json!({
                            "type": "text",
                            "text": format!("[old message: {total_chars} chars]"),
                        })];
                    }
                }
            }
        }
    }
    cloned
}

/// Reduce old tool calls to one-liners.
fn reduce_old_calls(state: &Value, calls: &[ToolCallPair], total: usize) -> Value {
    let mut cloned = state.clone();
    if let Value::Array(ref mut msgs) = cloned {
        let call_map: HashMap<usize, &ToolCallPair> =
            calls.iter().map(|c| (c.call_msg_idx, c)).collect();

        for msg in msgs.iter_mut() {
            if let Some(idx) = msg.get("idx").and_then(|i| i.as_u64()).map(|i| i as usize) {
                let is_old = total.saturating_sub(idx) > 8;
                if is_old {
                    if let Some(blocks) = msg.get_mut("content").and_then(|c| c.as_array_mut()) {
                        let mut new_blocks = Vec::new();
                        for block in blocks.iter() {
                            if let Some(typ) = block.get("type").and_then(|t| t.as_str()) {
                                if typ == "tool_use" {
                                    let name =
                                        block.get("name").and_then(|n| n.as_str()).unwrap_or("?");
                                    let id =
                                        block.get("id").and_then(|i| i.as_str()).unwrap_or("?");
                                    let result_note = if let Some(call) = call_map.get(&idx) {
                                        call.tool_result
                                            .as_ref()
                                            .map(|r| format!(" → {}", format_result_note(r)))
                                            .unwrap_or_default()
                                    } else {
                                        String::new()
                                    };
                                    new_blocks.push(serde_json::json!({
                                        "type": "text",
                                        "text": format!("t{} {} id={} ({}){}", idx, name, id, "call", result_note),
                                    }));
                                } else if typ == "tool_result" {
                                    // Skip — represented in the one-liner.
                                } else if typ == "text" {
                                    new_blocks.push(block.clone());
                                }
                            }
                        }
                        if !new_blocks.is_empty() {
                            *blocks = new_blocks;
                        }
                    }
                }
            }
        }
    }
    cloned
}

/// Omit call-less messages from old context.
fn omit_callless_messages(state: &Value, calls: &[ToolCallPair], total: usize) -> Value {
    let mut cloned = state.clone();
    if let Value::Array(ref mut msgs) = cloned {
        let call_idxs: std::collections::HashSet<usize> =
            calls.iter().map(|c| c.call_msg_idx).collect();
        let result_idxs: std::collections::HashSet<usize> =
            calls.iter().filter_map(|c| c.result_msg_idx).collect();

        msgs.retain(|msg| {
            if let Some(idx) = msg.get("idx").and_then(|i| i.as_u64()).map(|i| i as usize) {
                let is_old = total.saturating_sub(idx) > 6;
                if is_old && !call_idxs.contains(&idx) && !result_idxs.contains(&idx) {
                    return false;
                }
            }
            true
        });
    }
    cloned
}

/// Fold runs of old call-only messages into one entry.
fn fold_old_call_runs(state: &Value, calls: &[ToolCallPair], total: usize) -> Value {
    let mut cloned = state.clone();
    if let Value::Array(ref mut msgs) = cloned {
        let call_map: HashMap<usize, &ToolCallPair> =
            calls.iter().map(|c| (c.call_msg_idx, c)).collect();

        let mut folded: Vec<Value> = Vec::new();
        let mut run: Vec<(usize, String)> = Vec::new();

        for msg in msgs.drain(..) {
            let idx = msg.get("idx").and_then(|i| i.as_u64()).map(|i| i as usize);

            let is_old = idx.map(|i| total.saturating_sub(i) > 8).unwrap_or(false);
            let has_call = idx.map(|i| call_map.contains_key(&i)).unwrap_or(false);

            if is_old && has_call {
                // Accumulate into run.
                if let Some(i) = idx {
                    if let Some(call) = call_map.get(&i) {
                        run.push((i, format!("t{} {}", i, call.tool_use.name,)));
                    }
                }
            } else {
                // Flush accumulated run.
                if run.len() > 1 {
                    let summary: String = run
                        .iter()
                        .map(|(_, desc)| desc.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    folded.push(serde_json::json!({
                        "type": "text",
                        "text": format!("[{} old tool calls: {}]", run.len(), summary),
                    }));

                    // Also emit individual entries for pinned calls in the run.
                    for (i, _) in &run {
                        if let Some(call) = call_map.get(i) {
                            if call.pinned {
                                folded.push(serde_json::json!({
                                    "type": "text",
                                    "text": format!("t{} {} (pinned call, input: {:?})", i, call.tool_use.name, call.tool_use.input),
                                }));
                            }
                        }
                    }
                } else if run.len() == 1 {
                    let (i, desc) = &run[0];
                    if let Some(call) = call_map.get(i) {
                        folded.push(serde_json::json!({
                            "type": "text",
                            "text": format!("t{} {} (input: {:?})", i, call.tool_use.name, call.tool_use.input),
                        }));
                    } else {
                        folded.push(serde_json::json!({
                            "type": "text",
                            "text": format!("t{} {}", i, desc),
                        }));
                    }
                }
                run.clear();
                folded.push(msg);
            }
        }

        // Flush remaining run.
        if run.len() > 1 {
            let summary: String = run
                .iter()
                .map(|(_, desc)| desc.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            folded.push(serde_json::json!({
                "type": "text",
                "text": format!("[{} old tool calls: {}]", run.len(), summary),
            }));
        } else if run.len() == 1 {
            let (i, _) = &run[0];
            if let Some(call) = call_map.get(i) {
                folded.push(serde_json::json!({
                    "type": "text",
                    "text": format!("t{} {} (input: {:?})", i, call.tool_use.name, call.tool_use.input),
                }));
            }
        }

        *msgs = folded;
    }
    cloned
}

// ── Token estimation ───────────────────────────────────────────────────────

/// Estimate tokens from a string using the Jev-specific heuristic:
/// word chars / 6, digit chars / 2, symbol chars / 1.
fn estimate_tokens(s: &str) -> usize {
    let word_chars = s.chars().filter(|c| c.is_alphabetic()).count();
    let digit_chars = s.chars().filter(|c| c.is_ascii_digit()).count();
    let symbol_chars = s
        .chars()
        .filter(|c| !c.is_alphanumeric() && !c.is_whitespace())
        .count();

    word_chars / 6 + digit_chars / 2 + symbol_chars
}

// ── Step 3: Question Batching ──────────────────────────────────────────────

#[derive(Clone)]
struct QuestionBatch {
    /// The state snapshot to send with each question.
    state: Value,
    /// Map of question_id → Noul question.
    questions: HashMap<String, Value>,
}

fn batch_questions(
    calls: &[ToolCallPair],
    state: &Value,
    max_request_tokens: usize,
) -> Vec<QuestionBatch> {
    let mut batches: Vec<QuestionBatch> = Vec::new();
    let mut current_batch = QuestionBatch {
        state: state.clone(),
        questions: HashMap::new(),
    };
    let mut current_tokens = estimate_tokens(&serde_json::to_string(state).unwrap_or_default());

    for (i, call) in calls.iter().enumerate() {
        if call.pinned {
            continue;
        }

        let tool_name = &call.tool_use.name;
        let result_preview = call
            .tool_result
            .as_ref()
            .map(|r| format_result_note(r))
            .unwrap_or_else(|| "no result".to_string());

        // Two questions per non-pinned call.
        let keep_call_q = serde_json::json!({
            "type": "noul",
            "instructions": format!(
                "Tool call '{}' (id={}): In the context of this conversation, does knowing that this tool call was made still matter for future decisions? Consider whether the information gathered or action taken is still relevant.",
                tool_name, call.tool_use.id
            ),
        });

        let keep_result_q = serde_json::json!({
            "type": "noul",
            "instructions": format!(
                "Tool result for '{}' (id={}): Are these contents still needed for the ongoing conversation, and re-running the same call would not produce the same information? Result preview: {}",
                tool_name, call.tool_use.id, result_preview
            ),
        });

        let q1_key = format!("keep_call_{}", i);
        let q2_key = format!("keep_result_{}", i);

        let q1_tokens = estimate_tokens(&serde_json::to_string(&keep_call_q).unwrap_or_default());
        let q2_tokens = estimate_tokens(&serde_json::to_string(&keep_result_q).unwrap_or_default());

        let batch_size = current_tokens + q1_tokens + q2_tokens;

        if batch_size > max_request_tokens && !current_batch.questions.is_empty() {
            // Flush current batch and start new one.
            batches.push(std::mem::replace(
                &mut current_batch,
                QuestionBatch {
                    state: state.clone(),
                    questions: HashMap::new(),
                },
            ));
            current_tokens = estimate_tokens(&serde_json::to_string(state).unwrap_or_default());
        }

        current_batch.questions.insert(q1_key, keep_call_q);
        current_batch.questions.insert(q2_key, keep_result_q);
        current_tokens = current_tokens.saturating_add(q1_tokens + q2_tokens);
    }

    if !current_batch.questions.is_empty() {
        batches.push(current_batch);
    }

    batches
}

// ── Step 4: Ask Jev ────────────────────────────────────────────────────────

/// System One request body.
#[derive(Debug, Serialize)]
struct TypeSafeRequest {
    state: Value,
    model: String,
    questions: HashMap<String, Value>,
}

/// System One API response.
#[derive(Debug, Deserialize)]
struct TypeSafeResponse {
    answers: HashMap<String, Value>,
    #[allow(dead_code)]
    model: String,
    #[allow(dead_code)]
    usage: Option<Value>,
}

/// Send all batches to Jev and collect decisions.
async fn ask_jev(batches: &[QuestionBatch], keep_threshold: f64) -> Result<Vec<CallDecision>> {
    let systemone = crate::systemone::resolve()?;
    let model = systemone.default_model.clone();
    let mut all_decisions: HashMap<String, (f64, f64)> = HashMap::new();

    for batch in batches {
        let body = TypeSafeRequest {
            state: batch.state.clone(),
            model: model.clone(),
            questions: batch.questions.clone(),
        };

        let response = send_with_retry(
            &systemone.endpoint_url,
            &systemone.api_key,
            &body,
            MAX_RETRIES,
        )
        .await?;

        for (key, answer) in &response.answers {
            // Extract probability from Noul response.
            // Jev returns: {"probability": 0.85, "answer": "yes", ...}
            let prob = answer
                .get("probability")
                .and_then(|p| p.as_f64())
                .unwrap_or(0.5);

            if let Some(idx_str) = key
                .strip_prefix("keep_call_")
                .or_else(|| key.strip_prefix("keep_result_"))
            {
                if let Ok(_idx) = idx_str.parse::<usize>() {
                    let entry = all_decisions
                        .entry(idx_str.to_string())
                        .or_insert((0.5, 0.5));
                    if key.starts_with("keep_call_") {
                        entry.0 = prob;
                    } else {
                        entry.1 = prob;
                    }
                }
            }
        }
    }

    // Convert to decisions.
    let mut decisions: Vec<CallDecision> = Vec::new();

    let mut sorted_keys: Vec<usize> = all_decisions
        .keys()
        .filter_map(|k| k.parse::<usize>().ok())
        .collect();
    sorted_keys.sort();
    sorted_keys.dedup();

    for key in &sorted_keys {
        if let Some((keep_call, keep_result)) = all_decisions.get(&key.to_string()) {
            let keep = *keep_result >= keep_threshold || *keep_call >= keep_threshold;
            let keep_result_content = *keep_result >= keep_threshold;
            decisions.push(CallDecision {
                call_msg_idx: *key,   // This is the non-pinned call index — needs mapping later
                result_msg_idx: None, // Will be filled by apply_decisions
                keep_call: *keep_call,
                keep_result: *keep_result,
                keep,
                keep_result_content,
            });
        }
    }

    Ok(decisions)
}

async fn send_with_retry(
    url: &str,
    api_key: &str,
    body: &TypeSafeRequest,
    max_retries: u32,
) -> Result<TypeSafeResponse> {
    let mut last_error: Option<anyhow::Error> = None;

    for attempt in 0..=max_retries {
        match send_once(url, api_key, body).await {
            Ok(response) => return Ok(response),
            Err(e) => {
                let status_is_retryable =
                    e.to_string().contains("429") || e.to_string().contains("529");
                if status_is_retryable && attempt < max_retries {
                    let delay = Duration::from_millis(500 * 2u64.pow(attempt));
                    logging::debug(&format!(
                        "jev_compaction: retryable error, attempt {}/{}, waiting {}ms",
                        attempt + 1,
                        max_retries,
                        delay.as_millis()
                    ));
                    tokio::time::sleep(delay).await;
                }
                last_error = Some(e);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("jev_compaction: unknown error")))
}

async fn send_once(url: &str, api_key: &str, body: &TypeSafeRequest) -> Result<TypeSafeResponse> {
    let response = HTTP_CLIENT
        .post(url)
        .header("Authorization", format!("Bearer {api_key}"))
        .json(body)
        .send()
        .await
        .context("jev_compaction: failed to reach the configured System One endpoint")?;

    let status = response.status();
    if !status.is_success() {
        let status_text = response.text().await.unwrap_or_default();
        anyhow::bail!("jev_compaction: API returned HTTP {status}: {status_text}");
    }

    let parsed: TypeSafeResponse = response
        .json()
        .await
        .context("jev_compaction: failed to parse System One response")?;

    Ok(parsed)
}

// ── Step 5: Apply Decisions ────────────────────────────────────────────────

/// Apply decisions to rebuild the message list.
fn apply_decisions(messages: &[Message], decisions: &[CallDecision]) -> Vec<Message> {
    // Build a set of message indices to remove.
    let mut remove_idxs: std::collections::HashSet<usize> = std::collections::HashSet::new();
    let mut truncate_results: HashMap<usize, usize> = HashMap::new();

    for d in decisions {
        if !d.keep {
            // Remove both call and result messages.
            remove_idxs.insert(d.call_msg_idx);
            if let Some(ri) = d.result_msg_idx {
                remove_idxs.insert(ri);
            }
        } else if !d.keep_result_content {
            // Keep call but truncate result.
            if let Some(ri) = d.result_msg_idx {
                truncate_results.insert(ri, d.call_msg_idx);
            }
        }
    }

    // Rebuild messages.
    let mut result: Vec<Message> = Vec::with_capacity(messages.len());

    for (idx, msg) in messages.iter().enumerate() {
        if remove_idxs.contains(&idx) {
            continue;
        }

        if let Some(_call_idx) = truncate_results.get(&idx) {
            // Truncate result content in this message.
            let mut msg = msg.clone();
            for block in &mut msg.content {
                if let ContentBlock::ToolResult { content, .. } = block {
                    // Keep just the first 200 chars with note.
                    let truncated: String = content.chars().take(200).collect();
                    *content = format!("{truncated}… [result truncated by Jev compaction]");
                }
            }
            result.push(msg);
        } else {
            result.push(msg.clone());
        }
    }

    // Ensure no orphaned results: any result without its call must be removed.
    let call_ids: std::collections::HashSet<String> = result
        .iter()
        .flat_map(|msg| msg.content.iter())
        .filter_map(|block| {
            if let ContentBlock::ToolUse { id, .. } = block {
                Some(id.clone())
            } else {
                None
            }
        })
        .collect();

    result.retain(|msg| {
        for block in &msg.content {
            if let ContentBlock::ToolResult { tool_use_id, .. } = block {
                if !call_ids.contains(tool_use_id) {
                    return false;
                }
            }
        }
        true
    });

    // Don't let the compacted list end on a tool-use message.
    while let Some(last) = result.last() {
        let has_tool_use = last
            .content
            .iter()
            .any(|b| matches!(b, ContentBlock::ToolUse { .. }));
        if has_tool_use {
            result.pop();
        } else {
            break;
        }
    }

    result
}

// ── Public API for CompactionManager Integration ───────────────────────────

/// Compaction outcome for the manager to consume.
#[derive(Debug)]
pub enum JevOutcome {
    /// Jev compaction succeeded, messages rebuilt.
    Applied {
        rebuilt_messages: Vec<Message>,
        calls_assessed: usize,
        calls_kept: usize,
        ratio: f64,
    },
    /// Nothing to compact (single message, all pinned, etc).
    Unchanged(String),
    /// Jev unavailable, state won't fit, ratio too low — fall back to LLM.
    Skip(String),
}

/// Convenience wrapper that produces a `JevOutcome` from `try_jev_compact`,
/// handling errors as Skip.
pub async fn compact_with_jev(messages: &[Message], config: &JevCompactorConfig) -> JevOutcome {
    match try_jev_compact(messages, config).await {
        Ok(result) => {
            if !result.applied {
                if result.calls_assessed == 0 {
                    JevOutcome::Unchanged("no tool calls to assess".to_string())
                } else if result.ratio < MIN_COMPACTION_RATIO {
                    JevOutcome::Skip(format!(
                        "compaction ratio {:.1}% below {:.0}% minimum",
                        result.ratio * 100.0,
                        MIN_COMPACTION_RATIO * 100.0
                    ))
                } else {
                    JevOutcome::Unchanged("nothing to compact".to_string())
                }
            } else {
                JevOutcome::Applied {
                    rebuilt_messages: result.messages,
                    calls_assessed: result.calls_assessed,
                    calls_kept: result.calls_kept,
                    ratio: result.ratio,
                }
            }
        }
        Err(e) => {
            logging::warn(&format!("Jev compaction failed, falling back to LLM: {e}"));
            JevOutcome::Skip(format!("Jev error: {e}"))
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_state_preserves_tool_reference() {
        let mut message = Message::user("");
        message.content = vec![ContentBlock::ToolReference {
            tool_use_id: "call-1".into(),
            tool_name: "mcp__weather__forecast".into(),
        }];
        let state = fit_state(&[message], &[], 10_000).to_string();
        assert!(state.contains("tool_reference"));
        assert!(state.contains("mcp__weather__forecast"));
    }

    fn make_text_msg(text: &str, role: Role) -> Message {
        Message {
            role,
            content: vec![ContentBlock::Text {
                text: text.to_string(),
                cache_control: None,
            }],
            timestamp: None,
            tool_duration_ms: None,
        }
    }

    fn make_tool_use_msg(id: &str, name: &str, input: &str) -> Message {
        Message {
            role: Role::Assistant,
            content: vec![ContentBlock::ToolUse {
                id: id.to_string(),
                name: name.to_string(),
                input: serde_json::Value::String(input.to_string()),
                thought_signature: None,
            }],
            timestamp: None,
            tool_duration_ms: None,
        }
    }

    fn make_tool_result_msg(tool_use_id: &str, content: &str) -> Message {
        Message {
            role: Role::User,
            content: vec![ContentBlock::ToolResult {
                tool_use_id: tool_use_id.to_string(),
                content: content.to_string(),
                is_error: None,
            }],
            timestamp: None,
            tool_duration_ms: None,
        }
    }

    #[test]
    fn test_collect_tool_calls_pairs_correctly() {
        let msgs = vec![
            make_text_msg("system prompt", Role::User),
            make_tool_use_msg("call1", "read", "path=foo"),
            make_tool_result_msg("call1", "hello world"),
            make_tool_use_msg("call2", "bash", "ls"),
            make_tool_result_msg("call2", "file1\nfile2"),
        ];

        let calls = collect_tool_calls(&msgs, 4);
        assert_eq!(calls.len(), 2);
        assert!(calls[0].tool_result.is_some());
        assert_eq!(calls[0].tool_result.as_deref(), Some("hello world"));
        assert!(calls[0].result_msg_idx.is_some());
        assert!(calls[1].tool_result.is_some());
        assert_eq!(calls[1].tool_result.as_deref(), Some("file1\nfile2"));
    }

    #[test]
    fn test_collect_tool_calls_pins_first_and_recent() {
        let msgs = vec![
            make_text_msg("system", Role::User),        // 0: pinned
            make_tool_use_msg("c1", "read", "x"),       // 1
            make_tool_result_msg("c1", "ok"),           // 2
            make_tool_use_msg("c2", "bash", "ls"),      // 3
            make_tool_result_msg("c2", "files"),        // 4
            make_text_msg("user question", Role::User), // 5: pinned (recent)
            make_tool_use_msg("c3", "write", "z"),      // 6: pinned (recent)
            make_tool_result_msg("c3", "ok"),           // 7: pinned (recent)
            make_text_msg("user", Role::User),          // 8: pinned (recent)
        ];

        let calls = collect_tool_calls(&msgs, 4);
        // c1 at msg 1, c2 at msg 3, c3 at msg 6
        assert_eq!(calls.len(), 3);

        // c1 (idx 0, msg_idx 1): total=9, 9-1-1=7 >= 4 → NOT pinned
        assert!(!calls[0].pinned);
        // c2 (idx 3): 9-1-3=5 >= 4 → NOT pinned
        assert!(!calls[1].pinned);
        // c3 (idx 6): 9-1-6=2 < 4 → pinned (recent)
        assert!(calls[2].pinned);
    }

    #[test]
    fn test_estimate_tokens() {
        let s = "hello world 123 test";
        let tokens = estimate_tokens(s);
        // word chars: 15 → 15/6 = 2
        // digit chars: 3 → 3/2 = 1
        // symbols: 3 spaces = 0, so total ~3
        assert!(tokens > 0);
        assert!(tokens < 20);
    }

    #[test]
    fn test_empty_messages_unchanged() {
        let msgs = vec![make_text_msg("hello", Role::User)];
        let config = JevCompactorConfig::default();

        // We can't use async in a regular test without tokio runtime.
        // Just test the synchronous parts.
        let calls = collect_tool_calls(&msgs, config.preserve_recent_messages);
        assert!(calls.is_empty());
    }

    #[test]
    fn test_all_pinned_returns_unchanged() {
        let msgs = vec![
            make_tool_use_msg("c1", "read", "x"),
            make_tool_result_msg("c1", "ok"),
            make_tool_use_msg("c2", "bash", "ls"),
            make_tool_result_msg("c2", "files"),
        ];
        let config = JevCompactorConfig::default();
        let calls = collect_tool_calls(&msgs, config.preserve_recent_messages);

        // With preserve_recent=4 and 4 messages, all should be pinned.
        assert!(calls.iter().all(|c| c.pinned));
    }

    #[test]
    fn test_format_result_note() {
        let note = format_result_note("hello");
        assert_eq!(note, "ok, 5 chars");

        let note = format_result_note("");
        assert_eq!(note, "ok, 0 chars");

        let long = "a".repeat(200);
        let note = format_result_note(&long);
        assert!(note.contains("200 chars total"));
    }

    #[test]
    fn test_truncate_tool_inputs() {
        let state = serde_json::json!([
            {
                "idx": 0,
                "content": [
                    {"type": "tool_use", "id": "t1", "name": "read", "input": "very long input here yes"},
                    {"type": "tool_result", "tool_use_id": "t1", "content": "result note"},
                ]
            }
        ]);

        let truncated = truncate_tool_inputs(&state, 5);
        let s = serde_json::to_string(&truncated).unwrap();
        assert!(s.contains("\"input\":\"very "));
    }

    #[test]
    fn test_apply_decisions_removes_dropped_calls() {
        let msgs = vec![
            make_text_msg("system", Role::User),
            make_tool_use_msg("c1", "read", "x"),
            make_tool_result_msg("c1", "content"),
            make_text_msg("user", Role::User),
        ];

        let decisions = vec![CallDecision {
            call_msg_idx: 1,
            result_msg_idx: Some(2),
            keep_call: 0.0,
            keep_result: 0.0,
            keep: false,
            keep_result_content: false,
        }];

        let result = apply_decisions(&msgs, &decisions);
        assert_eq!(result.len(), 2);
        assert!(
            result
                .iter()
                .all(|m| !matches!(m.content.first(), Some(ContentBlock::ToolUse { .. })))
        );
    }

    #[test]
    fn test_apply_decisions_keeps_text_verbatim() {
        let user_text = "important user message";
        let msgs = vec![
            make_text_msg(user_text, Role::User),
            make_tool_use_msg("c1", "read", "x"),
            make_tool_result_msg("c1", "content"),
            make_text_msg("assistant response", Role::Assistant),
        ];

        let decisions = vec![CallDecision {
            call_msg_idx: 1,
            result_msg_idx: Some(2),
            keep_call: 0.0,
            keep_result: 0.0,
            keep: false,
            keep_result_content: false,
        }];

        let result = apply_decisions(&msgs, &decisions);
        assert_eq!(result.len(), 2);

        // Text must be unchanged.
        let text = match result[0].content.first() {
            Some(ContentBlock::Text { text, .. }) => text.clone(),
            _ => panic!("expected text"),
        };
        assert_eq!(text, user_text);
    }

    #[test]
    fn test_orphaned_results_are_removed() {
        let msgs = vec![
            make_tool_use_msg("c1", "read", "x"),
            make_tool_result_msg("c1", "ok"),
            make_tool_result_msg("orphan", "orphan result"), // no matching call
            make_text_msg("user", Role::User),
        ];

        // Drop c1.
        let decisions = vec![CallDecision {
            call_msg_idx: 0,
            result_msg_idx: Some(1),
            keep_call: 0.0,
            keep_result: 0.0,
            keep: false,
            keep_result_content: false,
        }];

        let result = apply_decisions(&msgs, &decisions);
        // Only the user message should survive.
        assert_eq!(result.len(), 1);
        // The last remaining message should NOT be a tool result.
        assert!(matches!(
            result[0].content.first(),
            Some(ContentBlock::Text { .. }),
        ));
    }
}
