//! Explicit TOML composition for `config.toml`.
//!
//! The root file may carry a top-level `include = ["config/ui.toml", ...]`
//! manifest. Listed fragments are merged in order (recursive table merge, later
//! scalars and arrays replace earlier ones), then the root's own settings are
//! applied on top. Typed defaults and environment overrides are applied by the
//! caller after a successful typed load. Only one include level exists and no
//! directory is ever scanned.
//!
//! Diagnostics name source paths and keys/lines. They never echo configuration
//! values or source excerpts.

use std::collections::{BTreeMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::{Component, Path, PathBuf};

pub(super) const INCLUDE_KEY: &str = "include";

/// Result of composing the root document and its listed fragments.
pub(super) struct Composed {
    /// Merged document with the manifest removed.
    pub table: toml::Table,
}

/// Mask value-bearing text in a serde/toml message so values never leak.
///
/// serde reports offending values as quoted strings (`"..."`, escaped, so also
/// multi-line), backticked tokens (`` `5` ``, `` `1.5` ``, unknown variants)
/// and apostrophe literals. Everything before the schema suffix
/// (", expected ...") is scanned and each quoted or backticked segment is
/// replaced, an unterminated segment masks the remainder. Field-name messages
/// keep their key names. The expected-schema suffix (", expected ...", found
/// outside any quoted segment) is never value-bearing.
pub(super) fn redact_message(message: &str) -> String {
    let keeps_names = [
        "missing field",
        "unknown field",
        "duplicate field",
        "duplicate key",
    ]
    .iter()
    .any(|prefix| message.starts_with(prefix));
    let mut out = String::with_capacity(message.len());
    let mut chars = message.char_indices();
    let mut prev = ' ';
    while let Some((at, ch)) = chars.next() {
        let before = std::mem::replace(&mut prev, ch);
        if keeps_names || message[at..].starts_with(", expected") {
            out.push_str(&message[at..]);
            break;
        }
        // An apostrophe inside a word ("doesn't") is punctuation, not a quote.
        let quote = matches!(ch, '"' | '`') || (ch == '\'' && !before.is_alphanumeric());
        if !quote {
            out.push(ch);
            continue;
        }
        out.push_str(&format!("{ch}<redacted>{ch}"));
        let mut escaped = false;
        for (_, inner) in chars.by_ref() {
            if escaped {
                escaped = false;
            } else if inner == '\\' && ch == '"' {
                escaped = true;
            } else if inner == ch {
                break;
            }
        }
    }
    out
}

/// Format a TOML error as `path:line:col: message` without any excerpt.
pub(super) fn describe_parse_error(path: &Path, text: &str, error: &toml::de::Error) -> String {
    let location = error
        .span()
        .map(|span| {
            let start = span.start.min(text.len());
            let before = &text.as_bytes()[..start];
            let line = before.iter().filter(|b| **b == b'\n').count() + 1;
            let col = start
                - before
                    .iter()
                    .rposition(|b| *b == b'\n')
                    .map_or(0, |i| i + 1)
                + 1;
            format!(":{line}:{col}")
        })
        .unwrap_or_default();
    format!(
        "{}{}: {}",
        path.display(),
        location,
        redact_message(error.message())
    )
}

/// Describe a typed deserialization failure for a composed document.
pub(super) fn describe_typed_error(path: &Path, error: &toml::de::Error) -> String {
    format!("{}: {}", path.display(), redact_message(error.message()))
}

fn parse_table(path: &Path, text: &str) -> anyhow::Result<toml::Table> {
    text.parse::<toml::Table>().map_err(|error| {
        anyhow::anyhow!(
            "Failed to parse config file {}",
            describe_parse_error(path, text, &error)
        )
    })
}

/// Validate one manifest entry and return it as a clean relative path.
pub(super) fn validate_include_entry(entry: &str) -> anyhow::Result<PathBuf> {
    if entry.trim().is_empty() {
        anyhow::bail!("include entries must be non-empty paths");
    }
    if entry.chars().any(|c| {
        matches!(
            c,
            '$' | '%' | '*' | '?' | '[' | ']' | '{' | '}' | '`' | '\\' | '\0'
        )
    }) || entry.starts_with('~')
    {
        anyhow::bail!(
            "include `{entry}` uses expansion or glob syntax, which is not supported; list explicit relative paths"
        );
    }
    let mut clean = PathBuf::new();
    for component in Path::new(entry).components() {
        match component {
            Component::Normal(part) => clean.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                anyhow::bail!(
                    "include `{entry}` contains `..`; includes must stay inside the config directory"
                )
            }
            Component::RootDir | Component::Prefix(_) => {
                anyhow::bail!(
                    "include `{entry}` is absolute; use a path relative to the config directory"
                )
            }
        }
    }
    let is_toml = clean
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("toml"));
    if !is_toml {
        anyhow::bail!("include `{entry}` must be a `.toml` file");
    }
    Ok(clean)
}

/// Extract the manifest from a root table, removing the key.
fn take_manifest(root: &mut toml::Table, root_path: &Path) -> anyhow::Result<Vec<String>> {
    let Some(value) = root.remove(INCLUDE_KEY) else {
        return Ok(Vec::new());
    };
    let toml::Value::Array(items) = value else {
        anyhow::bail!(
            "{}: `include` must be an array of relative TOML paths",
            root_path.display()
        );
    };
    items
        .into_iter()
        .map(|item| match item {
            toml::Value::String(entry) => Ok(entry),
            _ => Err(anyhow::anyhow!(
                "{}: `include` must contain only strings",
                root_path.display()
            )),
        })
        .collect()
}

struct Origins {
    labels: Vec<String>,
    by_key: BTreeMap<String, usize>,
}

impl Origins {
    /// Record `source` as the origin of `key` and of every key nested inside a
    /// table value, so later conflicts can name the source of any nested key.
    fn record(&mut self, key: &str, value: &toml::Value, source: usize) {
        self.by_key.insert(key.to_string(), source);
        if let toml::Value::Table(table) = value {
            for (child, nested) in table {
                self.record(&format!("{key}.{child}"), nested, source);
            }
        }
    }

    fn label(&self, key: &str) -> &str {
        self.by_key
            .get(key)
            .and_then(|index| self.labels.get(*index))
            .map_or("an earlier source", String::as_str)
    }
}

fn merge_table(
    dst: &mut toml::Table,
    src: toml::Table,
    prefix: &str,
    source: usize,
    origins: &mut Origins,
) -> anyhow::Result<()> {
    for (key, incoming) in src {
        let dotted = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}.{key}")
        };
        match (dst.get_mut(&key), incoming) {
            (Some(toml::Value::Table(existing)), toml::Value::Table(next)) => {
                merge_table(existing, next, &dotted, source, origins)?;
            }
            (Some(toml::Value::Table(_)), _) | (Some(_), toml::Value::Table(_)) => {
                anyhow::bail!(
                    "config key `{dotted}` is a table in one source and a value in another ({} vs {})",
                    origins.label(&dotted),
                    origins.labels[source]
                );
            }
            (Some(slot), incoming) => {
                origins.record(&dotted, &incoming, source);
                *slot = incoming;
            }
            (None, incoming) => {
                origins.record(&dotted, &incoming, source);
                dst.insert(key, incoming);
            }
        }
    }
    Ok(())
}

/// Read and validate every fragment the manifest selects, in listed order.
fn load_fragments(
    root_path: &Path,
    manifest: &[String],
) -> anyhow::Result<Vec<(PathBuf, toml::Table)>> {
    let mut loaded = Vec::new();
    if manifest.is_empty() {
        return Ok(loaded);
    }
    let root_dir = root_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("config path {} has no directory", root_path.display()))?;
    let root_dir = if root_dir.as_os_str().is_empty() {
        Path::new(".")
    } else {
        root_dir
    };
    let canonical_dir = std::fs::canonicalize(root_dir).map_err(|e| {
        anyhow::anyhow!(
            "Failed to resolve config directory {}: {e}",
            root_dir.display()
        )
    })?;
    let canonical_root = std::fs::canonicalize(root_path).ok();
    let mut seen = HashSet::new();
    for entry in manifest {
        let here = root_path.display();
        let relative = validate_include_entry(entry).map_err(|e| anyhow::anyhow!("{here}: {e}"))?;
        let canonical = std::fs::canonicalize(canonical_dir.join(&relative)).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                anyhow::anyhow!("{here}: include `{entry}` does not exist")
            } else {
                anyhow::anyhow!("{here}: include `{entry}` is unreadable: {e}")
            }
        })?;
        if !canonical.starts_with(&canonical_dir) {
            anyhow::bail!("{here}: include `{entry}` resolves outside the config directory");
        }
        if canonical_root.as_deref() == Some(canonical.as_path()) {
            anyhow::bail!("{here}: include `{entry}` includes the root file itself");
        }
        if !std::fs::metadata(&canonical).is_ok_and(|m| m.is_file()) {
            anyhow::bail!("{here}: include `{entry}` is not a regular file");
        }
        if !seen.insert(canonical.clone()) {
            anyhow::bail!("{here}: include `{entry}` duplicates an earlier include");
        }
        let bytes = std::fs::read(&canonical)
            .map_err(|e| anyhow::anyhow!("{here}: include `{entry}` is unreadable: {e}"))?;
        let text = String::from_utf8(bytes)
            .map_err(|_| anyhow::anyhow!("{here}: include `{entry}` is not valid UTF-8"))?;
        let table = parse_table(&canonical, &text)?;
        if table.contains_key(INCLUDE_KEY) {
            anyhow::bail!(
                "{}: nested `include` is not supported; only the root config may list includes",
                canonical.display()
            );
        }
        loaded.push((canonical, table));
    }
    Ok(loaded)
}

/// The listed fragments of `root_text`, parsed, in include order.
pub(super) fn fragment_tables(
    root_path: &Path,
    root_text: &str,
) -> anyhow::Result<Vec<(PathBuf, toml::Table)>> {
    let mut root = parse_table(root_path, root_text)?;
    let manifest = take_manifest(&mut root, root_path)?;
    load_fragments(root_path, &manifest)
}

/// Compose `root_path` (whose text is `root_text`) with its listed fragments.
pub(super) fn compose(root_path: &Path, root_text: &str) -> anyhow::Result<Composed> {
    let mut root = parse_table(root_path, root_text)?;
    let manifest = take_manifest(&mut root, root_path)?;
    let loaded = load_fragments(root_path, &manifest)?;
    if loaded.is_empty() {
        return Ok(Composed { table: root });
    }
    let mut origins = Origins {
        labels: loaded
            .iter()
            .map(|(p, _)| p.display().to_string())
            .collect(),
        by_key: BTreeMap::new(),
    };
    origins.labels.push(root_path.display().to_string());
    let root_index = loaded.len();
    let mut merged = toml::Table::new();
    for (index, (_, table)) in loaded.into_iter().enumerate() {
        merge_table(&mut merged, table, "", index, &mut origins)?;
    }
    merge_table(&mut merged, root, "", root_index, &mut origins)?;
    Ok(Composed { table: merged })
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SourceStamp {
    Missing,
    Unreadable(std::io::ErrorKind),
    Content { len: u64, hash: u64 },
}

impl SourceStamp {
    fn read(path: &Path) -> (Self, Option<Vec<u8>>) {
        match std::fs::read(path) {
            Ok(bytes) => {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                bytes.hash(&mut hasher);
                (
                    Self::Content {
                        len: bytes.len() as u64,
                        hash: hasher.finish(),
                    },
                    Some(bytes),
                )
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (Self::Missing, None),
            Err(e) => (Self::Unreadable(e.kind()), None),
        }
    }
}

/// Content identity of the root file and every file its manifest selects.
///
/// Captures missing/unreadable states and manifest membership so additions,
/// removals, equal-length edits, deletion and restoration all change it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct SourceFingerprint {
    root: Option<PathBuf>,
    entries: Vec<(PathBuf, SourceStamp)>,
}

impl SourceFingerprint {
    pub(crate) fn read(root: Option<PathBuf>) -> Self {
        let Some(root_path) = root.clone() else {
            return Self::default();
        };
        let (root_stamp, root_bytes) = SourceStamp::read(&root_path);
        let mut entries = vec![(root_path.clone(), root_stamp)];
        if let Some(bytes) = root_bytes {
            for relative in selected_includes(&bytes) {
                let path = root_path
                    .parent()
                    .map_or_else(|| relative.clone(), |dir| dir.join(&relative));
                let (stamp, _) = SourceStamp::read(&path);
                entries.push((path, stamp));
            }
        }
        Self { root, entries }
    }

    /// Whether any selected source differs from `other` (for reload logs).
    pub(crate) fn describe_change(&self, other: &Self) -> Option<String> {
        if self == other {
            return None;
        }
        if self.root != other.root {
            return Some("root_path_changed=true".to_string());
        }
        let membership_changed = self.entries.len() != other.entries.len()
            || self
                .entries
                .iter()
                .zip(&other.entries)
                .any(|(a, b)| a.0 != b.0);
        if membership_changed {
            Some("include_set_changed=true".to_string())
        } else {
            Some("source_content_changed=true".to_string())
        }
    }
}

/// The root file followed by every fragment its manifest selects.
///
/// Lenient (a broken manifest yields just the root) and never merges values, so
/// edit watchers can find every file that influences the effective config.
pub(crate) fn source_paths(root: &Path) -> Vec<PathBuf> {
    let mut paths = vec![root.to_path_buf()];
    if let Ok(bytes) = std::fs::read(root) {
        for relative in selected_includes(&bytes) {
            paths.push(
                root.parent()
                    .map_or_else(|| relative.clone(), |dir| dir.join(&relative)),
            );
        }
    }
    paths
}

/// Lenient manifest read used only to decide which files to watch.
fn selected_includes(root_bytes: &[u8]) -> Vec<PathBuf> {
    let Ok(text) = std::str::from_utf8(root_bytes) else {
        return Vec::new();
    };
    // Quoted keys can spell `include` with escapes, so only skip the parse when
    // neither the literal word nor an escape could produce it.
    if !text.contains(INCLUDE_KEY) && !text.contains('\\') {
        return Vec::new();
    }
    let Ok(table) = text.parse::<toml::Table>() else {
        return Vec::new();
    };
    let Some(toml::Value::Array(items)) = table.get(INCLUDE_KEY) else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| item.as_str())
        .filter_map(|entry| validate_include_entry(entry).ok())
        .collect()
}
