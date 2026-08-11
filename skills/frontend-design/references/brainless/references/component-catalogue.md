
# Brainless Component Catalogue

> Source: `theswerd/brainless` (MIT license), recon'd 2026-07-16 —
> `docs/recon/theswerd-brainless.md` / `.html`. Point-in-time reference (registry.json at recon
> time); re-verify against upstream before relying on exact names for an install.

40 registry items total: 9 Claude, 8 Codex, 17 Grok, 6 Blocks.

## Install

Three documented install paths (all via `bunx shadcn@latest`, upstream README):

```bash
# 1. Namespace add — registers @brainless once, then install by short name
bunx shadcn@latest registry add @brainless=https://brainless.swerdlow.dev/r/{name}.json
bunx shadcn@latest add @brainless/claude-session

# 2. Direct-URL add — no namespace registration, one-shot install
bunx shadcn@latest add https://brainless.swerdlow.dev/r/claude-session.json

# 3. GitHub add — install straight from the source repo
bunx shadcn@latest add https://github.com/theswerd/brainless/tree/main/registry/brainless/blocks/claude-session
```

Replace `claude-session` / `{name}` with any `name` column value below. Nothing from this
registry is vendored into this repo — every install pulls fresh from upstream at use time.

## Claude

| Name | Title | Description |
| --- | --- | --- |
| `claude-header` | Claude Header | Welcome box — logo, tips, model/cwd lines as a semantic fieldset legend. |
| `claude-message` | Claude Message | Conversation turn — `❯` chip for user, plain text for assistant. |
| `claude-thinking` | Claude Thinking | Working line — pulsing sparkle, rotating verb, as an `aria-live` region. |
| `claude-tool-call` | Claude Tool Call | `⏺`/`⎿` tool line as a real keyboard-operable `<details>`. |
| `claude-todo-list` | Claude Todo List | Update Todos block as a real list, struck-through done style. |
| `claude-diff` | Claude Diff | Inline edit hunk — tinted +/- rows, line numbers, off-screen labels. |
| `claude-permission` | Claude Permission | "Do you want to proceed?" approval prompt as an arrow-key radiogroup. |
| `claude-prompt` | Claude Prompt | Input composer — real text input, effort chips (low/medium/high). |
| `claude-slash-menu` | Claude Slash Menu | Slash-command palette above the real composer, arrow-key filterable. |

## Codex

| Name | Title | Description |
| --- | --- | --- |
| `codex-header` | Codex Header | Monochrome launch card — model/directory rows. |
| `codex-message` | Codex Message | Conversation turn — bold `›` user marker or plain assistant text. |
| `codex-exec` | Codex Exec | Action/exec line — status dot + expandable output as real `<details>`. |
| `codex-working` | Codex Working | Working line — `•` bullet, grayscale shimmer, live elapsed/interrupt hint. |
| `codex-prompt` | Codex Prompt | `›` text input, warm model/green cwd status, Plan-mode tag. |
| `codex-diff` | Codex Diff | Full-screen `/diff` pager — spaced title bar, unified git hunks. |
| `codex-permissions` | Codex Permissions | `/permissions` chooser — numbered options as a real radiogroup. |
| `codex-slash-menu` | Codex Slash Menu | Slash-command palette under the real composer. |

## Grok

| Name | Title | Description |
| --- | --- | --- |
| `grok-header` | Grok Header | Launch card — braille mark as a crisp dot-matrix SVG. |
| `grok-status` | Grok Status | Top status bar — branch glyph, cwd, context usage, turn progress. |
| `grok-message` | Grok Message | Conversation turn — `❯` user marker, optional wall-clock stamp. |
| `grok-event` | Grok Event | `◆` diamond event line — Thought / user_prompt_submit / stop, hooks badge. |
| `grok-thought` | Grok Thought | Completed or streaming thought — bordered thinking body. |
| `grok-thinking` | Grok Thinking | Idle working line — braille spinner + rotating verb, amber, `aria-live`. |
| `grok-working` | Grok Working | In-turn status — elapsed, token count, `[stop]` hint. |
| `grok-tool` | Grok Tool | Action — compact verb+path, or a bordered card with hooks count. |
| `grok-write` | Grok Write | Inline write/edit chrome — line-numbered before/after in a gutter. |
| `grok-permission` | Grok Permission | Left-border approval card — radio options + select hint bar. |
| `grok-plan` | Grok Plan | `plan.md` approval card — framed viewer, a/s/c/q actions. |
| `grok-project-picker` | Grok Project Picker | "Run in this directory?" chooser — recent dirs + free-text. |
| `grok-shortcuts` | Grok Shortcuts | Keyboard Shortcuts modal — nested categories with key bindings. |
| `grok-settings` | Grok Settings | Settings modal — Appearance/Mouse sections, nested rows. |
| `grok-turn-end` | Grok Turn End | Post-turn footer — "Turn completed in Ns". |
| `grok-slash-menu` | Grok Slash Menu | Slash-command overlay above the real composer. |
| `grok-prompt` | Grok Prompt | Rounded CSS-border composer — model/mode legend. |

## Blocks

| Name | Title | Description |
| --- | --- | --- |
| `claude-session` | Claude Session | Complete screen — header, full turn (plan, tool calls, diff, approval, working, composer). |
| `codex-session` | Codex Session | Complete screen — header, full turn (messages, exec lines, working) and composer. |
| `grok-session` | Grok Session | Complete screen — status bar, header, full turn (events, thought, tools, working). |
| `grok-session-active` | Grok Session Active | Mid-turn screen — streaming thought, tool card, write preview. |
| `pied-piper-onboarding` | Pied Piper Onboarding | Interactive sign-up flow easter egg — typed assistant lines, thinking between. |
| `index` | brainless | Default install — Claude, Codex, and Grok session blocks bundled. |

## Accessibility Pattern

The registry's defining trait (recon verdict: verified by reading
`registry/brainless/claude/claude-tool-call.tsx` in full) is that every terminal-chrome visual —
box-drawing glyphs, status colors, disclosure affordances — is reimplemented on top of real
semantic HTML, not CSS/div art. Two illustrative excerpts below reconstruct that structure from
the verified recon description (`docs/recon/theswerd-brainless.md` § Architecture & Key
Patterns); they are **paraphrased sketches of the pattern, not a verbatim copy of upstream
source** (the recon pass described the implementation, it did not capture the literal file
contents) — always pull the real component via the install commands above rather than
hand-copying from here.

### 1. `claude-tool-call` — collapsible tool-call line as a real `<details>`

> Credit: `registry/brainless/claude/claude-tool-call.tsx`, `theswerd/brainless`, MIT license.
> Reconstructed structure, not verbatim upstream source — see caveat above.

```tsx
// Illustrative structure only — install the real component instead of copying this.
<details className="group">
  <summary className="focus-visible:ring flex items-center gap-1 cursor-pointer">
    <span aria-hidden="true">⏺</span>
    <span>{toolLabel}</span>
    <ChevronIcon className="group-open:hidden" aria-hidden="true" />
  </summary>
  <div className="pl-4">
    <span aria-hidden="true">⎿</span>
    <pre>{result}</pre>
  </div>
</details>
```

The collapsed `⏺ tool(arg)` / `⎿ result` line that a real terminal fakes with box-drawing glyphs
and a "ctrl+o to expand" hint becomes a keyboard-operable, screen-reader-announced disclosure:
decorative glyphs get `aria-hidden`, the chevron toggles via `group-open:hidden`, and the summary
carries a `focus-visible:ring`.

### 2. `claude-thinking` — live status region

> Credit: `registry/brainless/claude/claude-thinking.tsx` (referenced in the recon README
> summary), `theswerd/brainless`, MIT license. Reconstructed structure, not verbatim upstream
> source — see caveat above.

```tsx
// Illustrative structure only — install the real component instead of copying this.
<div role="status" aria-live="polite">
  <span aria-hidden="true">✳</span>
  <span>{rotatingVerb}…</span>
</div>
```

The pulsing "thinking" line — a rotating verb next to a decorative sparkle glyph — is exposed to
assistive tech as an `aria-live="polite"` status region so a screen reader announces progress
without the visual spinner needing to be perceivable.
