## Context

See `proposal.md` and `specs/tui-link-opening/spec.md`. The current open tool already resolves `file:` URLs to existing local filesystem paths and dispatches Unix targets through `xdg-open` with `gio` fallback. That path is guarded by the shared browser-suppression predicate, which intentionally suppresses test-harness launches. TUI Markdown currently parses link destinations to render them but exposes the link label as styled text; mouse handling is centralized in the TUI app and currently prioritizes selection and pane interactions.

## Goals / Non-Goals

**Goals:**
- Preserve an association from each rendered link-label cell to its original destination through wrapping and viewport scrolling.
- Use one explicit user gesture to resolve and dispatch a link, while retaining established mouse behavior everywhere else.
- Keep user-click dispatch separate from agent tool execution so no general suppression bypass is introduced.
- Reuse existing URL validation, local file resolution, and platform system opener behavior where possible.

**Non-Goals:**
- Automatically open links during rendering, after tool output, or on message arrival.
- Allow arbitrary URI schemes or remote file authorities.
- Add per-extension application selection, previews, or a new opener dependency.
- Change the generic agent `open` tool's harness suppression.

## Decisions

1. **Carry link hit-target metadata alongside rendered text.** Extend the renderer/layout projection with destination ranges associated with rendered cells, rather than infer the destination from visible text or search raw Markdown at click time. A renderer-level mapping can preserve exact href semantics and can be tested for wrapped labels. Translate mouse coordinates through the same message viewport/scroll offsets used for drawing before resolving a hit.
   - Alternative: parse and search the message text on every click. Rejected because rendered wrapping, formatting, images, and virtualization make source offsets unreliable.

2. **Route only explicit user clicks through a dedicated open entry point.** The click handler invokes a narrow user-initiated opening path, rather than disabling the shared `browser_suppressed` check for the generic tool. The path honors explicit `NO_BROWSER` and `JCODE_NO_BROWSER` values, validates allowed schemes and file URL locality, then calls the existing detached platform opener.
   - Alternative: remove harness suppression globally. Rejected because agent calls and tests could launch real applications.
   - Alternative: allow the generic open tool to bypass suppression. Rejected because the tool can be invoked by the model without a fresh user gesture.

3. **Delegate local file type selection to the desktop.** Convert valid `file:///` URLs to local paths and pass them to the OS opener; do not add an extension allowlist for HTML, Markdown, PNG, SVG, JSON, or future file types. Reject nonlocal/invalid file URLs and missing paths before launching.
   - Alternative: maintain an application-level extension list. Rejected because file associations are OS configuration and the list would be incomplete and stale.

4. **Keep click opening asynchronous and failure-visible.** Dispatch without blocking the TUI event loop; report resolver/spawn errors through the existing user-visible error path and leave the session running. Ignore real opener launching in tests by injecting or substituting the dispatch boundary.
   - Alternative: silently swallow failures. Rejected because a click would appear ineffective with no diagnosis.

## Risks / Trade-offs

- [Hit targets drift from rendered content after resize, scroll, or virtualization] -> Derive ranges from final rendered line layout and test coordinate mapping after wrapping, scrolling, and resize.
- [TUI click path could accidentally inherit broad test suppression bypass] -> Keep the dedicated user-gesture entry point separate from generic agent tool execution; provide a fake opener in tests.
- [Opening a malicious link can invoke a local application] -> Require a direct user click, constrain schemes, honor explicit no-browser opt-outs, and reject nonlocal file authorities.
- [A platform opener may return before the target application fails] -> Report immediate process-start/nonzero-exit failures; document that later desktop-handler failures may not be observable.

## Migration Plan

No data migration is required. Add the behavior behind normal TUI mouse handling. Rollback consists of reverting the implementation; existing `open` tool behavior and its suppression remain unchanged.

## Open Questions

None that change the agreed scope. Keyboard activation and terminal OSC 8 hyperlink integration are out of scope for this proposal.
