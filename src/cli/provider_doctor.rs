//! `jcode provider-doctor` command: a user-facing strict provider/model diagnostic.

use std::io::IsTerminal;

use anyhow::{Context, Result, anyhow};

use crate::live_tests::LiveVerificationStageStatus;
use jcode_provider_doctor::{
    DoctorReport, DoctorTier, NativeProviderKind, native_doctor_supports_provider,
    run_antigravity_native_e2e, run_claude_native_e2e, run_generic_native_e2e, run_provider_e2e,
};

pub async fn run_provider_doctor_command(
    provider: &str,
    model: Option<&str>,
    tier: &str,
    emit_json: bool,
) -> Result<()> {
    let tier: DoctorTier = tier
        .parse()
        .map_err(|message: String| anyhow!("{message}"))?;

    // Native-runtime providers cannot be driven by the OpenAI-compatible doctor;
    // route them to their native drivers, which exercise the production runtime.
    // Claude and Antigravity keep bespoke drivers (unusual credential/catalog
    // stories); everything else flows through the generic native driver.
    if native_doctor_supports_provider(provider) {
        let normalized = crate::auth::lifecycle::normalized_auth_provider_id(Some(provider));
        let report = match normalized {
            Some("claude") => run_claude_native_e2e(provider, model, tier).await?,
            Some("antigravity") => run_antigravity_native_e2e(provider, model, tier).await?,
            Some(other) => {
                let kind = NativeProviderKind::from_normalized(other)
                    .ok_or_else(|| anyhow!("`{provider}` has no native provider-doctor driver"))?;
                run_generic_native_e2e(kind, model, tier).await?
            }
            None => anyhow::bail!("`{provider}` has no native provider-doctor driver"),
        };
        emit_report(&report, emit_json);
        return if report.tier_passed {
            Ok(())
        } else {
            anyhow::bail!("provider-doctor: one or more checks failed for {provider}")
        };
    }

    let profile =
        crate::provider_catalog::openai_compatible_profile_by_id(provider).with_context(|| {
            format!(
                "`{provider}` is not a known OpenAI-compatible provider. \
                 Run `jcode provider-test-coverage` to see provider ids, or check your spelling."
            )
        })?;
    let resolved = crate::provider_catalog::resolve_openai_compatible_profile(profile);

    // Resolve the API key when the tier needs one.
    let api_key = if tier.requires_api_key() {
        let key = crate::provider_catalog::load_api_key_from_env_or_config(
            &resolved.api_key_env,
            &resolved.env_file,
        )
        .with_context(|| {
            format!(
                "no API key found for `{provider}` (looked in env `{}` and `{}`). \
                 Run `jcode login --provider {provider}`, or use `--tier offline` to check wiring only.",
                resolved.api_key_env, resolved.env_file
            )
        })?;
        Some(key)
    } else {
        None
    };

    let report = run_provider_e2e(profile, api_key.as_deref(), model, tier).await?;

    emit_report(&report, emit_json);

    // Non-zero exit when the chosen tier did not fully pass, so scripts/CI can gate on it.
    if report.tier_passed {
        Ok(())
    } else {
        anyhow::bail!("provider-doctor: one or more checks failed for {provider}")
    }
}

fn emit_report(report: &DoctorReport, emit_json: bool) {
    if emit_json {
        println!("{}", report_to_json(report));
    } else {
        let colorize = std::io::stdout().is_terminal()
            && std::env::var_os("NO_COLOR").is_none()
            && std::env::var_os("JCODE_NO_COLOR").is_none();
        print!("{}", format_report(report, colorize));
    }
}

fn status_symbol(status: LiveVerificationStageStatus) -> &'static str {
    match status {
        LiveVerificationStageStatus::Passed => "PASS",
        LiveVerificationStageStatus::Failed => "FAIL",
        LiveVerificationStageStatus::Blocked => "BLOCK",
        LiveVerificationStageStatus::Skipped => "skip",
        LiveVerificationStageStatus::NotRun => "----",
    }
}

fn status_color(status: LiveVerificationStageStatus) -> &'static str {
    match status {
        LiveVerificationStageStatus::Passed => "32", // green
        LiveVerificationStageStatus::Failed | LiveVerificationStageStatus::Blocked => "31", // red
        LiveVerificationStageStatus::Skipped => "90", // dim
        LiveVerificationStageStatus::NotRun => "90",
    }
}

fn format_report(report: &DoctorReport, colorize: bool) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Provider doctor: {} / {}\n",
        report.provider_label,
        if report.model.is_empty() {
            "<no model>"
        } else {
            &report.model
        }
    ));
    out.push_str(&format!("Tier: {} ", report.tier.as_str()));
    out.push_str(match report.tier {
        DoctorTier::Offline => "(no API key, no spend: validates jcode wiring only)\n",
        DoctorTier::Catalog => "(API key, ~no spend: adds live catalog fetch)\n",
        DoctorTier::Full => "(API key, spends balance: chat + streaming + tools)\n",
    });
    out.push_str(
        "Each line is one strict checkpoint. PASS/FAIL exercise it; skip means the\n\
         current tier does not run it (use --tier full for the API-dependent ones).\n\n",
    );

    for check in &report.checks {
        let symbol = status_symbol(check.status);
        let line = format!("  [{symbol:>5}] {:<38} {}\n", check.label, check.detail);
        if colorize {
            let color = status_color(check.status);
            out.push_str(&format!("\x1b[{color}m{line}\x1b[0m"));
        } else {
            out.push_str(&line);
        }
    }

    out.push('\n');
    if report.tier.spends_balance() {
        out.push_str(&format!(
            "Spend this run: {}\n",
            report.spend.human_summary()
        ));
    }
    if report.strict_passed {
        out.push_str("Verdict: READY. Every strict checkpoint passed for this provider/model.\n");
    } else if report.tier_passed {
        out.push_str(&format!(
            "Verdict: tier `{}` passed. Run `--tier full` to confirm full readiness (spends balance).\n",
            report.tier.as_str()
        ));
    } else if let Some(failure) = report.first_failure() {
        out.push_str(&format!(
            "Verdict: FAILED at `{}`.\n  Reason: {}\n",
            failure.label, failure.detail
        ));
        out.push_str(&next_step_hint(failure.checkpoint));
    } else {
        out.push_str("Verdict: FAILED.\n");
    }
    out
}

fn next_step_hint(checkpoint: &str) -> String {
    use crate::live_tests::checkpoints as cp;
    let hint = match checkpoint {
        cp::AUTH_CREDENTIAL_LOADED => {
            "  Next: run `jcode login --provider <provider>` to store a working credential."
        }
        cp::MODEL_CATALOG_LIVE_ENDPOINT => {
            "  Next: the live /models call failed. Check the key, network, and provider status."
        }
        cp::CATALOG_HOT_RELOAD_CURRENT_SESSION
        | cp::PICKER_LIVE_MODELS
        | cp::PICKER_FALLBACK_LABELING
        | cp::MODEL_SWITCH_ROUTE => {
            "  Next: this is a jcode-side routing/picker bug for this provider. \
             Please file an issue with this output."
        }
        cp::NON_STREAMING_CHAT_COMPLETION | cp::STREAMING_CHAT_COMPLETION => {
            "  Next: the model did not return a usable completion. Try another model from the catalog."
        }
        cp::TOOL_CALL_PARSE
        | cp::TOOL_EXECUTION_LOOP
        | cp::TOOL_RESULT_FOLLOWUP
        | cp::REAL_JCODE_TOOL_SMOKE => {
            "  Next: this model did not produce a valid tool call. It may not support tools well."
        }
        _ => "",
    };
    if hint.is_empty() {
        String::new()
    } else {
        format!("{hint}\n")
    }
}

fn report_to_json(report: &DoctorReport) -> String {
    let checks: Vec<serde_json::Value> = report
        .checks
        .iter()
        .map(|check| {
            serde_json::json!({
                "checkpoint": check.checkpoint,
                "label": check.label,
                "status": status_symbol(check.status).to_ascii_lowercase(),
                "detail": check.detail,
            })
        })
        .collect();
    serde_json::to_string_pretty(&serde_json::json!({
        "provider_id": report.provider_id,
        "provider_label": report.provider_label,
        "model": report.model,
        "tier": report.tier.as_str(),
        "tier_passed": report.tier_passed,
        "strict_passed": report.strict_passed,
        "spend": report.spend.to_json(),
        "checks": checks,
    }))
    .unwrap_or_else(|_| "{}".to_string())
}

pub fn run_coverage_command(emit_json: bool) -> Result<()> {
    use std::io::{BufRead, BufReader};
    let path = crate::live_tests::LiveVerificationPaths::resolve()?.events;
    let mut events = Vec::new();
    match std::fs::File::open(&path) {
        Ok(file) => {
            for line in BufReader::new(file).lines() {
                let line = line.context("read provider coverage event")?;
                if !line.trim().is_empty() {
                    events.push(
                        serde_json::from_str(&line).context("parse provider coverage event")?,
                    );
                }
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error).context("open provider coverage events"),
    }
    let report = jcode_provider_doctor::provider_e2e::provider_failure_coverage(&events);
    if emit_json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        let width = crossterm::terminal::size()
            .map(|(w, _)| usize::from(w))
            .unwrap_or(80);
        print!("{}", format_coverage(&report, width));
    }
    Ok(())
}

fn format_coverage(report: &serde_json::Value, width: usize) -> String {
    use unicode_width::UnicodeWidthChar;
    let mut out = String::new();
    if let Some(providers) = report["providers"].as_array() {
        for provider in providers {
            if let Some(scenarios) = provider["scenarios"].as_array() {
                for scenario in scenarios {
                    let status = scenario["status"].as_str().unwrap_or("not-covered");
                    let reason = if status == "skipped" {
                        format!(
                            "({})",
                            scenario["reason"].as_str().unwrap_or("reason not recorded")
                        )
                    } else {
                        String::new()
                    };
                    let line = format!(
                        "{} | {} | {status}{reason}",
                        provider["provider_id"].as_str().unwrap_or("unknown"),
                        scenario["scenario"].as_str().unwrap_or("unknown")
                    );
                    let line = crate::message::strip_ansi_escape_sequences(&line)
                        .replace(char::is_control, " ");
                    let mut used = 0;
                    for ch in line.chars() {
                        let cells = ch.width().unwrap_or(0);
                        if used + cells > width.saturating_sub(1) {
                            if width > 0 {
                                out.push('…');
                            }
                            break;
                        }
                        out.push(ch);
                        used += cells;
                    }
                    out.push('\n');
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn coverage_command_needs_no_provider_and_existing_diagnosis_still_does() {
        assert!(
            crate::cli::args::Args::try_parse_from([
                "jcode",
                "provider-doctor",
                "--coverage",
                "--json"
            ])
            .is_ok()
        );
        assert!(crate::cli::args::Args::try_parse_from(["jcode", "provider-doctor"]).is_err());
        assert!(
            crate::cli::args::Args::try_parse_from([
                "jcode",
                "provider-doctor",
                "openai",
                "--tier",
                "offline"
            ])
            .is_ok()
        );
    }

    #[test]
    fn coverage_table_is_cell_bounded_and_shows_unknown_evidence() {
        let report = jcode_provider_doctor::provider_e2e::provider_failure_coverage(&[]);
        assert!(format_coverage(&report, 80).contains("not-covered"));
        for width in [0, 1, 8, 20, 80] {
            let table = format_coverage(&report, width);
            assert!(
                table
                    .lines()
                    .all(|line| unicode_width::UnicodeWidthStr::width(line) <= width)
            );
        }
    }

    #[test]
    fn coverage_table_strips_escape_sequences_and_controls() {
        let report = serde_json::json!({"providers": [{"provider_id": "provider", "scenarios": [{
            "scenario": "expired_auth", "status": "skipped",
            "reason": "before\u{1b}[31mred\u{1b}[0m\u{9b}2J\u{1b}]0;hidden-title\u{7}after\u{8}\u{7}\u{0}\nnext"
        }]}]});
        for width in [0, 1, 12, 80, 200] {
            let table = format_coverage(&report, width);
            assert!(
                table.chars().all(|ch| ch == '\n' || !ch.is_control()),
                "{table:?}"
            );
            assert!(!table.contains("hidden-title"));
            assert!(!table.contains("[31m"));
            assert!(
                table
                    .lines()
                    .all(|line| unicode_width::UnicodeWidthStr::width(line) <= width)
            );
        }
        assert!(format_coverage(&report, 200).contains("beforeredafter    next"));
    }

    #[test]
    fn coverage_table_preserves_skipped_reason_and_bounds_wide_text() {
        let report = serde_json::json!({"providers": [{"provider_id": "提供者", "scenarios": [{"scenario": "expired_auth", "status": "skipped", "reason": "missing credentials"}]}]});
        assert!(format_coverage(&report, 80).contains("skipped(missing credentials)"));
        for width in [0, 1, 4, 12, 80] {
            assert!(
                format_coverage(&report, width)
                    .lines()
                    .all(|line| unicode_width::UnicodeWidthStr::width(line) <= width)
            );
        }
    }
}
