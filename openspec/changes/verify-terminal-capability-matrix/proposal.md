---
execution:
  version: 1
  depends_on: []
---
# Proposal

Roadmap owner: [P10: Distribution/platform support/performance/diagnostics](../../../docs/plans/PRODUCT_ROADMAP.md#p10-distributionplatform-supportperformancediagnostics)

## Why

Terminal capability detection is scattered: rounded pill caps (`crates/jcode-tui/src/tui/ui_onboarding.rs:44`), image protocols in `crates/jcode-terminal-image`, Windows VT mouse handling (`src/cli/terminal.rs:28`). No report states which capabilities were detected or which fallback is active, so platform bugs cannot be triaged from user evidence.

## What Changes

- `jcode debug terminal` prints detected terminal, color depth, image protocol, mouse, Unicode width mode and the chosen fallback for each.
- A test matrix asserts fallbacks when each capability is forced off.

## Non-scope

New protocols, changes to default detection heuristics, global rewriting of user/chat/tool content.

## Capabilities

### New Capabilities

- `terminal-capability-acceptance`: report detected terminal capabilities and active fallbacks.

## Impact

New read-only CLI subcommand calling existing detectors.

## Accepted decision, 2026-11-10

The user approved explicit environment-only image-off and ASCII-safe controls.
No config keys, automatic fallback heuristics or changed defaults. Existing
`JCODE_GLYPH_SAFE_MODE` remains color quantization, not an ASCII switch.

- `JCODE_IMAGES=off` overrides every image protocol and suppresses direct output
  and TUI previews, including cached image rendering. `off|false|0|no` disable;
  unset, invalid or positive values retain existing detection.
- `JCODE_ASCII_SAFE=on` selects ASCII on application-owned generated chrome,
  without rewriting user/chat/tool text, paths, URLs or supplied titles/labels.
  `on|true|1|yes` enable; unset, invalid or negative values retain defaults.
  Both controls ignore case and surrounding whitespace.
- The current bounded ASCII implementation covers shared rounded-box and rail
  borders, shared truncation markers, primary activity spinners and onboarding
  pill caps. Other Unicode chrome remains. This subset does **not** satisfy full
  forced-Unicode-off acceptance. The report and docs must expose that gap.
- The coordinator owns final builds/private acceptance, independent review,
  commits and archive. Partial worker verification does not close native tasks.
