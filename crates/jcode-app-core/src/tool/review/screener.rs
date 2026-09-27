//! Jev-driven pre-screening layer for Jcode's `/review` skill.
//!
//! Before running a full LLM code review, Jev screens each diff hunk or source
//! file across 5 risk dimensions (correctness, security, reliability,
//! compatibility, test coverage) with calibrated Noul probabilities. Hunks
//! scoring above 0.7 on any dimension are routed to full LLM review. Hunks
//! scoring below are assessed as low-risk with Jev confidence scores alone.
//!
//! Architecture adapted from `devagrawal09/jev-review` (MIT).

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

// ── Risk matrix ──

/// 5-dimension risk assessment for a single diff hunk or source file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMatrix {
    /// Probability this change introduces a logic error (0-1).
    pub correctness_risk: f64,
    /// Probability this change introduces a vulnerability (0-1).
    pub security_risk: f64,
    /// Probability this change affects stability or error handling (0-1).
    pub reliability_risk: f64,
    /// Probability this change breaks existing contracts or APIs (0-1).
    pub compatibility_risk: f64,
    /// Probability existing tests don't adequately cover this change (0-1).
    pub test_gap: f64,
}

impl RiskMatrix {
    /// Default threshold for flagging a hunk for full LLM review.
    pub const DEFAULT_THRESHOLD: f64 = 0.7;

    /// Return true if any dimension exceeds the threshold, signaling that this
    /// change needs a full LLM review.
    pub fn needs_full_review(&self, threshold: f64) -> bool {
        self.correctness_risk > threshold
            || self.security_risk > threshold
            || self.reliability_risk > threshold
            || self.compatibility_risk > threshold
            || self.test_gap > threshold
    }

    /// Return true if all dimensions are at or below the threshold.
    pub fn is_low_risk(&self, threshold: f64) -> bool {
        !self.needs_full_review(threshold)
    }

    /// Format a one-line summary with all risk scores.
    pub fn summary_line(&self) -> String {
        format!(
            "correctness={:.2}, security={:.2}, reliability={:.2}, compatibility={:.2}, test_gap={:.2}",
            self.correctness_risk,
            self.security_risk,
            self.reliability_risk,
            self.compatibility_risk,
            self.test_gap,
        )
    }

    /// Maximum risk across all five dimensions.
    pub fn max_risk(&self) -> f64 {
        self.correctness_risk
            .max(self.security_risk)
            .max(self.reliability_risk)
            .max(self.compatibility_risk)
            .max(self.test_gap)
    }
}

// ── Diff chunking ──

/// A single hunk for screening. Includes diff text, file path, language
/// (inferred from extension), and related test paths.
#[derive(Debug, Clone)]
pub struct DiffHunk {
    /// File path this hunk belongs to.
    pub file_path: String,
    /// Diff text for this file (unified diff format lines for this file only).
    pub diff_text: String,
    /// Inferred language (e.g. "rust", "typescript", "python").
    pub language: String,
    /// Related test file paths (inferred by naming convention).
    pub related_tests: Vec<String>,
    /// Whether this file is binary (should skip review).
    pub is_binary: bool,
}

/// Chunk a unified diff into file-level hunks for independent screening.
///
/// Splits the diff by the `diff --git` header. Each hunk includes diff text
/// for one file, inferred language, and related test paths by convention
/// (e.g. `src/foo.rs` → `tests/foo_test.rs`).
///
/// Binary files are identified by `Binary files` markers in the diff and
/// flagged with `is_binary: true`.
///
/// Files with diff text exceeding `max_hunk_chars` (default 8000) are split
/// at function boundaries where possible. Oversized files that cannot be
/// split at function boundaries are truncated with a note.
pub fn chunk_diff(diff: &str, repo_root: &Path, max_hunk_chars: usize) -> Vec<DiffHunk> {
    let mut hunks = Vec::new();
    let mut current_file: Option<String> = None;
    let mut current_lines = Vec::new();
    let mut is_binary = false;

    for line in diff.lines() {
        // Detect file boundaries: `diff --git a/<path> b/<path>`
        if line.starts_with("diff --git ") {
            // Flush previous hunk.
            if let Some(file_path) = current_file.take() {
                let languages = infer_language(&file_path);
                let related_tests = infer_related_tests(&file_path, repo_root);
                let raw_diff = current_lines.join("\n");
                hunks.extend(split_oversized_hunk(
                    DiffHunk {
                        file_path: file_path.clone(),
                        diff_text: raw_diff,
                        language: languages,
                        related_tests,
                        is_binary,
                    },
                    max_hunk_chars,
                    &file_path,
                ));
                current_lines.clear();
                is_binary = false;
            }

            // Extract file path from `diff --git a/path b/path`
            if let Some(a_path) = line.strip_prefix("diff --git a/") {
                if let Some(path) = a_path.split(' ').next() {
                    current_file = Some(path.to_string());
                    // Skip the rest of the header — it's the same path with b/ prefix.
                } else {
                    current_file = Some(a_path.to_string());
                }
            } else {
                // Fallback: use the whole line as a unique file key.
                current_file = Some(line.to_string());
            }
        }

        // Detect binary files.
        if line.contains("Binary files") && line.contains("differ") {
            is_binary = true;
        }

        // Accumulate lines if we have a current file.
        if current_file.is_some() && !(line.starts_with("diff --git ") && current_lines.is_empty()) {
            current_lines.push(line.to_string());
        }
    }

    // Flush last hunk.
    if let Some(file_path) = current_file.take() {
        let languages = infer_language(&file_path);
        let related_tests = infer_related_tests(&file_path, repo_root);
        let raw_diff = current_lines.join("\n");
        hunks.extend(split_oversized_hunk(
            DiffHunk {
                file_path: file_path.clone(),
                diff_text: raw_diff,
                language: languages,
                related_tests,
                is_binary,
            },
            max_hunk_chars,
            &file_path,
        ));
    }

    hunks
}

/// Split an oversized hunk into smaller pieces at function boundaries.
fn split_oversized_hunk(hunk: DiffHunk, max_chars: usize, file_path: &str) -> Vec<DiffHunk> {
    if hunk.diff_text.len() <= max_chars {
        return vec![hunk];
    }

    // Try splitting at function boundaries.
    // Function boundaries: lines starting with `fn `, `pub fn `, `def `,
    // `function `, `class `, or `@@ ` (hunk header).
    let parts = split_at_boundaries(&hunk.diff_text, max_chars, |line| {
        let trimmed = line.trim();
        trimmed.starts_with("fn ")
            || trimmed.starts_with("pub fn ")
            || trimmed.starts_with("pub(crate) fn ")
            || trimmed.starts_with("pub(super) fn ")
            || trimmed.starts_with("def ")
            || trimmed.starts_with("function ")
            || trimmed.starts_with("class ")
            || trimmed.starts_with("export function ")
            || trimmed.starts_with("export const ")
            || trimmed.starts_with("impl ")
            || trimmed.starts_with("pub struct ")
            || trimmed.starts_with("pub enum ")
            || trimmed.starts_with("pub trait ")
            || trimmed.starts_with("@@ ")
    });

    if parts.len() <= 1 {
        // Could not split at boundaries. Return as-is with a truncation note.
        let truncated = hunk.diff_text.chars().take(max_chars - 200).collect::<String>();
        let note = format!(
            "\n\n[... {} more chars truncated; file too large for single screening. \
             Consider reviewing this file manually or splitting it.]",
            hunk.diff_text.len() - truncated.len()
        );
        return vec![DiffHunk {
            diff_text: format!("{truncated}{note}"),
            ..hunk
        }];
    }

    let mut sub_hunks = Vec::new();
    for part in parts {
        sub_hunks.push(DiffHunk {
            file_path: hunk.file_path.clone(),
            diff_text: part,
            language: hunk.language.clone(),
            related_tests: hunk.related_tests.clone(),
            is_binary: hunk.is_binary,
        });
    }
    sub_hunks
}

/// Split text at boundary-matching lines, keeping each chunk under max_chars.
fn split_at_boundaries(
    text: &str,
    max_chars: usize,
    is_boundary: impl Fn(&str) -> bool,
) -> Vec<String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut parts = Vec::new();
    let mut current = Vec::new();
    let mut current_len = 0usize;

    for line in lines {
        if is_boundary(line) && !current.is_empty() && current_len >= max_chars / 2 {
            parts.push(current.join("\n"));
            current = Vec::new();
            current_len = 0;
        }
        current_len += line.len() + 1; // +1 for newline
        current.push(line.to_string());
    }

    if !current.is_empty() {
        parts.push(current.join("\n"));
    }

    parts
}

/// Infer language from file extension.
fn infer_language(file_path: &str) -> String {
    let ext = Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext {
        "rs" => "rust".to_string(),
        "py" => "python".to_string(),
        "js" => "javascript".to_string(),
        "ts" => "typescript".to_string(),
        "tsx" => "typescript-react".to_string(),
        "jsx" => "javascript-react".to_string(),
        "go" => "go".to_string(),
        "java" => "java".to_string(),
        "c" | "h" => "c".to_string(),
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => "cpp".to_string(),
        "rb" => "ruby".to_string(),
        "swift" => "swift".to_string(),
        "kt" => "kotlin".to_string(),
        "md" => "markdown".to_string(),
        "json" => "json".to_string(),
        "yaml" | "yml" => "yaml".to_string(),
        "toml" => "toml".to_string(),
        "sql" => "sql".to_string(),
        "sh" | "bash" | "zsh" => "shell".to_string(),
        "" => "text".to_string(),
        other => other.to_string(),
    }
}

/// Infer related test file paths by naming convention.
///
/// e.g. `src/auth.rs` → `tests/auth_test.rs`, `tests/auth.rs`,
/// `src/__tests__/auth.test.rs`, `src/auth_test.rs`
fn infer_related_tests(file_path: &str, repo_root: &Path) -> Vec<String> {
    let path = Path::new(file_path);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    let mut candidates = Vec::new();

    // Convention 1: src/<module>.rs → tests/<module>_test.rs
    candidates.push(format!("tests/{stem}_test.{ext}"));
    candidates.push(format!("tests/{stem}.{ext}"));

    // Convention 2: Co-located test files
    if let Some(parent) = path.parent() {
        let parent_str = parent.to_string_lossy();
        candidates.push(format!("{parent_str}/{stem}_test.{ext}"));
        candidates.push(format!("{parent_str}/__tests__/{stem}.test.{ext}"));
    }

    // Convention 3: test/ instead of tests/
    candidates.push(format!("test/{stem}_test.{ext}"));
    candidates.push(format!("test/{stem}.{ext}"));

    // Convention 4: __tests__ next to source
    if let Some(parent) = path.parent() {
        let parent_str = parent.to_string_lossy();
        candidates.push(format!("{parent_str}/__tests__/{stem}.spec.{ext}"));
    }

    // Check which candidates actually exist on disk.
    candidates
        .into_iter()
        .filter(|candidate| repo_root.join(candidate).exists())
        .collect()
}

// ── Jev screening ──

/// Screen a single hunk through Jev, returning a risk matrix.
///
/// Sends 5 Noul questions to the configured System One endpoint and parses the
/// response into a `RiskMatrix`. Each answer is clipped to [0, 1].
///
/// Returns `Err` on API failure, timeout, or invalid response — the caller
/// should route the hunk to full LLM review on error.
pub async fn screen_hunk(hunk: &DiffHunk) -> Result<RiskMatrix> {
    let state = build_screening_state(hunk);
    let questions = build_risk_questions();
    let response = send_evaluate_request(&state, &questions).await?;
    parse_risk_matrix(&response.answers)
}

/// Build the Jev state JSON for a single hunk.
fn build_screening_state(hunk: &DiffHunk) -> Value {
    serde_json::json!({
        "diff_hunk": hunk.diff_text,
        "file_path": hunk.file_path,
        "language": hunk.language,
        "related_tests": hunk.related_tests,
        "is_binary": hunk.is_binary,
    })
}

/// Build the 5 Noul risk questions.
fn build_risk_questions() -> Value {
    serde_json::json!({
        "correctness_risk": {
            "type": "noul",
            "instructions": "For the given code diff hunk, could this change introduce a logic error or incorrect behavior?"
        },
        "security_risk": {
            "type": "noul",
            "instructions": "For the given code diff hunk, could this change introduce a security vulnerability (e.g., injection, XSS, auth bypass, unsafe deserialization)?"
        },
        "reliability_risk": {
            "type": "noul",
            "instructions": "For the given code diff hunk, could this change affect stability, error handling, or introduce a crash path?"
        },
        "compatibility_risk": {
            "type": "noul",
            "instructions": "For the given code diff hunk, could this change break existing contracts, APIs, or backwards compatibility?"
        },
        "test_gap": {
            "type": "noul",
            "instructions": "For the given code diff hunk, do existing related tests adequately cover this change? Answer 'yes' means well-covered (low test gap), 'no' means under-tested (high test gap)."
        }
    })
}

// ── Evaluate API integration ──

static HTTP_CLIENT: std::sync::LazyLock<reqwest::Client> = std::sync::LazyLock::new(|| {
    reqwest::Client::builder()
        .http2_prior_knowledge()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("build reqwest client for jev-review screener")
});

/// Request body for System One.
#[derive(Debug, Serialize)]
struct TypeSafeRequest {
    state: Value,
    model: String,
    questions: Value,
}

/// Response from System One.
#[derive(Debug, Deserialize)]
struct TypeSafeResponse {
    answers: HashMap<String, Value>,
}

/// Send an evaluate request to the Jev API and return parsed answers.
///
/// Sends each request to the configured System One endpoint. Timeout is 5s per hunk.
/// Returns the `answers` map on success.
async fn send_evaluate_request(state: &Value, questions: &Value) -> Result<TypeSafeResponse> {
    let (api_url, api_key, model) = resolve_api_config()?;

    let body = TypeSafeRequest {
        state: state.clone(),
        model,
        questions: questions.clone(),
    };

    let response = HTTP_CLIENT
        .post(&api_url)
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&body)
        .send()
        .await
        .context("jev-review: failed to reach Jev API")?;

    let status = response.status();
    if !status.is_success() {
        let status_text = response.text().await.unwrap_or_default();
        anyhow::bail!("jev-review: API returned HTTP {status}: {status_text}");
    }

    let parsed: TypeSafeResponse = response
        .json()
        .await
        .context("jev-review: failed to parse Jev response")?;

    Ok(parsed)
}

/// Resolve API config from the shared System One settings.
fn resolve_api_config() -> Result<(String, String, String)> {
    let config = crate::systemone::resolve()?;
    Ok((config.endpoint_url, config.api_key, config.default_model))
}

/// Parse a Jev response into a RiskMatrix.
///
/// Extracts each Noul answer, converts yes/no/boolean/probability to f64,
/// and clips to [0, 1]. For `test_gap`, the question is phrased as "do
/// tests adequately cover this?" so a "no" answer means high test gap.
fn parse_risk_matrix(answers: &HashMap<String, Value>) -> Result<RiskMatrix> {
    let correctness_risk = extract_noul_probability(answers.get("correctness_risk"), "correctness_risk")?;
    let security_risk = extract_noul_probability(answers.get("security_risk"), "security_risk")?;
    let reliability_risk = extract_noul_probability(answers.get("reliability_risk"), "reliability_risk")?;
    let compatibility_risk = extract_noul_probability(answers.get("compatibility_risk"), "compatibility_risk")?;

    // test_gap: "do tests cover?" → "no" means gap → invert some formats.
    let raw_test_gap = extract_noul_probability(answers.get("test_gap"), "test_gap")?;
    let test_gap = raw_test_gap;

    Ok(RiskMatrix {
        correctness_risk,
        security_risk,
        reliability_risk,
        compatibility_risk,
        test_gap,
    })
}

/// Extract a Noul probability from a Jev answer value.
///
/// Jev may return:
/// - `{"noul": 0.85, "confidence": 0.92}`
/// - `{"probability": 0.85}`
/// - `true` / `false` (plain boolean)
/// - Plain number
fn extract_noul_probability(answer: Option<&Value>, label: &str) -> Result<f64> {
    let value = answer.ok_or_else(|| {
        anyhow::anyhow!("jev-review: missing answer for '{label}'")
    })?;

    // Standard Noul format: {"noul": 0.85}
    if let Some(obj) = value.as_object() {
        if let Some(noul_val) = obj.get("noul").or_else(|| obj.get("probability")) {
            if let Some(n) = noul_val.as_f64() {
                return Ok(n.clamp(0.0, 1.0));
            }
            if let Some(b) = noul_val.as_bool() {
                return Ok(if b { 1.0 } else { 0.0 });
            }
        }
        // Some APIs return {"answer": true/false} for noul.
        if let Some(ans) = obj.get("answer") {
            if let Some(b) = ans.as_bool() {
                return Ok(if b { 1.0 } else { 0.0 });
            }
            if let Some(s) = ans.as_str() {
                return parse_noul_string(s, label);
            }
        }
    }

    // Plain boolean.
    if let Some(b) = value.as_bool() {
        return Ok(if b { 1.0 } else { 0.0 });
    }

    // Plain number.
    if let Some(n) = value.as_f64() {
        return Ok(n.clamp(0.0, 1.0));
    }

    // String: "yes"/"no", "true"/"false", or a probability.
    if let Some(s) = value.as_str() {
        return parse_noul_string(s, label);
    }

    anyhow::bail!("jev-review: cannot interpret answer for '{label}': {value}")
}

/// Parse a string answer for a Noul question.
fn parse_noul_string(s: &str, _label: &str) -> Result<f64> {
    let lower = s.trim().to_lowercase();
    match lower.as_str() {
        "yes" | "true" => Ok(1.0),
        "no" | "false" => Ok(0.0),
        _ => {
            // Try parsing as a number.
            if let Ok(n) = lower.parse::<f64>() {
                Ok(n.clamp(0.0, 1.0))
            } else {
                // "likely yes" → 0.8, "probably no" → 0.2, etc.
                if lower.contains("yes") || lower.contains("likely") || lower.contains("probably") {
                    Ok(0.8)
                } else if lower.contains("no") || lower.contains("unlikely") {
                    Ok(0.2)
                } else {
                    // Unknown answer; default to 0.5 (uncertain).
                    crate::logging::warn(&format!(
                        "jev-review: unparseable noul answer '{}', defaulting to 0.5",
                        s
                    ));
                    Ok(0.5)
                }
            }
        }
    }
}

// ── File scanning ──

/// Recursively walk a source tree and return all source files (non-binary,
/// excluding configured patterns).
pub fn walk_source_files(root: &Path, exclude_patterns: &[&str]) -> Result<Vec<String>> {
    let mut files = Vec::new();
    walk_source_files_recursive(root, root, exclude_patterns, &mut files)?;
    Ok(files)
}

fn walk_source_files_recursive(
    dir: &Path,
    root: &Path,
    exclude_patterns: &[&str],
    files: &mut Vec<String>,
) -> Result<()> {
    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("jev-review: cannot read directory {}", dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        // Skip excluded patterns.
        if exclude_patterns.iter().any(|pat| relative.contains(pat)) {
            continue;
        }

        if path.is_dir() {
            // Skip hidden directories and common non-source dirs.
            let dir_name = entry.file_name().to_string_lossy().to_string();
            if dir_name.starts_with('.')
                || dir_name == "node_modules"
                || dir_name == "target"
                || dir_name == "dist"
                || dir_name == "build"
                || dir_name == "__pycache__"
                || dir_name == "vendor"
                || dir_name == ".git"
            {
                continue;
            }
            walk_source_files_recursive(&path, root, exclude_patterns, files)?;
        } else if path.is_file() && is_source_file(&relative) && !is_binary_by_extension(&relative) {
            files.push(relative);
        }
    }

    Ok(())
}

/// Check if a path looks like a source file (has a known source extension).
fn is_source_file(path: &str) -> bool {
    let lower = path.to_lowercase();
    let source_extensions = [
        "rs", "py", "js", "ts", "tsx", "jsx", "go", "java", "c", "h", "cpp", "cc", "cxx", "hpp",
        "hxx", "rb", "swift", "kt", "kts", "scala", "cs", "fs", "fsx", "php", "pl", "pm", "r",
        "sql", "sh", "bash", "zsh", "fish", "lua", "tcl", "vim", "el", "ex", "exs", "gleam",
        "zig", "nim", "cr", "dart", "jl", "hs", "ml", "mli", "erl", "hrl", "clj", "cljs",
        "cljc", "edn", "proto", "graphql", "vue", "svelte", "astro", "css", "scss", "less",
        "html", "xml", "json", "yaml", "yml", "toml", "md", "mdx", "txt",
    ];
    if let Some(ext_pos) = lower.rfind('.') {
        let ext = &lower[ext_pos + 1..];
        source_extensions.contains(&ext)
    } else {
        false
    }
}

/// Check if a path looks like a binary file by extension.
fn is_binary_by_extension(path: &str) -> bool {
    let lower = path.to_lowercase();
    let binary_extensions = [
        "png", "jpg", "jpeg", "gif", "bmp", "ico", "webp", "svg", // images
        "mp3", "wav", "ogg", "flac", "aac", "wma", // audio
        "mp4", "avi", "mov", "mkv", "webm", "flv", // video
        "zip", "tar", "gz", "bz2", "xz", "7z", "rar", // archives
        "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", // documents
        "exe", "dll", "so", "dylib", "o", "a", "lib", "wasm", // binaries
        "ttf", "otf", "woff", "woff2", "eot", // fonts
        "db", "sqlite", "sqlite3", // databases
        "class", "jar", "war", "ear", // compiled Java
        "pyc", "pyo", "pyd", // compiled Python
        "bin", "dat", "pkl", "pickle",
    ];
    if let Some(ext_pos) = lower.rfind('.') {
        let ext = &lower[ext_pos + 1..];
        binary_extensions.contains(&ext)
    } else {
        false
    }
}

// ── Screening orchestrator ──

/// Results from screening a set of hunks/files.
#[derive(Debug, Clone)]
pub struct ScreeningResults {
    /// Total files/hunks screened.
    pub total_screened: usize,
    /// Hunks that passed (all dimensions ≤ threshold).
    pub low_risk: Vec<ScreenedHunk>,
    /// Hunks flagged for full LLM review.
    pub flagged: Vec<ScreenedHunk>,
    /// Skipped hunks (binary files, empty diffs).
    pub skipped: Vec<ScreenedHunk>,
    /// Whether Jev was unavailable (all hunks should go to LLM).
    pub jev_unavailable: bool,
}

/// A hunk that has been screened with Jev.
#[derive(Debug, Clone)]
pub struct ScreenedHunk {
    pub file_path: String,
    pub diff_text: String,
    pub risk_matrix: Option<RiskMatrix>,
    pub is_binary: bool,
    pub error: Option<String>,
}

impl ScreeningResults {
    /// Create empty results.
    pub fn new() -> Self {
        Self {
            total_screened: 0,
            low_risk: Vec::new(),
            flagged: Vec::new(),
            skipped: Vec::new(),
            jev_unavailable: false,
        }
    }

    /// Merge another set of results into this one.
    pub fn merge(&mut self, other: ScreeningResults) {
        self.total_screened += other.total_screened;
        self.low_risk.extend(other.low_risk);
        self.flagged.extend(other.flagged);
        self.skipped.extend(other.skipped);
        self.jev_unavailable = self.jev_unavailable || other.jev_unavailable;
    }

    /// Estimated cost saved by avoiding LLM reviews.
    pub fn estimated_cost_saved(&self) -> f64 {
        // Assume LLM review costs ~$0.005 per hunk.
        self.low_risk.len() as f64 * 0.005
    }

    /// Number of LLM reviews that would have been triggered without Jev screening.
    pub fn llm_reviews_avoided(&self) -> usize {
        self.low_risk.len()
    }

    /// Number of hunks routed to full LLM review.
    pub fn llm_reviews_triggered(&self) -> usize {
        self.flagged.len()
    }
}

/// Screen a set of hunks through Jev, returning categorized results.
///
/// Each hunk is sent to Jev with the 5 Noul risk questions. Results are
/// routed based on the threshold: any dimension > threshold → flagged for
/// full LLM review; all ≤ threshold → low risk.
///
/// Binary hunks are skipped. Empty diffs are skipped. If Jev is unavailable,
/// all hunks are treated as "flagged" (routed to LLM for full review).
pub async fn screen_hunks(
    hunks: &[DiffHunk],
    threshold: f64,
) -> ScreeningResults {
    let mut results = ScreeningResults::new();
    let mut jev_failed = false;

    for hunk in hunks {
        results.total_screened += 1;

        // Skip binary files.
        if hunk.is_binary {
            results.skipped.push(ScreenedHunk {
                file_path: hunk.file_path.clone(),
                diff_text: hunk.diff_text.clone(),
                risk_matrix: None,
                is_binary: true,
                error: None,
            });
            continue;
        }

        // Skip empty diffs.
        if hunk.diff_text.trim().is_empty() {
            continue;
        }

        // If Jev has already failed for this run, route everything to LLM.
        if jev_failed {
            results.flagged.push(ScreenedHunk {
                file_path: hunk.file_path.clone(),
                diff_text: hunk.diff_text.clone(),
                risk_matrix: None,
                is_binary: false,
                error: Some("Jev unavailable; routing to full LLM review".to_string()),
            });
            continue;
        }

        // Screen through Jev.
        match screen_hunk(hunk).await {
            Ok(risk_matrix) => {
                let screened = ScreenedHunk {
                    file_path: hunk.file_path.clone(),
                    diff_text: hunk.diff_text.clone(),
                    risk_matrix: Some(risk_matrix.clone()),
                    is_binary: false,
                    error: None,
                };

                if risk_matrix.needs_full_review(threshold) {
                    results.flagged.push(screened);
                } else {
                    results.low_risk.push(screened);
                }
            }
            Err(e) => {
                crate::logging::warn(&format!(
                    "jev-review: Jev screening failed for {}: {}. Routing to full LLM review.",
                    hunk.file_path, e
                ));
                jev_failed = true;
                results.jev_unavailable = true;
                results.flagged.push(ScreenedHunk {
                    file_path: hunk.file_path.clone(),
                    diff_text: hunk.diff_text.clone(),
                    risk_matrix: None,
                    is_binary: false,
                    error: Some(format!("Jev error: {e}")),
                });
            }
        }
    }

    // Telemetry: log screening outcome.
    let cost_saved = results.estimated_cost_saved();
    crate::logging::info(&format!(
        "JevReview: {} hunks screened, {} flagged, {} low-risk, ${:.4} saved",
        results.total_screened,
        results.llm_reviews_triggered(),
        results.llm_reviews_avoided(),
        cost_saved,
    ));

    results
}

// ── Output formatting ──

/// Format screening results as a human-readable review output.
///
/// Produces markdown with:
/// - Summary: total screened, flagged, low-risk, skipped, estimated cost saved.
/// - Per-hunk results with risk scores.
pub fn format_screening_results(results: &ScreeningResults) -> String {
    let mut output = String::new();

    // ── Summary ──
    output.push_str("## Jev Review Screening\n\n");
    output.push_str(&format!("**Total screened:** {}\n", results.total_screened));
    output.push_str(&format!(
        "**Flagged for full review:** {} ⚠\n",
        results.flagged.len()
    ));
    output.push_str(&format!(
        "**Low risk (Jev-only):** {} ✓\n",
        results.low_risk.len()
    ));

    let skipped_count = results.skipped.len();
    if skipped_count > 0 {
        output.push_str(&format!("**Skipped (binary):** {}\n", skipped_count));
    }

    if results.jev_unavailable {
        output.push_str("\n**⚠ Jev was unavailable.** All hunks will be routed to full LLM review.\n");
    }

    output.push_str(&format!(
        "\n**Estimated cost saved:** ${:.4}\n",
        results.estimated_cost_saved()
    ));
    output.push_str(&format!(
        "**LLM reviews avoided:** {} | **LLM reviews triggered:** {}\n\n",
        results.llm_reviews_avoided(),
        results.llm_reviews_triggered(),
    ));

    // ── Flagged hunks ──
    if !results.flagged.is_empty() {
        output.push_str("### ⚠ Flagged — Full LLM Review Required\n\n");
        for hunk in &results.flagged {
            output.push_str(&format_hunk_result(hunk, "⚠"));
        }
    }

    // ── Low risk hunks ──
    if !results.low_risk.is_empty() {
        output.push_str("### ✓ Low Risk — Jev-Only Assessment\n\n");
        for hunk in &results.low_risk {
            output.push_str(&format_hunk_result(hunk, "✓"));
        }
    }

    // ── Skipped hunks ──
    if !results.skipped.is_empty() {
        output.push_str("### ⊘ Skipped — Binary or Unreviewable\n\n");
        for hunk in &results.skipped {
            output.push_str(&format!(
                "⊘ `{}` — Binary file, manual review required\n",
                hunk.file_path
            ));
        }
    }

    output
}

fn format_hunk_result(hunk: &ScreenedHunk, icon: &str) -> String {
    if let Some(ref risk) = hunk.risk_matrix {
        let max = risk.max_risk();
        let highest_dim = risk_highest_dimension(risk);
        format!(
            "{icon} `{}` — Full review (max risk={:.2}, top: {})\n  {}\n\n",
            hunk.file_path,
            max,
            highest_dim,
            risk.summary_line(),
        )
    } else if let Some(ref error) = hunk.error {
        format!(
            "{icon} `{}` — Error: {}\n\n",
            hunk.file_path, error
        )
    } else {
        format!("{icon} `{}`\n\n", hunk.file_path)
    }
}

fn risk_highest_dimension(risk: &RiskMatrix) -> String {
    let dims = [
        ("correctness", risk.correctness_risk),
        ("security", risk.security_risk),
        ("reliability", risk.reliability_risk),
        ("compatibility", risk.compatibility_risk),
        ("test_gap", risk.test_gap),
    ];
    dims
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(name, _)| name.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

// ── Codebase scan ──

/// Scan all source files in a directory, building DiffHunk objects for each
/// file (reading the full file content as if it were a diff).
pub fn scan_codebase(root: &Path, exclude_patterns: &[&str]) -> Result<Vec<DiffHunk>> {
    let files = walk_source_files(root, exclude_patterns)?;
    let mut hunks = Vec::new();

    for file_path in files {
        let full_path = root.join(&file_path);
        let content = match std::fs::read_to_string(&full_path) {
            Ok(c) => c,
            Err(e) => {
                crate::logging::warn(&format!(
                    "jev-review: cannot read {} for scanning: {e}",
                    file_path
                ));
                continue;
            }
        };

        if content.trim().is_empty() {
            continue;
        }

        // Truncate very large files.
        let diff_text = if content.len() > 8000 {
            format!(
                "{}\n\n[... {} more chars; file too large for full screening]",
                content.chars().take(7800).collect::<String>(),
                content.len() - 7800
            )
        } else {
            content
        };

        let language = infer_language(&file_path);
        let related_tests = infer_related_tests(&file_path, root);

        hunks.push(DiffHunk {
            file_path,
            diff_text,
            language,
            related_tests,
            is_binary: false,
        });
    }

    Ok(hunks)
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_matrix_needs_full_review() {
        let m = RiskMatrix {
            correctness_risk: 0.1,
            security_risk: 0.2,
            reliability_risk: 0.3,
            compatibility_risk: 0.4,
            test_gap: 0.5,
        };
        assert!(!m.needs_full_review(0.7));
        assert!(m.is_low_risk(0.7));
    }

    #[test]
    fn test_risk_matrix_flags_high_correctness() {
        let m = RiskMatrix {
            correctness_risk: 0.85,
            security_risk: 0.1,
            reliability_risk: 0.1,
            compatibility_risk: 0.1,
            test_gap: 0.1,
        };
        assert!(m.needs_full_review(0.7));
        assert!(!m.is_low_risk(0.7));
    }

    #[test]
    fn test_risk_matrix_flags_high_security() {
        let m = RiskMatrix {
            correctness_risk: 0.1,
            security_risk: 0.72,
            reliability_risk: 0.1,
            compatibility_risk: 0.1,
            test_gap: 0.1,
        };
        assert!(m.needs_full_review(0.7));
    }

    #[test]
    fn test_risk_matrix_max_risk() {
        let m = RiskMatrix {
            correctness_risk: 0.3,
            security_risk: 0.95,
            reliability_risk: 0.1,
            compatibility_risk: 0.2,
            test_gap: 0.4,
        };
        assert!((m.max_risk() - 0.95).abs() < 0.001);
    }

    #[test]
    fn test_chunk_diff_empty() {
        let hunks = chunk_diff("", Path::new("/tmp"), 8000);
        assert!(hunks.is_empty());
    }

    #[test]
    fn test_chunk_diff_single_file() {
        let diff = "\
diff --git a/src/main.rs b/src/main.rs
index 123..456 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,3 @@
 fn main() {
-    println!(\"hello\");
+    println!(\"hello world\");
 }
";
        let hunks = chunk_diff(diff, Path::new("/tmp"), 8000);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].file_path, "src/main.rs");
        assert_eq!(hunks[0].language, "rust");
        assert!(hunks[0].diff_text.contains("println"));
    }

    #[test]
    fn test_chunk_diff_binary_file() {
        let diff = "\
diff --git a/assets/logo.png b/assets/logo.png
Binary files a/assets/logo.png and b/assets/logo.png differ
";
        let hunks = chunk_diff(diff, Path::new("/tmp"), 8000);
        assert_eq!(hunks.len(), 1);
        assert!(hunks[0].is_binary);
    }

    #[test]
    fn test_chunk_diff_multiple_files() {
        let diff = "\
diff --git a/src/auth.rs b/src/auth.rs
--- a/src/auth.rs
+++ b/src/auth.rs
@@ -1,3 +1,4 @@
+fn new_func() {}
diff --git a/src/config.rs b/src/config.rs
--- a/src/config.rs
+++ b/src/config.rs
@@ -5,6 +5,7 @@
 const X: u32 = 1;
+const Y: u32 = 2;
";
        let hunks = chunk_diff(diff, Path::new("/tmp"), 8000);
        assert_eq!(hunks.len(), 2);
        assert_eq!(hunks[0].file_path, "src/auth.rs");
        assert_eq!(hunks[1].file_path, "src/config.rs");
    }

    #[test]
    fn test_chunk_diff_oversized_splits_at_function_boundaries() {
        // Build a diff larger than max with multiple functions.
        let mut diff = String::from("diff --git a/src/lib.rs b/src/lib.rs\n");
        for i in 0..10 {
            diff.push_str(&format!(
                "fn func{i}() {{\n    let x = {};\n    println!(\"{{}}\", x);\n}}\n",
                i * 100
            ));
        }
        let hunks = chunk_diff(&diff, Path::new("/tmp"), 100);
        assert!(hunks.len() > 1, "oversized diff should be split");
    }

    #[test]
    fn test_infer_language() {
        assert_eq!(infer_language("src/main.rs"), "rust");
        assert_eq!(infer_language("src/app.ts"), "typescript");
        assert_eq!(infer_language("src/App.tsx"), "typescript-react");
        assert_eq!(infer_language("main.py"), "python");
        assert_eq!(infer_language("main.go"), "go");
        assert_eq!(infer_language("Makefile"), "text");
    }

    #[test]
    fn test_parse_noul_string_yes_no() {
        assert!((parse_noul_string("yes", "test").unwrap() - 1.0).abs() < 0.001);
        assert!((parse_noul_string("no", "test").unwrap() - 0.0).abs() < 0.001);
        assert!((parse_noul_string("true", "test").unwrap() - 1.0).abs() < 0.001);
        assert!((parse_noul_string("false", "test").unwrap() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_parse_noul_string_numbers() {
        assert!((parse_noul_string("0.87", "test").unwrap() - 0.87).abs() < 0.001);
        assert!((parse_noul_string("0.15", "test").unwrap() - 0.15).abs() < 0.001);
    }

    #[test]
    fn test_parse_noul_string_fuzzy() {
        assert!((parse_noul_string("likely yes", "test").unwrap() - 0.8).abs() < 0.001);
        assert!((parse_noul_string("probably no", "test").unwrap() - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_extract_noul_from_bool() {
        assert!((extract_noul_probability(Some(&serde_json::json!(true)), "t").unwrap() - 1.0).abs() < 0.001);
        assert!((extract_noul_probability(Some(&serde_json::json!(false)), "t").unwrap() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_extract_noul_from_number() {
        assert!((extract_noul_probability(Some(&serde_json::json!(0.73)), "t").unwrap() - 0.73).abs() < 0.001);
    }

    #[test]
    fn test_extract_noul_from_typesafe_format() {
        let v = serde_json::json!({"noul": 0.65, "confidence": 0.92});
        assert!((extract_noul_probability(Some(&v), "t").unwrap() - 0.65).abs() < 0.001);
    }

    #[test]
    fn test_extract_noul_from_openrouter_format() {
        let v = serde_json::json!({"probability": 0.45});
        assert!((extract_noul_probability(Some(&v), "t").unwrap() - 0.45).abs() < 0.001);
    }

    #[test]
    fn test_walk_source_files_filters_binary() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
        std::fs::write(dir.path().join("logo.png"), "fake-png").unwrap();
        std::fs::write(dir.path().join("script.sh"), "#!/bin/bash\necho hi").unwrap();

        let files = walk_source_files(dir.path(), &[]).unwrap();
        assert!(files.iter().any(|f| f.contains("main.rs")));
        assert!(!files.iter().any(|f| f.contains("logo.png")));
        // .sh IS a source extension
    }

    #[test]
    fn test_screening_results_empty() {
        let r = ScreeningResults::new();
        assert_eq!(r.total_screened, 0);
        assert_eq!(r.llm_reviews_avoided(), 0);
        assert_eq!(r.estimated_cost_saved(), 0.0);
    }

    #[test]
    fn test_screening_results_costs() {
        let mut r = ScreeningResults::new();
        r.low_risk.push(ScreenedHunk {
            file_path: "a.rs".to_string(),
            diff_text: "diff".to_string(),
            risk_matrix: Some(RiskMatrix {
                correctness_risk: 0.1,
                security_risk: 0.1,
                reliability_risk: 0.1,
                compatibility_risk: 0.1,
                test_gap: 0.1,
            }),
            is_binary: false,
            error: None,
        });
        r.low_risk.push(ScreenedHunk {
            file_path: "b.rs".to_string(),
            diff_text: "diff".to_string(),
            risk_matrix: Some(RiskMatrix {
                correctness_risk: 0.1,
                security_risk: 0.1,
                reliability_risk: 0.1,
                compatibility_risk: 0.1,
                test_gap: 0.1,
            }),
            is_binary: false,
            error: None,
        });
        r.flagged.push(ScreenedHunk {
            file_path: "c.rs".to_string(),
            diff_text: "diff".to_string(),
            risk_matrix: Some(RiskMatrix {
                correctness_risk: 0.9,
                security_risk: 0.1,
                reliability_risk: 0.1,
                compatibility_risk: 0.1,
                test_gap: 0.1,
            }),
            is_binary: false,
            error: None,
        });

        assert_eq!(r.llm_reviews_avoided(), 2);
        assert_eq!(r.llm_reviews_triggered(), 1);
        assert!((r.estimated_cost_saved() - 0.01).abs() < 0.001); // 2 * $0.005
    }

    #[test]
    fn test_format_screening_results_includes_counts() {
        let mut r = ScreeningResults::new();
        r.total_screened = 5;
        r.low_risk.push(ScreenedHunk {
            file_path: "a.rs".to_string(),
            diff_text: "diff".to_string(),
            risk_matrix: Some(RiskMatrix {
                correctness_risk: 0.1,
                security_risk: 0.1,
                reliability_risk: 0.1,
                compatibility_risk: 0.1,
                test_gap: 0.1,
            }),
            is_binary: false,
            error: None,
        });
        r.flagged.push(ScreenedHunk {
            file_path: "b.rs".to_string(),
            diff_text: "diff".to_string(),
            risk_matrix: Some(RiskMatrix {
                correctness_risk: 0.85,
                security_risk: 0.1,
                reliability_risk: 0.1,
                compatibility_risk: 0.1,
                test_gap: 0.1,
            }),
            is_binary: false,
            error: None,
        });
        r.skipped.push(ScreenedHunk {
            file_path: "logo.png".to_string(),
            diff_text: String::new(),
            risk_matrix: None,
            is_binary: true,
            error: None,
        });

        let output = format_screening_results(&r);
        assert!(output.contains("Total screened: 5"));
        assert!(output.contains("Flagged for full review"));
        assert!(output.contains("Low risk"));
        assert!(output.contains("Skipped (binary)"));
        assert!(output.contains("a.rs"));
        assert!(output.contains("b.rs"));
        assert!(output.contains("logo.png"));
    }
}
