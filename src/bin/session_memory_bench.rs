use clap::{Parser, ValueEnum};
use jcode::message::{ContentBlock, Role};
use jcode::process_memory;
use jcode::session::Session;
use jcode::side_panel::{
    SidePanelPage, SidePanelPageFormat, SidePanelPageSource, SidePanelSnapshot,
};

#[derive(Parser, Debug)]
#[command(about = "Benchmark heavy-session memory attribution and process footprint")]
struct Args {
    /// Scenario source
    #[arg(long, value_enum, default_value = "synthetic")]
    scenario: Scenario,

    /// Saved session id or path (required for --scenario saved)
    #[arg(long)]
    session: Option<String>,

    /// Memory mode to benchmark
    #[arg(long, value_enum, default_value = "local")]
    mode: BenchMode,

    /// Synthetic turns to generate
    #[arg(long, default_value_t = 24)]
    turns: usize,

    /// Synthetic tool input size in KiB per turn
    #[arg(long, default_value_t = 4)]
    tool_input_kib: usize,

    /// Synthetic tool output size in KiB per turn
    #[arg(long, default_value_t = 48)]
    tool_output_kib: usize,

    /// Synthetic side-panel page count
    #[arg(long, default_value_t = 0)]
    side_panel_pages: usize,

    /// Synthetic side-panel content size in KiB per page
    #[arg(long, default_value_t = 32)]
    side_panel_page_kib: usize,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum Scenario {
    Synthetic,
    Saved,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum BenchMode {
    /// Current local steady state: canonical session + display, provider view only transient
    Local,
    /// Simulated pre-refactor duplicate steady state: keep a resident provider copy too
    Duplicated,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let executable = std::env::current_exe().ok();
    let metadata = benchmark_metadata(
        executable.as_deref(),
        jcode_build_meta::VERSION,
        jcode_build_meta::GIT_HASH,
    );

    let process_before = process_memory::snapshot_with_source("bench:session-memory:before");
    let session = load_or_build_session(&args)?;
    let display_messages = jcode::tui::display_messages_from_session(&session);
    let side_panel = build_side_panel(&args);

    let resident_provider_messages = match args.mode {
        BenchMode::Local => Vec::new(),
        BenchMode::Duplicated => session.messages_for_provider_uncached(),
    };

    let process_after_build =
        process_memory::snapshot_with_source("bench:session-memory:after-build");

    let materialized_provider_messages = match args.mode {
        BenchMode::Local => session.messages_for_provider_uncached(),
        BenchMode::Duplicated => resident_provider_messages.clone(),
    };
    let provider_view_source = match args.mode {
        BenchMode::Local => "session_materialized",
        BenchMode::Duplicated => "resident_ui",
    };

    let client_memory = jcode::tui::transcript_memory_profile(
        &session,
        &resident_provider_messages,
        &materialized_provider_messages,
        provider_view_source,
        &display_messages,
        &side_panel,
    );

    let process_after_profile =
        process_memory::snapshot_with_source("bench:session-memory:after-profile");

    drop(materialized_provider_messages);
    let process_after_drop_transient =
        process_memory::snapshot_with_source("bench:session-memory:after-drop-transient");

    let mut payload = serde_json::json!({
        "metadata": metadata,
        "scenario": match args.scenario {
            Scenario::Synthetic => "synthetic",
            Scenario::Saved => "saved",
        },
        "mode": match args.mode {
            BenchMode::Local => "local",
            BenchMode::Duplicated => "duplicated",
        },
        "config": {
            "turns": args.turns,
            "tool_input_kib": args.tool_input_kib,
            "tool_output_kib": args.tool_output_kib,
            "side_panel_pages": args.side_panel_pages,
            "side_panel_page_kib": args.side_panel_page_kib,
            "session": args.session,
        },
        "counts": {
            "session_messages": session.messages.len(),
            "display_messages": display_messages.len(),
            "resident_provider_messages": resident_provider_messages.len(),
            "side_panel_pages": side_panel.pages.len(),
        },
        "process": {
            "before": process_before,
            "after_build": process_after_build,
            "after_profile": process_after_profile,
            "after_drop_transient": process_after_drop_transient,
        },
        "client_memory": client_memory,
    });

    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok();
    redact_report(&mut payload, home.as_deref());
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

fn benchmark_metadata(
    executable: Option<&std::path::Path>,
    version: &str,
    revision: &str,
) -> serde_json::Value {
    let identity = |value: &str| {
        if value.trim().is_empty() {
            "unknown".to_string()
        } else {
            value.to_string()
        }
    };
    let binary_path = executable
        .and_then(|path| path.to_str())
        .map(identity)
        .unwrap_or_else(|| "unknown".to_string());
    let version = identity(version);
    let revision = identity(revision);
    let cpu = std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|text| {
            text.lines().find_map(|line| {
                let (key, value) = line.split_once(':')?;
                (key.trim() == "model name").then(|| identity(value.trim()))
            })
        })
        .unwrap_or_else(|| "unknown".to_string());
    let memory_bytes = std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|text| {
            text.lines().find_map(|line| {
                let value = line.strip_prefix("MemTotal:")?;
                value
                    .split_whitespace()
                    .next()?
                    .parse::<u64>()
                    .ok()?
                    .checked_mul(1024)
            })
        });
    serde_json::json!({
        "comparable": ![&binary_path, &version, &revision, &cpu].iter().any(|value| value.as_str() == "unknown")
            && memory_bytes.is_some(),
        "binary_path": binary_path,
        "version": version,
        "revision": revision,
        "git_dirty": jcode_build_meta::git_dirty(),
        "source_fingerprint": jcode_build_meta::SOURCE_FINGERPRINT,
        "cleanliness_verified": jcode_build_meta::cleanliness_verified(),
        "hardware": {
            "cpu": cpu,
            "cpu_arch": std::env::consts::ARCH,
            "os": std::env::consts::OS,
            "memory_bytes": memory_bytes.map(serde_json::Value::from)
                .unwrap_or_else(|| serde_json::json!("unknown")),
        },
    })
}

fn redact_report(value: &mut serde_json::Value, home: Option<&str>) {
    match value {
        serde_json::Value::String(text) => {
            if let Some(home) = home.filter(|home| !home.is_empty() && *home != "/") {
                *text = text.replace(home.trim_end_matches('/'), "~");
            }
            *text = jcode::message::redact_secrets(text);
        }
        serde_json::Value::Array(values) => {
            for value in values {
                redact_report(value, home);
            }
        }
        serde_json::Value::Object(values) => {
            for value in values.values_mut() {
                redact_report(value, home);
            }
        }
        _ => {}
    }
}

fn load_or_build_session(args: &Args) -> anyhow::Result<Session> {
    match args.scenario {
        Scenario::Synthetic => Ok(build_synthetic_session(
            args.turns,
            args.tool_input_kib,
            args.tool_output_kib,
        )),
        Scenario::Saved => {
            let Some(value) = args.session.as_deref() else {
                anyhow::bail!("--session is required with --scenario saved");
            };
            let path = std::path::Path::new(value);
            if path.exists() {
                Session::load_from_path(path)
            } else {
                Session::load(value)
            }
        }
    }
}

fn build_synthetic_session(turns: usize, tool_input_kib: usize, tool_output_kib: usize) -> Session {
    let mut session = Session::create_with_id(
        format!("session_memory_bench_{}", std::process::id()),
        None,
        Some("session memory bench".to_string()),
    );
    let tool_input_bytes = tool_input_kib * 1024;
    let tool_output_bytes = tool_output_kib * 1024;

    for idx in 0..turns {
        session.add_message(
            Role::User,
            vec![text_block(make_blob(
                &format!("user turn {idx} - "),
                768 + (idx % 5) * 64,
            ))],
        );
        session.add_message(
            Role::Assistant,
            vec![
                text_block(make_blob(
                    &format!("assistant summary {idx} - "),
                    1024 + (idx % 7) * 96,
                )),
                ContentBlock::ToolUse {
                    id: format!("tool_{idx}"),
                    name: "bash".to_string(),
                    input: serde_json::json!({
                        "command": make_blob(&format!("printf 'turn {idx}' && # "), tool_input_bytes),
                        "description": format!("Synthetic tool call {idx}"),
                    }), thought_signature: None, },
            ],
        );
        session.add_message(
            Role::User,
            vec![ContentBlock::ToolResult {
                tool_use_id: format!("tool_{idx}"),
                content: make_blob(&format!("tool output {idx} - "), tool_output_bytes),
                is_error: None,
            }],
        );
    }

    session
}

fn build_side_panel(args: &Args) -> SidePanelSnapshot {
    if args.side_panel_pages == 0 {
        return SidePanelSnapshot::default();
    }

    let mut pages = Vec::with_capacity(args.side_panel_pages);
    for idx in 0..args.side_panel_pages {
        pages.push(SidePanelPage {
            id: format!("bench_page_{idx}"),
            title: format!("Bench Page {idx}"),
            file_path: format!("/tmp/bench_page_{idx}.md"),
            format: SidePanelPageFormat::Markdown,
            pdf_data: None,
            source: SidePanelPageSource::Managed,
            content: make_blob(
                &format!("# Bench Page {idx}\n\n"),
                args.side_panel_page_kib * 1024,
            ),
            updated_at_ms: idx as u64,
        });
    }

    SidePanelSnapshot {
        focus_revision: 0,
        focused_page_id: pages.first().map(|page| page.id.clone()),
        pages,
    }
}

fn text_block(text: String) -> ContentBlock {
    ContentBlock::Text {
        text,
        cache_control: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_binary_identity_is_unknown_and_non_comparable() {
        let metadata = benchmark_metadata(None, "", "unknown");
        assert_eq!(metadata["binary_path"], "unknown");
        assert_eq!(metadata["version"], "unknown");
        assert_eq!(metadata["revision"], "unknown");
        assert_eq!(metadata["comparable"], false);
    }

    #[test]
    fn metadata_records_identity_and_hardware() {
        let metadata = benchmark_metadata(Some(std::path::Path::new("/bench")), "v1", "abc123");
        assert_eq!(metadata["binary_path"], "/bench");
        assert_eq!(metadata["version"], "v1");
        assert_eq!(metadata["revision"], "abc123");
        assert!(metadata["hardware"]["cpu_arch"].is_string());
        assert!(metadata["hardware"].get("memory_bytes").is_some());
    }

    #[test]
    fn report_redacts_home_and_secrets_without_changing_measurements() {
        let secret = "sk-ant-oat01-ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut value = serde_json::json!({
            "path": "/home/example/session.json",
            "nested": [{"value": secret}],
            "measurement": 42,
        });
        redact_report(&mut value, Some("/home/example"));
        assert_eq!(value["path"], "~/session.json");
        assert_eq!(value["measurement"], 42);
        assert!(!value.to_string().contains(secret));
        assert!(!value.to_string().contains("/home/example"));
        redact_report(&mut value, Some(""));
        assert_eq!(value["path"], "~/session.json");
    }
}

fn make_blob(prefix: &str, target_len: usize) -> String {
    if target_len <= prefix.len() {
        return prefix[..target_len].to_string();
    }
    let mut out = String::with_capacity(target_len);
    out.push_str(prefix);
    const CHUNK: &str = "abcdefghijklmnopqrstuvwxyz0123456789 ";
    while out.len() < target_len {
        out.push_str(CHUNK);
    }
    out.truncate(target_len);
    out
}
