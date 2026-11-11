use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::server;

pub fn run_terminal_command() -> Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(&terminal_capability_report())?
    );
    Ok(())
}

fn terminal_capability_report() -> serde_json::Value {
    use std::io::IsTerminal;
    let color = jcode_tui_style::color_capability();
    let image = crate::tui::image::ImageProtocol::detect();
    let policy = crate::perf::tui_policy();
    let stdout_is_terminal = std::io::stdout().is_terminal();
    serde_json::json!({
        "terminal": crate::message::redact_secrets(&crate::perf::profile().terminal),
        "stdout_is_terminal": stdout_is_terminal,
        "color": {
            "detected": format!("{color:?}"),
            "fallback": if jcode_tui_style::theme_mode::no_color() { "monochrome (NO_COLOR/JCODE_NO_COLOR)" } else if color == jcode_tui_style::ColorCapability::Color256 { "256-color quantization" } else { "none" },
        },
        "image": {
            "detected": format!("{image:?}"),
            "disabled_by_env": jcode_tui::tui::image::images_disabled(),
            "fallback": if jcode_tui::tui::image::images_disabled() { "text (JCODE_IMAGES=off)" } else if image.is_supported() && stdout_is_terminal { "protocol output available" } else { "text (image protocol unavailable or redirected output)" },
        },
        "mouse": {
            "configured": policy.enable_mouse_capture,
            "terminal_probe": "not performed",
            "fallback": if policy.enable_mouse_capture { "keyboard remains available" } else { "keyboard only" },
        },
        "ascii_safe": {
            "enabled": jcode_tui_style::theme::ascii_safe(),
            "scope": "shared box/rail borders, truncation markers, activity spinners, onboarding pill caps",
            "fallback": if jcode_tui_style::theme::ascii_safe() { "ASCII symbols on covered chrome only" } else { "existing Unicode chrome" },
            "content_transformed": false,
            "full_ascii_output": false,
            "coverage_gaps": "other generated chrome, Markdown decoration, header icons, usage bars, swarm glyphs and custom borders",
        },
        "unicode_width": {
            "mode": "unicode-width",
            "terminal_probe": "not performed",
            "fallback": "fixed Unicode cell widths; terminal glyph compatibility is not probed",
        },
    })
}

/// Print every candidate instruction file for `dir` (default: cwd), in prompt order.
pub fn run_instructions_command(dir: Option<&std::path::Path>) -> Result<()> {
    let dir = match dir {
        Some(dir) => dir.to_path_buf(),
        None => std::env::current_dir()?,
    };
    if !dir.is_dir() {
        anyhow::bail!("not a directory: {}", dir.display());
    }
    print!("{}", instructions_report(&dir));
    Ok(())
}

fn instructions_report(dir: &std::path::Path) -> String {
    let (_, info) = crate::prompt::load_agents_md_files_from_dir(Some(dir));
    let mut out = String::new();
    for row in crate::prompt::instruction_source_rows(&info.instruction_sources) {
        out.push_str(&row);
        out.push('\n');
    }
    out
}

pub async fn run_debug_command(
    command: &str,
    arg: &str,
    session_id: Option<String>,
    socket_path: Option<String>,
    _wait: bool,
) -> Result<()> {
    match command {
        "list" => return debug_list_servers().await,
        "start" => return debug_start_server(arg, socket_path).await,
        _ => {}
    }

    let debug_socket = if let Some(ref path) = socket_path {
        let main_path = std::path::PathBuf::from(path);
        let filename = main_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("jcode.sock");
        let debug_filename = filename.replace(".sock", "-debug.sock");
        main_path.with_file_name(debug_filename)
    } else {
        server::debug_socket_path()
    };

    if !crate::transport::is_socket_path(&debug_socket) {
        eprintln!("Debug socket not found at {:?}", debug_socket);
        eprintln!("\nMake sure:");
        eprintln!("  1. A jcode server is running (jcode or jcode serve)");
        eprintln!("  2. debug_socket is enabled in ~/.jcode/config.toml");
        eprintln!("     [display]");
        eprintln!("     debug_socket = true");
        eprintln!("\nOr use 'jcode debug start' to start a server.");
        eprintln!("Use 'jcode debug list' to see running servers.");
        anyhow::bail!("Debug socket not available");
    }

    let stream = server::connect_socket(&debug_socket).await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let debug_cmd = if arg.is_empty() {
        command.to_string()
    } else {
        format!("{}:{}", command, arg)
    };

    let request = serde_json::json!({
        "type": "debug_command",
        "id": 1,
        "command": debug_cmd,
        "session_id": session_id,
    });

    let mut json = serde_json::to_string(&request)?;
    json.push('\n');
    writer.write_all(json.as_bytes()).await?;

    let mut line = String::new();
    let n = reader.read_line(&mut line).await?;
    if n == 0 {
        anyhow::bail!("Server disconnected before sending response");
    }

    let response: serde_json::Value = serde_json::from_str(&line)?;

    match response.get("type").and_then(|v| v.as_str()) {
        Some("debug_response") => {
            let ok = response
                .get("ok")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let output = response
                .get("output")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if ok {
                println!("{}", output);
            } else {
                eprintln!("Error: {}", output);
                std::process::exit(1);
            }
        }
        Some("error") => {
            let message = response
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            eprintln!("Error: {}", message);
            std::process::exit(1);
        }
        _ => {
            println!("{}", serde_json::to_string_pretty(&response)?);
        }
    }

    Ok(())
}

async fn debug_list_servers() -> Result<()> {
    let mut servers = Vec::new();

    let runtime_dir = crate::storage::runtime_dir();
    let mut scan_dirs = vec![runtime_dir.clone()];
    let temp_dir = std::env::temp_dir();
    if temp_dir != runtime_dir {
        scan_dirs.push(temp_dir);
    }

    for dir in scan_dirs {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str())
                    && name.starts_with("jcode")
                    && name.ends_with(".sock")
                    && !name.contains("-debug")
                {
                    servers.push(path);
                }
            }
        }
    }

    if servers.is_empty() {
        println!("No running jcode servers found.");
        println!("\nStart one with: jcode debug start");
        return Ok(());
    }

    println!("Running jcode servers:\n");

    for socket_path in servers {
        let debug_socket = {
            let filename = socket_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("jcode.sock");
            let debug_filename = filename.replace(".sock", "-debug.sock");
            socket_path.with_file_name(debug_filename)
        };

        let alive = crate::transport::Stream::connect(&socket_path)
            .await
            .is_ok();

        let debug_enabled = if crate::transport::is_socket_path(&debug_socket) {
            crate::transport::Stream::connect(&debug_socket)
                .await
                .is_ok()
        } else {
            false
        };

        let session_info = if debug_enabled {
            get_server_info(&debug_socket).await.unwrap_or_default()
        } else {
            String::new()
        };

        let status = if alive {
            if debug_enabled {
                format!("✓ running, debug: enabled{}", session_info)
            } else {
                "✓ running, debug: disabled".to_string()
            }
        } else {
            "✗ not responding (stale socket?)".to_string()
        };

        println!("  {} ({})", socket_path.display(), status);
    }

    println!("\nUse -s/--socket to target a specific server:");
    println!("  jcode debug -s /path/to/socket.sock sessions");

    Ok(())
}

async fn get_server_info(debug_socket: &std::path::Path) -> Result<String> {
    use crate::transport::Stream;

    let stream = Stream::connect(debug_socket).await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let request = serde_json::json!({
        "type": "debug_command",
        "id": 1,
        "command": "sessions",
    });
    let mut json = serde_json::to_string(&request)?;
    json.push('\n');
    writer.write_all(json.as_bytes()).await?;

    let mut line = String::new();
    let n = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        reader.read_line(&mut line),
    )
    .await??;
    if n == 0 {
        return Ok(String::new());
    }

    let response: serde_json::Value = serde_json::from_str(&line)?;
    if let Some(output) = response.get("output").and_then(|v| v.as_str())
        && let Ok(sessions) = serde_json::from_str::<Vec<String>>(output)
    {
        return Ok(format!(", sessions: {}", sessions.len()));
    }

    Ok(String::new())
}

async fn debug_start_server(arg: &str, socket_path: Option<String>) -> Result<()> {
    let socket = socket_path.unwrap_or_else(|| {
        if !arg.is_empty() {
            arg.to_string()
        } else {
            server::socket_path().to_string_lossy().to_string()
        }
    });

    let socket_pathbuf = std::path::PathBuf::from(&socket);

    if crate::transport::is_socket_path(&socket_pathbuf)
        && crate::transport::Stream::connect(&socket_pathbuf)
            .await
            .is_ok()
    {
        eprintln!("Server already running at {}", socket);
        eprintln!("Use 'jcode debug list' to see all servers.");
        return Ok(());
    }

    let debug_socket = {
        let filename = socket_pathbuf
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("jcode.sock");
        let debug_filename = filename.replace(".sock", "-debug.sock");
        socket_pathbuf.with_file_name(debug_filename)
    };
    let _ = std::fs::remove_file(&debug_socket);

    eprintln!("Starting jcode server...");

    let exe = std::env::current_exe()?;
    let mut cmd = std::process::Command::new(&exe);
    cmd.arg("serve");

    if socket != server::socket_path().to_string_lossy() {
        cmd.arg("--socket").arg(&socket);
    }

    cmd.stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    let start = std::time::Instant::now();
    loop {
        if start.elapsed() > std::time::Duration::from_secs(10) {
            anyhow::bail!("Server failed to start within 10 seconds");
        }
        if crate::transport::is_socket_path(&socket_pathbuf)
            && crate::transport::Stream::connect(&socket_pathbuf)
                .await
                .is_ok()
        {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    eprintln!("✓ Server started at {}", socket);

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    if crate::transport::is_socket_path(&debug_socket) {
        eprintln!("✓ Debug socket at {}", debug_socket.display());
    } else {
        eprintln!("⚠ Debug socket not enabled. Add to ~/.jcode/config.toml:");
        eprintln!("  [display]");
        eprintln!("  debug_socket = true");
    }

    Ok(())
}

#[cfg(test)]
mod instructions_tests {
    use super::*;

    #[test]
    fn terminal_report_names_detected_capabilities_and_fallbacks() {
        let report = terminal_capability_report();
        for field in ["terminal", "color", "image", "mouse", "unicode_width"] {
            assert!(report.get(field).is_some(), "{field}");
        }
        assert!(report["color"].get("fallback").is_some());
        assert!(report["image"].get("fallback").is_some());
        assert!(report["mouse"].get("fallback").is_some());
        assert_eq!(report["unicode_width"]["mode"], "unicode-width");
        assert_eq!(report["unicode_width"]["terminal_probe"], "not performed");
    }

    #[test]
    fn terminal_report_no_color_environment() {
        if std::env::var_os("JCODE_TEST_TERMINAL_REPORT_CHILD").is_none() {
            for key in ["NO_COLOR", "JCODE_NO_COLOR"] {
                let output = std::process::Command::new(std::env::current_exe().unwrap())
                    .args([
                        "--exact",
                        "cli::debug::instructions_tests::terminal_report_no_color_environment",
                        "--nocapture",
                    ])
                    .env_remove("NO_COLOR")
                    .env_remove("JCODE_NO_COLOR")
                    .env("JCODE_TEST_TERMINAL_REPORT_CHILD", "1")
                    .env(key, "1")
                    .output()
                    .unwrap();
                assert!(
                    output.status.success(),
                    "{} {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            return;
        }
        assert_eq!(
            terminal_capability_report()["color"]["fallback"],
            "monochrome (NO_COLOR/JCODE_NO_COLOR)"
        );
    }

    #[test]
    fn terminal_report_explicit_fallback_environment() {
        if std::env::var_os("JCODE_TEST_EXPLICIT_FALLBACK_CHILD").is_none() {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "cli::debug::instructions_tests::terminal_report_explicit_fallback_environment",
                    "--nocapture",
                ])
                .env("JCODE_TEST_EXPLICIT_FALLBACK_CHILD", "1")
                .env("JCODE_ASCII_SAFE", "on")
                .env("JCODE_IMAGES", "off")
                .env("KITTY_WINDOW_ID", "1")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
            return;
        }
        let report = terminal_capability_report();
        assert_eq!(report["ascii_safe"]["enabled"], true);
        assert_eq!(report["ascii_safe"]["content_transformed"], false);
        assert_eq!(report["ascii_safe"]["full_ascii_output"], false);
        assert_eq!(report["image"]["disabled_by_env"], true);
        assert_eq!(report["image"]["detected"], "None");
        assert_eq!(report["unicode_width"]["mode"], "unicode-width");
    }

    #[test]
    fn instructions_lists_project_file() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(dir.path().join("AGENTS.md"), "x").unwrap();
        let report = instructions_report(dir.path());
        assert!(report.contains("loaded"));
        assert!(report.contains("Project Instructions (AGENTS.md)"));
        assert!(run_instructions_command(Some(dir.path())).is_ok());
    }

    #[test]
    fn instructions_rejects_invalid_dir() {
        let err = run_instructions_command(Some(std::path::Path::new("/nonexistent/jcode-x")))
            .unwrap_err();
        assert!(err.to_string().contains("not a directory"));
    }
}
