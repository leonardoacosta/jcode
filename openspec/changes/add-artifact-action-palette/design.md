## Context

Jcode already carries explicit rendered-artifact metadata through live tool completion, persistence, reconnect, replay, and TUI rendering. The current contract recognizes Markdown, Message, and Code cards. Clicked transcript links currently use one generic detached local-open helper and copy the URL as a remote-view fallback. The installed `mopen`, `ropen`, and `iopen` commands provide distinct destinations, while Herald owns speech through one fail-soft pipeline and exposes `say_brief` for explicit spoken digests.

The user approved a contextual palette rather than four global shortcuts or permanent inline badges. They named the options-and-recommendation section a Decision Brief and approved Option 2: Jcode composes; Herald speaks.

This change depends on `add-rendered-artifact-cards`. It must not absorb Herald implementation or bypass Herald's single-speech-path, explicit-only briefing rules.

## Goals / Non-Goals

**Goals:**

- Open one discoverable action palette for the focused rendered artifact or rendered URL.
- Resolve one stable action target without scraping terminal pixels after activation.
- Create a persisted Decision Brief artifact with options, architecture choices, recommendation, and approval or next-step state.
- Create separate spoken prose and deliver it through `say_brief` only after an explicit user action.
- Route explicit open actions through `mopen`, `ropen`, and `iopen` with bounded, fail-soft execution and useful notices.
- Preserve existing click and copy behavior outside the palette.

**Non-Goals:**

- Automatically speaking completions, hooks, schedules, or artifact creation.
- Asking Herald to summarize Markdown or adding another speech transport.
- Replacing repository Markdown side-panel opening, authentication browser flows, or browser-tool automation.
- Adding permanent action badges to every transcript card.
- Making missing optional helpers fatal.

## Decisions

### 1. Use a contextual action palette

A configurable binding with default `Alt+Ctrl+A` opens a palette only when Jcode resolves a focused artifact or link. It contains Brief aloud, Open on Mac, Remote preview, Send to iPhone, and Copy target. Unsupported actions stay visible with a reason.

Rejected: four direct shortcuts consume global keyspace and are less discoverable. Permanent inline buttons add transcript noise and narrow-width complexity.

### 2. Capture a typed action target before opening

The palette receives a snapshot containing the source message identity, optional artifact descriptor and body, optional resolved URL or path, title, and capabilities. It does not re-read screen coordinates after opening. If the source disappears, execution fails closed instead of retargeting silently.

### 3. Add a Decision Brief artifact identity

Extend the artifact kind with `DecisionBrief`. Its body is Markdown and its title defaults to `Decision Brief`. Rendering reuses Markdown card primitives with a distinct identity. Persistence, replay, unknown-kind fallback, and semantic copy follow the existing artifact contract.

A Decision Brief contains context, two or more options with trade-offs, architecture or ownership choices when relevant, one recommendation with rationale, and the approval state or next action.

### 4. Generate paired written and spoken representations

Jcode owns semantic composition because it has the artifact and conversation context. The written representation is compact Markdown. The spoken representation is separate natural prose of 60-150 words ordered as outcome, why it matters, decision points, and next step.

Spoken text contains no Markdown, file paths, identifiers, code, or unrequested measurements. It describes effects rather than mechanisms. Raw Markdown is never sent to Herald.

### 5. Invoke Herald through its existing brief path

Resolve `say_brief` from the client environment. Do not implement synthesis, HTTP service logic, fallback, history, playback, or retry in Jcode. Invocation is foreground-bounded and never backgrounded. Jcode reports accepted, unavailable, or failed-to-launch; Herald history remains authoritative for eventual delivery.

### 6. Route explicit opener actions through installed helpers

Open on Mac executes `mopen <target>`, Remote preview executes `ropen <target>`, Send to iPhone executes `iopen <target>`, and Copy target uses the existing clipboard path. Arguments are passed directly without a shell. Empty, unresolved, unsupported, or unsafe option-like targets are rejected. Missing helpers disable only their action. Ordinary click behavior remains unchanged.

### 7. Keep the feature additive and configurable

The palette binding can be changed or disabled. Existing artifact kinds and serialized sessions remain compatible. External helpers are runtime capabilities, not package dependencies.

## Risks / Trade-offs

- **[Risk] Wrong focus target** → capture semantic target data and fail closed.
- **[Risk] Duplicate speech** → invoke one Herald path once and never retry an ambiguous request.
- **[Risk] Accepted request later fails playback** → report accepted and rely on Herald history.
- **[Risk] Spoken text leaks screen-only material** → validate speech before invocation and retain written Markdown separately.
- **[Risk] Helpers hang** → bounded child execution with concise stderr capture.
- **[Risk] Hotkey conflict** → existing parser, configurable binding, and conflict feedback.

## Migration Plan

1. Land the Decision Brief kind and compatibility tests.
2. Add target resolution and palette UI before external actions.
3. Add opener adapters and Herald delivery behind focused tests.
4. Enable the default binding and run isolated-daemon acceptance.
5. Roll back by disabling the binding and producers; existing Decision Brief bodies remain readable through Markdown or generic fallback.

## Open Questions

None. The user approved the contextual palette and Jcode-composes/Herald-speaks ownership model.
