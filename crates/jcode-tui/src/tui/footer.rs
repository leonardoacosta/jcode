//! Persistent status footer (Zentui port, roadmap P2).
//!
//! One reserved bottom row answering: where am I (directory, git, execution
//! mode), what am I running (model, provider, effort), and what is it costing
//! me (context, tokens, cost). Pure render state: the row is composed from the
//! per-frame [`InfoWidgetData`] snapshot plus config, and never mutates agent
//! state or probes the filesystem, git, or any subprocess on the render path.
//!
//! Width degradation drops segments in a fixed documented order (session name,
//! cost, tokens, effort, provider, git counts, directory depth), then
//! truncates (branch, context suffix, directory, model). The row never wraps
//! to a second line.

use ratatui::{prelude::*, widgets::Paragraph};
use std::sync::LazyLock;
use unicode_width::UnicodeWidthStr;

use crate::config::{FooterConfig, FooterIconMode, FooterPathDisplay};
use crate::provider::DEFAULT_CONTEXT_LIMIT;

use super::TuiState;
use super::info_widget::{GitInfo, InfoWidgetData, model::shorten_model_name};
use jcode_tui_style::theme::{
    accent_color, dim_color, error_color, info_color, success_color, warning_color,
};

static HOME_DIR: LazyLock<Option<String>> = LazyLock::new(|| {
    std::env::var("HOME")
        .ok()
        .filter(|h| !h.is_empty())
        .or_else(|| std::env::var("USERPROFILE").ok().filter(|h| !h.is_empty()))
});

/// Draw the status footer into `area` (expected to be the reserved footer row).
///
/// Reads the footer config and remote mode from `app` and everything else from
/// the already-assembled per-frame `data` snapshot.
pub(crate) fn draw_footer(frame: &mut Frame, app: &dyn TuiState, area: Rect, data: &InfoWidgetData) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let cfg = app.footer_config();
    if !cfg.enabled() {
        return;
    }
    let spans = footer_line(data, app.is_remote_mode(), &cfg, area.width, HOME_DIR.as_deref());
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

/// Mutable per-frame fields the shrink/truncate steps operate on. All content
/// is pre-colored at render time from a fixed field->role mapping, so a
/// shrink step never changes another field's style.
#[derive(Debug, Default)]
struct FooterFields {
    /// Working-directory display text (current degradation level).
    cwd: Option<String>,
    /// Full path retained so the depth shrink step can recompute.
    cwd_path: Option<String>,
    /// 0 = full, 1 = depth, 2 = basename (most degraded).
    cwd_level: u8,
    mode: Option<&'static str>,
    branch: Option<String>,
    /// Dirty indicators, e.g. "!2 +1 ?3".
    dirty: Option<String>,
    /// Ahead/behind counts, e.g. "↑1 ↓2".
    counts: Option<String>,
    session_name: Option<String>,
    model: Option<String>,
    provider: Option<String>,
    effort: Option<String>,
    /// Context percentage text, e.g. "42%" or "~42%" (stale).
    context: Option<String>,
    /// Long suffix, e.g. "/131k".
    context_suffix: Option<String>,
    /// Context severity: 0 normal, 1 warning, 2 error.
    context_severity: u8,
    tokens: Option<String>,
    cost: Option<String>,
}

/// Build the footer row as styled spans. Pure: identical inputs produce
/// byte-identical output.
fn footer_line(
    data: &InfoWidgetData,
    is_remote: bool,
    cfg: &FooterConfig,
    width: u16,
    home: Option<&str>,
) -> Vec<Span<'static>> {
    let ascii = matches!(cfg.icon_mode, FooterIconMode::Ascii);
    let mut fields = build_fields(data, is_remote, cfg, home);

    let fits = |fields: &FooterFields| -> bool {
        let (left, right) = render_zones(fields, ascii);
        let lw = spans_width(&left);
        let rw = spans_width(&right);
        let gap = if lw > 0 && rw > 0 { 1 } else { 0 };
        lw + gap + rw <= width as usize
    };

    // Fixed drop/shrink order (spec: session name, cost, tokens, effort,
    // provider/upstream extras, directory depth, git counts), then truncation
    // of branch, context suffix, directory, and finally the model label.
    let mut step = 0u8;
    while !fits(&fields) {
        let changed = match step {
            0 => take(&mut fields.session_name),
            1 => take(&mut fields.cost),
            2 => take(&mut fields.tokens),
            3 => take(&mut fields.effort),
            4 => take(&mut fields.provider),
            5 => shrink_cwd(&mut fields, cfg, home),
            6 => take(&mut fields.counts),
            7 => truncate_opt(&mut fields.branch, 10),
            8 => take(&mut fields.context_suffix),
            9 => truncate_opt(&mut fields.cwd, 8),
            10 => truncate_opt(&mut fields.model, 8),
            _ => false,
        };
        step += 1;
        if step > 10 && !changed {
            break;
        }
    }

    let (mut left, mut right) = render_zones(&fields, ascii);
    let width = width as usize;

    if !fits(&fields) {
        // Last-resort truncation so the one-row guarantee holds: shrink the
        // right zone if it alone overflows, then the left zone to the
        // remaining budget.
        let mut rw = spans_width(&right);
        if rw > width {
            truncate_spans(&mut right, width);
            rw = spans_width(&right);
        }
        let left_budget = width.saturating_sub(if rw > 0 { rw + 1 } else { 0 });
        if spans_width(&left) > left_budget {
            if left_budget == 0 {
                left.clear();
            } else {
                truncate_spans(&mut left, left_budget);
            }
        }
    }

    join_zones(left, right, width)
}

/// Assemble the initial fields from the info snapshot.
fn build_fields(
    data: &InfoWidgetData,
    is_remote: bool,
    cfg: &FooterConfig,
    home: Option<&str>,
) -> FooterFields {
    let seg = &cfg.segments;
    let ascii = matches!(cfg.icon_mode, FooterIconMode::Ascii);
    let mut fields = FooterFields::default();

    if seg.cwd {
        if let Some(dir) = data.working_dir.as_deref() {
            fields.cwd_level = match cfg.path_display {
                FooterPathDisplay::Full => 0,
                FooterPathDisplay::Depth => 1,
                FooterPathDisplay::Basename => 2,
            };
            fields.cwd_path = Some(dir.to_string());
            fields.cwd = Some(display_path(dir, fields.cwd_level, cfg, home));
        }
    }

    if seg.mode {
        fields.mode = Some(if is_remote { "remote" } else { "local" });
    }

    if seg.git {
        if let Some(git) = data.git_info.as_ref() {
            fields.branch = Some(git.branch.clone());
            let dirty = git_dirty_text(git);
            if !dirty.is_empty() {
                fields.dirty = Some(dirty);
            }
            let counts = git_counts_text(git, ascii);
            if !counts.is_empty() {
                fields.counts = Some(counts);
            }
        }
    }

    if seg.session_name {
        fields.session_name = data.session_name.clone();
    }

    if seg.model {
        fields.model = data.model.as_deref().map(shorten_model_name);
    }

    if seg.provider {
        let provider = data.provider_name.clone();
        fields.provider = match (provider, data.upstream_provider.as_ref()) {
            (Some(base), Some(upstream)) if !upstream.is_empty() => {
                Some(format!("{base}/{upstream}"))
            }
            (Some(base), _) => Some(base),
            (None, Some(upstream)) if !upstream.is_empty() => Some(upstream.clone()),
            (None, _) => None,
        };
    }

    if seg.effort {
        fields.effort = data.reasoning_effort.clone();
    }

    if seg.context {
        let empty_info = data
            .context_info
            .as_ref()
            .map(|info| info.total_chars == 0)
            .unwrap_or(true);
        let used = data.observed_context_tokens.map(|t| t as usize).or_else(|| {
            if empty_info {
                None
            } else {
                data.context_info.as_ref().map(|info| info.estimated_tokens())
            }
        });
        if let Some(used) = used {
            let limit = data.context_limit.unwrap_or(DEFAULT_CONTEXT_LIMIT).max(1);
            let percent = ((used as u64) * 100 / (limit as u64)).min(999) as u32;
            let stale = data.context_info_stale;
            fields.context = Some(format!("{}{}%", if stale { "~" } else { "" }, percent));
            fields.context_suffix = Some(format!("/{}", kfmt(limit as u64)));
            fields.context_severity = if percent >= cfg.context_error {
                2
            } else if percent >= cfg.context_warning {
                1
            } else {
                0
            };
        }
    }

    if seg.tokens {
        if let Some(usage) = data.usage_info.as_ref() {
            let (input, output) = (usage.input_tokens, usage.output_tokens);
            if usage.available && input + output > 0 {
                fields.tokens = Some(format!("{}/{} tok", kfmt(input), kfmt(output)));
            }
        }
    }

    if seg.cost {
        if let Some(usage) = data.usage_info.as_ref() {
            if usage.available && usage.total_cost >= 0.005 {
                fields.cost = Some(format!("${:.2}", usage.total_cost));
            }
        }
    }

    fields
}

/// Render the two zones (uncolored join happens later). Left zone fields use
/// single-space separators; right zone fields use " · " (auto) or " | "
/// (ascii).
fn render_zones(fields: &FooterFields, ascii: bool) -> (Vec<Span<'static>>, Vec<Span<'static>>) {
    let dim = Style::default().fg(dim_color());
    let mut left: Vec<Span<'static>> = Vec::new();
    let push_left = |text: &str, style: Style, left: &mut Vec<Span<'static>>| {
        if !text.is_empty() {
            if !left.is_empty() {
                left.push(Span::styled(" ", dim));
            }
            left.push(Span::styled(text.to_string(), style));
        }
    };
    if let Some(cwd) = &fields.cwd {
        let text = match fields.mode {
            Some(mode) => format!("{cwd} ({mode})"),
            None => cwd.clone(),
        };
        push_left(&text, Style::default().fg(info_color()), &mut left);
    } else if let Some(mode) = fields.mode {
        push_left(mode, dim, &mut left);
    }
    if let Some(branch) = &fields.branch {
        push_left(
            branch,
            Style::default().fg(accent_color()).add_modifier(Modifier::BOLD),
            &mut left,
        );
    }
    if let Some(dirty) = &fields.dirty {
        push_left(dirty, Style::default().fg(warning_color()), &mut left);
    }
    if let Some(counts) = &fields.counts {
        push_left(counts, dim, &mut left);
    }
    if let Some(name) = &fields.session_name {
        push_left(name, dim, &mut left);
    }

    let sep = if ascii { " | " } else { " · " };
    let mut right: Vec<Span<'static>> = Vec::new();
    let push_right = |text: &str, style: Style, right: &mut Vec<Span<'static>>| {
        if !text.is_empty() {
            if !right.is_empty() {
                right.push(Span::styled(sep, dim));
            }
            right.push(Span::styled(text.to_string(), style));
        }
    };
    if let Some(model) = &fields.model {
        push_right(model, Style::default().fg(accent_color()), &mut right);
    }
    if let Some(provider) = &fields.provider {
        push_right(provider, dim, &mut right);
    }
    if let Some(effort) = &fields.effort {
        push_right(effort, dim, &mut right);
    }
    if let Some(context) = &fields.context {
        let color = match fields.context_severity {
            2 => error_color(),
            1 => warning_color(),
            _ => dim_color(),
        };
        let text = match &fields.context_suffix {
            Some(suffix) => format!("{context}{suffix}"),
            None => context.clone(),
        };
        push_right(&text, Style::default().fg(color), &mut right);
    }
    if let Some(tokens) = &fields.tokens {
        push_right(tokens, dim, &mut right);
    }
    if let Some(cost) = &fields.cost {
        push_right(cost, Style::default().fg(success_color()), &mut right);
    }

    (left, right)
}

/// Join left and right zones with the flexible gap.
fn join_zones(
    mut left: Vec<Span<'static>>,
    right: Vec<Span<'static>>,
    width: usize,
) -> Vec<Span<'static>> {
    let lw = spans_width(&left);
    let rw = spans_width(&right);
    if lw > 0 && rw > 0 {
        let gap = width.saturating_sub(lw + rw);
        if gap > 0 {
            left.push(Span::raw(" ".repeat(gap)));
        }
    }
    left.extend(right);
    left
}

fn spans_width(spans: &[Span<'static>]) -> usize {
    spans.iter().map(|span| UnicodeWidthStr::width(span.content.as_ref())).sum()
}

/// Truncate a span list to a display-width budget, appending an ellipsis.
fn truncate_spans(spans: &mut Vec<Span<'static>>, budget: usize) {
    if budget == 0 {
        spans.clear();
        return;
    }
    let limit = budget - 1; // reserve one column for the ellipsis
    let mut used = 0usize;
    let mut cut: Option<(usize, String, Style)> = None;
    for (idx, span) in spans.iter().enumerate() {
        let w = UnicodeWidthStr::width(span.content.as_ref());
        if used + w > limit {
            let mut partial = String::new();
            for ch in span.content.chars() {
                let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
                if used + cw > limit {
                    break;
                }
                used += cw;
                partial.push(ch);
            }
            cut = Some((idx, partial, span.style));
            break;
        }
        used += w;
    }
    if let Some((idx, partial, style)) = cut {
        spans.truncate(idx);
        if !partial.is_empty() {
            spans.push(Span::styled(partial, style));
        }
        spans.push(Span::raw("…"));
    }
}

fn take(opt: &mut Option<String>) -> bool {
    opt.take().is_some()
}

/// Directory depth degradation: full -> depth -> basename.
fn shrink_cwd(fields: &mut FooterFields, cfg: &FooterConfig, home: Option<&str>) -> bool {
    if fields.cwd_level >= 2 {
        return false;
    }
    fields.cwd_level += 1;
    if let Some(path) = fields.cwd_path.clone() {
        fields.cwd = Some(display_path(&path, fields.cwd_level, cfg, home));
    }
    true
}

fn truncate_opt(opt: &mut Option<String>, max_chars: usize) -> bool {
    let Some(text) = opt else {
        return false;
    };
    if text.chars().count() <= max_chars {
        return false;
    }
    let mut truncated: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    truncated.push('…');
    *opt = Some(truncated);
    true
}

/// Format the working directory for display at a degradation level.
fn display_path(path: &str, level: u8, cfg: &FooterConfig, home: Option<&str>) -> String {
    let collapsed = collapse_home(path, home);
    match level {
        0 => collapsed,
        2 => last_components(&collapsed, 1),
        _ => last_components(&collapsed, cfg.path_depth.max(1)),
    }
}

fn collapse_home(path: &str, home: Option<&str>) -> String {
    let Some(home) = home else {
        return path.to_string();
    };
    if path == home {
        "~".to_string()
    } else if let Some(rest) = path.strip_prefix(home) {
        if let Some(stripped) = rest.strip_prefix('/') {
            format!("~/{stripped}")
        } else {
            path.to_string()
        }
    } else {
        path.to_string()
    }
}

/// Keep the last `depth` path components, prefixing an ellipsis marker when
/// components were dropped.
fn last_components(path: &str, depth: usize) -> String {
    let is_rooted = path.starts_with('/');
    let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return path.to_string();
    }
    if parts.len() <= depth {
        let joined = parts.join("/");
        return if is_rooted { format!("/{joined}") } else { joined };
    }
    let tail: Vec<&str> = parts[parts.len() - depth..].to_vec();
    if depth == 1 {
        tail.join("/")
    } else {
        format!("…/{}", tail.join("/"))
    }
}

fn git_dirty_text(git: &GitInfo) -> String {
    let mut parts: Vec<String> = Vec::new();
    if git.modified > 0 {
        parts.push(format!("!{}", git.modified));
    }
    if git.staged > 0 {
        parts.push(format!("+{}", git.staged));
    }
    if git.untracked > 0 {
        parts.push(format!("?{}", git.untracked));
    }
    parts.join(" ")
}

fn git_counts_text(git: &GitInfo, ascii: bool) -> String {
    let mut parts: Vec<String> = Vec::new();
    let (up, down) = if ascii { ("^", "v") } else { ("↑", "↓") };
    if git.ahead > 0 {
        parts.push(format!("{up}{}", git.ahead));
    }
    if git.behind > 0 {
        parts.push(format!("{down}{}", git.behind));
    }
    parts.join(" ")
}

/// Compact magnitude formatting: 999, 9.9k, 42k, 1.2M.
fn kfmt(value: u64) -> String {
    if value < 1_000 {
        value.to_string()
    } else if value < 10_000 {
        format!("{:.1}k", value as f64 / 1_000.0)
    } else if value < 1_000_000 {
        format!("{}k", value / 1_000)
    } else {
        format!("{:.1}M", value as f64 / 1_000_000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::info_widget::{UsageInfo, UsageProvider};

    fn line_text(spans: &[Span<'static>]) -> String {
        spans.iter().map(|s| s.content.as_ref()).collect()
    }

    fn base_data() -> InfoWidgetData {
        InfoWidgetData {
            working_dir: Some("/home/user/dev/jcode".to_string()),
            model: Some("claude-fable-5".to_string()),
            provider_name: Some("anthropic".to_string()),
            reasoning_effort: Some("high".to_string()),
            git_info: Some(GitInfo {
                branch: "main".to_string(),
                modified: 2,
                staged: 0,
                untracked: 1,
                ahead: 3,
                behind: 0,
                dirty_files: Vec::new(),
            }),
            context_limit: Some(200_000),
            observed_context_tokens: Some(80_000),
            usage_info: Some(UsageInfo {
                provider: UsageProvider::CostBased,
                primary_limit_label: None,
                five_hour: 0.0,
                five_hour_resets_at: None,
                secondary_limit_label: None,
                seven_day: 0.0,
                seven_day_resets_at: None,
                spark: None,
                spark_resets_at: None,
                total_cost: 1.23,
                input_tokens: 12_300,
                output_tokens: 45_600,
                cache_read_tokens: None,
                cache_write_tokens: None,
                output_tps: None,
                available: true,
            }),
            ..Default::default()
        }
    }

    fn cfg() -> FooterConfig {
        FooterConfig::default()
    }

    #[test]
    fn full_render_at_wide_width() {
        let data = base_data();
        let spans = footer_line(&data, false, &cfg(), 120, Some("/home/user"));
        let text = line_text(&spans);
        assert!(text.contains("jcode"), "cwd basename: {text}");
        assert!(text.contains("(local)"), "mode marker: {text}");
        assert!(text.contains("main"), "branch: {text}");
        assert!(text.contains("!2"), "modified: {text}");
        assert!(text.contains("?1"), "untracked: {text}");
        assert!(text.contains("↑3"), "ahead: {text}");
        assert!(text.contains("fable"), "model: {text}");
        assert!(text.contains("anthropic"), "provider: {text}");
        assert!(text.contains("high"), "effort: {text}");
        assert!(text.contains("40%"), "context percent: {text}");
        assert!(text.contains("$1.23"), "cost: {text}");
        assert!(!text.contains('\n'));
    }

    #[test]
    fn remote_mode_shows_remote_marker() {
        let data = base_data();
        let spans = footer_line(&data, true, &cfg(), 120, Some("/home/user"));
        assert!(line_text(&spans).contains("(remote)"));
    }

    #[test]
    fn missing_git_omits_segment_without_separator_drift() {
        let mut data = base_data();
        data.git_info = None;
        let spans = footer_line(&data, false, &cfg(), 120, Some("/home/user"));
        let text = line_text(&spans);
        assert!(!text.contains("main"));
        assert!(!text.contains("!2"));
        assert!(text.contains("jcode (local)"));
    }

    #[test]
    fn zero_cost_and_tokens_omit_segments() {
        let mut data = base_data();
        if let Some(usage) = data.usage_info.as_mut() {
            usage.total_cost = 0.0;
            usage.input_tokens = 0;
            usage.output_tokens = 0;
        }
        let spans = footer_line(&data, false, &cfg(), 120, Some("/home/user"));
        let text = line_text(&spans);
        assert!(!text.contains('$'), "no zero-cost placeholder: {text}");
        assert!(!text.contains("tok"), "no zero-token placeholder: {text}");
    }

    #[test]
    fn narrow_width_drops_in_documented_order() {
        let mut data = base_data();
        data.session_name = Some("fox".to_string());
        let mut cfg_with_name = cfg();
        cfg_with_name.segments.session_name = true;

        let wide = line_text(&footer_line(&data, false, &cfg_with_name, 200, Some("/home/user")));
        assert!(wide.contains("fox"), "name at wide width: {wide}");
        assert!(wide.contains("$1.23"), "cost at wide width: {wide}");

        // Progressively narrower widths must shed name before cost, cost
        // before tokens, tokens before effort. Width 84 forces the name and
        // cost drops; 56 additionally sheds tokens and effort.
        let narrower = line_text(&footer_line(&data, false, &cfg_with_name, 84, Some("/home/user")));
        assert!(!narrower.contains("fox"), "name drops first: {narrower}");
        assert!(!narrower.contains("$1.23"), "cost drops second: {narrower}");
        assert!(narrower.contains("tok"), "tokens survive at 84: {narrower}");
        assert!(narrower.contains("high"), "effort survives at 84: {narrower}");
        assert!(narrower.contains("main"), "branch survives: {narrower}");

        let tight = line_text(&footer_line(&data, false, &cfg_with_name, 56, Some("/home/user")));
        assert!(!tight.contains("tok"), "tokens drop before effort: {tight}");
        assert!(!tight.contains("high"), "effort drops fourth: {tight}");
        assert!(tight.contains("main"), "branch kept at 56: {tight}");
        assert!(tight.contains("jcode"), "cwd kept at 56: {tight}");
        assert!(!tight.contains('\n'));
        assert!(UnicodeWidthStr::width(tight.trim_end()) <= 56);
    }

    #[test]
    fn one_row_guarantee_at_tiny_width() {
        let data = base_data();
        for w in [10u16, 20, 30, 40] {
            let spans = footer_line(&data, false, &cfg(), w, Some("/home/user"));
            let text = line_text(&spans);
            assert!(!text.contains('\n'));
            assert!(
                UnicodeWidthStr::width(text.trim_end()) <= w as usize,
                "width {w}: {text:?}"
            );
        }
    }

    #[test]
    fn ascii_mode_uses_ascii_glyphs() {
        let mut ascii_cfg = cfg();
        ascii_cfg.icon_mode = FooterIconMode::Ascii;
        let data = base_data();
        let spans = footer_line(&data, false, &ascii_cfg, 120, Some("/home/user"));
        let text = line_text(&spans);
        assert!(text.contains("^3"), "ascii ahead marker: {text}");
        assert!(!text.contains('↑'), "no unicode arrow: {text}");
        assert!(!text.contains('·'), "no middot separator: {text}");
    }

    #[test]
    fn home_collapses_in_full_and_depth_modes() {
        let mut full_cfg = cfg();
        full_cfg.path_display = FooterPathDisplay::Full;
        let data = base_data();
        let spans = footer_line(&data, false, &full_cfg, 200, Some("/home/user"));
        assert!(line_text(&spans).contains("~/dev/jcode"));

        let mut depth_cfg = cfg();
        depth_cfg.path_display = FooterPathDisplay::Depth;
        let spans = footer_line(&data, false, &depth_cfg, 200, Some("/home/user"));
        assert!(line_text(&spans).contains("dev/jcode"));
    }

    #[test]
    fn context_thresholds_set_severity() {
        let mut data = base_data();
        data.observed_context_tokens = Some(190_000);
        let spans = footer_line(&data, false, &cfg(), 120, Some("/home/user"));
        let context_span = spans
            .iter()
            .find(|s| s.content.contains("95%"))
            .expect("context segment present");
        assert_eq!(context_span.style.fg, Some(error_color()));
    }

    #[test]
    fn stale_context_gets_marker() {
        let mut data = base_data();
        data.context_info_stale = true;
        let spans = footer_line(&data, false, &cfg(), 120, Some("/home/user"));
        assert!(line_text(&spans).contains("~40%"));
    }

    #[test]
    fn deterministic_repeated_render() {
        let data = base_data();
        let first = footer_line(&data, false, &cfg(), 80, Some("/home/user"));
        let second = footer_line(&data, false, &cfg(), 80, Some("/home/user"));
        assert_eq!(line_text(&first), line_text(&second));
    }
}
