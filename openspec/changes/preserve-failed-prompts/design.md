# Design

## Context

See [proposal.md](proposal.md) for motivation and [research.md](research.md) for source evidence at the inspected revision. Existing recovery is split between local submission, prepared remote input and asynchronous turn errors. `last_submitted_input` is populated too late for unknown-skill rejection, stores raw placeholders after paste storage is cleared, and is consumed even when a newer composer draft prevents restoration.

This design is included because the fix crosses submission, error presentation, copying and privacy boundaries. The behavioral contract is [failed-prompt-recovery](specs/failed-prompt-recovery/spec.md).

## Goals / Non-Goals

**Goals:** establish one authoritative user-text snapshot for each affected attempt; expose it through error presentation and copying; preserve composer and attachment handling across local and remote paths.

**Non-Goals:** a second editor, a persistent prompt journal, crash recovery, new retry policy, skill installation, additional provider requests or changes to the separate Desktop repository. Existing input-size limits still apply. This proposal does not implement or reload the harness.

## Decisions

### 1. Retain the expanded user text before destructive preparation

Reuse `PreparedInput.expanded` and the existing last-submitted/fallback state rather than add a parallel submission journal. In the local path, capture equivalent expanded text before clearing paste storage or returning from validation. Keep the typed slash invocation in recovery, not the subsequently injected skill body or system reminders.

Snapshot conversational input only after identifying existing secret-interception flows. Do not move a blanket capture ahead of those privacy guards. Size-limit rejection must preserve its draft and expose the same recovery affordance without dispatching it.

Raw composer text is useful only while its paste mappings remain available. Restoration after those mappings are cleared must use the expanded text, or retained equivalent mappings, never a dead placeholder. Preserve still-available attachment state through existing prepared-input/fallback handling; the copy payload is text only. Keep existing explicit image-removal notices, including payload-limit handling.

**Alternative rejected:** moving `last_submitted_input` earlier without changing its raw-text lifetime fixes only some early returns and still loses expanded pastes.

### 2. Attach an inert, full-text block to the failure

Separate the diagnostic error from its user-text recovery payload so the latter is not accidentally sent to the provider or appended to diagnostic logs. Reuse existing error/display machinery and clipboard helpers. The block is not a successful user turn and must not create a second conversational message or trigger model processing.

Conceptual presentation:

````text
Error: Unknown skill: /missing-skill
Not sent.

Failed prompt
```text
/missing-skill Please review this change.
The complete pasted prompt continues here, without truncation.
```

Prompt restored to input. Copy the failed prompt to keep a separate copy.
````

The actual renderer must handle arbitrary embedded fences and terminal escapes as inert text, not assume three backticks are always safe. Normal chat content may wrap. Compact status labels must still fit terminal-cell width per repository rules.

The copy target must reference the authoritative prompt string, not rendered cells or the current viewport. Reuse keyboard copy/selection behavior where it preserves exact text; add only the smallest prompt-scoped target needed to copy the entire block. No new clipboard backend is needed. Copy failure leaves the block available and follows existing failure feedback.

**Alternative rejected:** an error-only toast or a truncated preview with history recall does not meet the explicit complete-copy requirement.

### 3. Keep recovery independent of the composer

An empty composer receives editable recovered text. A newer draft remains byte-for-byte unchanged, while the failed-prompt block stays available. With an occupied composer, copying and later pasting after preserving or clearing the new draft is sufficient; do not add a second editor or silently replace input.

Do not consume the only recovery value merely because composer restoration was skipped. Retain the payload alongside its associated error for the current UI session under normal transcript retention. Deliberately clearing the conversation or ending the process is not a new persistence guarantee. Do not introduce another persistent prompt store, bypass history eligibility or add raw-prompt logging. Existing transcript retention remains authoritative; this feature adds no crash/restart recovery guarantee.

**Alternative rejected:** composer-only restoration couples recovery to whatever the user types next and loses the error-adjacent copy target.

### 4. Reuse attempt identity and existing retry ownership

Use the active local turn and existing remote request/event IDs to associate a failure with its submitted snapshot. Keep this association through pending-send cleanup, provider fallback and existing bounded retries. A delayed error cannot capture the latest composer text or a different submission's fallback value.

Repeated notifications update the same failed-attempt recovery record. Copying or restoring never resends. Known validation/preparation rejection says the prompt was not sent; a transport failure after dispatch says delivery is uncertain and warns that manual resubmission may repeat work. Do not label an ambiguous failure as a safe-to-retry rejection.

No wire-format change is planned because remote error events already carry IDs. No new automatic retry is introduced, and existing retry/failover policy is not disabled or broadened.

## Risks / Trade-offs

- **Larger in-memory text:** retain/reuse the existing expanded payload where possible instead of independently cloning full raw, expanded and rendered copies. Existing submit limits remain unchanged; recovery must not truncate text to fit history limits.
- **Duplicate diagnostics or turns:** key recovery by failed attempt and test repeated/delayed events. Keep its UI block distinct from provider conversation messages.
- **Sensitive input exposure:** exercise login/account/SSH interception with sentinel values and verify their absence from display, copying and additional persistence.
- **Copy/render divergence:** compare the actual copy callback payload with the complete source string, including off-screen content, whitespace, Unicode and fences. Rendering snapshots alone are insufficient.
- **Unknown remote outcome:** make ambiguity visible and preserve user control over manual resubmission. Recovery cannot roll back work already performed by a server.

## Migration Plan

No data migration, dependency change or protocol rollout is required. After a separate implementation request, land failing regression tests, make the scoped shared recovery changes, then verify local and daemon-connected TUI behavior using the repository's selfdev/debug_socket route against an explicitly identified source-built binary. Do not treat a build or an old daemon as runtime proof.

Rollback is a scoped revert of the future implementation while preserving unrelated changes. This planning turn performs no build, install, runtime reload, publication or archive. No material design decisions remain unresolved.
