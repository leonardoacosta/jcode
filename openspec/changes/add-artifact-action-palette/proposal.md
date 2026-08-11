## Why

Rendered artifacts and links are visible in Jcode but do not have a focused action surface. Users must manually copy targets, reconstruct summaries, or invoke external open helpers themselves, and there is no explicit way to turn a decision-oriented artifact into both a readable record and an on-demand Herald briefing.

## What Changes

- Add a contextual artifact action palette, opened from the focused rendered artifact or rendered URL with a configurable `Alt+Ctrl+A` default binding.
- Add actions to create a Decision Brief, open on the Mac with `mopen`, refresh/open remotely with `ropen`, send to iPhone with `iopen`, and copy the resolved target.
- Add an explicit Decision Brief artifact identity that persists a compact Markdown decision record in the transcript.
- Generate separate natural spoken prose for the same brief and send it through Herald's existing explicit-only `say_brief` path.
- Preserve current ordinary click/open behavior and generic artifact fallback when the palette is unavailable, the target is unsupported, or an external helper is missing.

## Capabilities

### New Capabilities
- `artifact-action-palette`: Contextual actions for rendered artifacts and URLs, including persisted Decision Briefs and explicit Herald/mopen/ropen/iopen integration.

### Modified Capabilities

None. This change depends on the active `add-rendered-artifact-cards` change and extends its typed artifact contract without changing the already-authored requirements for Markdown, Message, and Code cards.

## Preconditions

- base-commit: jcode@6a57f8f7a2d026988ff9a289a515fc673028aac1
- The active `add-rendered-artifact-cards` contract and implementation remain available as the semantic artifact foundation.
- `/home/nyaptor/dev/personal/herald` continues to own `say_brief` and its fail-soft single speech pipeline.
- `mopen`, `ropen`, and `iopen` remain optional runtime helpers; their absence cannot block Jcode startup or ordinary transcript use.

## Decisions

- **decided-by: user**: use one contextual action palette rather than direct global shortcuts or permanent inline badges.
- **decided-by: user**: call the rendered options-and-recommendation document a Decision Brief.
- **decided-by: user**: Jcode composes paired written and spoken representations; Herald only speaks the natural prose through `say_brief`.
- **decided-by: default**: default palette binding is configurable `Alt+Ctrl+A`; existing click behavior remains unchanged.

## Impact

- Jcode TUI focus/navigation, hotkey registry, inline-interactive palette, transcript artifact rendering, message/tool metadata, and session restore paths.
- Jcode agent/tool surface for producing paired written and spoken decision briefs.
- External optional integrations: `/home/nyaptor/dev/personal/herald` through `say_brief`, plus installed `mopen`, `ropen`, and `iopen` CLIs.
- No new third-party dependency and no Herald repository code change. Herald remains the sole speech transport owner.

## Done Means

- A focused artifact or rendered URL opens the keyboard-operable palette through the configured binding.
- Decision Brief cards persist and restore with semantic copy behavior.
- Brief aloud persists written Markdown and sends separately validated natural prose through Herald only on explicit selection.
- Explicit `mopen`, `ropen`, and `iopen` actions are bounded and fail softly, while ordinary click behavior is unchanged.
- Focused tests, strict OpenSpec validation, isolated-socket runtime acceptance, and authorized real integration checks pass with no duplicate speech.

## Testing

- Run focused Rust tests for shared artifact types, persistence, TUI rendering/copy, keybindings, palette navigation, target resolution, brief composition, and process adapters; expect zero failures.
- Run `openspec validate add-artifact-action-palette --strict --no-interactive`; expect exit 0.
- Build and run Jcode with an isolated socket, then exercise artifact and URL palette workflows; expect semantic targeting, restore, and missing-helper behavior without affecting the shared daemon.
- After explicit authorization, invoke each real downstream helper once on harmless targets and inspect Herald history; expect one accepted speech attempt and the requested Mac, remote-preview, and iPhone effects.
