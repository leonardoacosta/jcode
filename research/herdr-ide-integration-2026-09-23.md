# Herdr integration with VS Code and Cursor

Date: 2026-09-23. Status: exploration only, not an approved implementation plan.

## Recommendation

Build a thin VS Code extension, distributed as a VSIX (Open VSX is a registry), that contributes a Herdr terminal profile and a native session tree. Keep Herdr as the sole process owner. First validate existing direct terminal attach in native IDE terminals. Do not start with a webview terminal, an IDE fork, or a replacement runtime. Settings plus a wrapper are a useful prototype and fallback, but settings alone cannot supply the requested reconciliation and sidebar.

High confidence in the architectural fit. Actual VS Code/Cursor terminal fidelity and lifecycle behavior have not been tested in this exploration.

## Intent and constraints

- First New Herdr Terminal finds or creates a Herdr workspace for the project directory.
- Each subsequent New creates a distinct Herdr tab with a shell pane, displayed as an ordinary IDE terminal.
- Closing a terminal view or exiting the IDE detaches the client. It must not terminate the shell or its children.
- Existing detached sessions must remain discoverable and reattachable.
- A sidebar may display authoritative Herdr metadata and offer explicit management.
- This is live-process persistence across client loss, not merely restoring labels, screen contents, command history, or agent conversation IDs.
- Host reboot and Herdr-server failure are different guarantees. Ordinary processes cannot keep running through a reboot. Do not describe reconstruction or agent resume as uninterrupted execution.

## Current local evidence

Inspected `/home/nyaptor/dev/herdr`, Rust project, HEAD `c3221903`. Package version is 0.8.0, but do not use that label as a capability guarantee for released builds. Installed help exposes the terminal session commands below. No repository changes or live pane mutations were made. Git status was clean before and after inspection. No IDE end-to-end test was performed and no existing tests were executed.

- `src/client/mod.rs:909-927`: `herdr terminal attach <terminal_id> [--takeover]` is a direct interactive attachment, without full Herdr workspace UI. This path explicitly rejects native Windows.
- `src/client/mod.rs:930-1185`: `herdr terminal session control <target>` accepts JSON-line input/resize/scroll/release messages and emits JSON envelopes containing base64 ANSI frames. `observe` supplies read-only viewing. Control stdin EOF sends Detach. This CLI encapsulates a private client-socket handshake. Consumers should not duplicate the binary protocol.
- `src/server/headless.rs:2773-2876`: a terminal has one direct controller. Another controller is rejected unless takeover is requested. Attaching sets resize ownership.
- `src/server/headless.rs:6525-6785`: source tests characterize multiple observers, pane-ID resolution, controller ownership, takeover and release. Reading test code is not a fresh passing test result.
- `src/server/render_stream.rs:65-109`: TerminalAnsi is generated from FrameData with a screen diff encoder. It is not a raw original PTY-output stream. This matters for scrollback, OSC shell integration, terminal accessibility and command extraction.
- CLI `workspace create` returns a workspace, initial tab, and root pane. Do not create an additional tab for the first terminal.
- CLI supports workspace/tab list, create, get, rename, focus and close. No atomic project-key workspace ensure command was found in inspected help/search. Avoid assuming a list-then-create sequence is concurrency-safe.
- `src/api/schema/panes.rs:439-503`: PaneInfo exposes pane/terminal/workspace/tab IDs, cwd, foreground cwd, labels, agent status, titles, display agent, state labels, metadata tokens, activity, agent session and revision. Activity includes overnight_active and outstanding_agent_count.
- `src/api/schema.rs`: `events.subscribe` and metadata reporting exist. A sidebar should consume server facts through public JSON snapshots/events rather than scraping a TUI.
- Repository AGENTS.md says shared facts belong in the server/API, presentation belongs in clients, and new integrations must not deepen private TUI-socket coupling. Prefer the existing CLI encapsulation for the initial terminal transport, and stabilize a neutral public terminal contract if it needs extension.

### Adjacent prior art and memories

No relevant durable memory was returned for Herdr/VS Code persistence. Local `herdr-jcode` README documents lifecycle hooks already reporting to Herdr. `herdr-sidebar-config` documents a workspace/tab/agent hierarchy driven by snapshots and metadata. `herdr-radar` documents project grouping and richer attention state. These READMEs show reusable concepts, not proof of runtime compatibility or code quality. Do not install these plugins or duplicate their reporting logic merely to build the IDE sidebar. No OpenSpec directory was located in the bounded Herdr tree inspection, and no existing VS Code extension was found in the searched Herdr source/docs.

## Context7 findings

Resolved Visual Studio Code Extension API to `/websites/code_visualstudio_api`. Queried terminal profile and pseudoterminal lifecycle documentation. The generated Context7 summary includes a stale-looking provider return-type example. The directly scraped official API reference is more precise: `provideTerminalProfile` returns a `TerminalProfile`, constructed with `TerminalOptions` or `ExtensionTerminalOptions`.

- TerminalOptions permits launching an executable with arguments in the native integrated terminal.
- ExtensionTerminalOptions permits supplying a Pseudoterminal while retaining the native terminal UI.
- Pseudoterminal has input and resize handling, output events, and a close callback for user closure. Implement close as attachment release, never pane termination.
- isTransient can opt out of IDE terminal persistence. Choose one restoration owner rather than allowing both IDE revival and extension restoration to create duplicate attachments.
- Shell integration is a separate contract, including trusted sequence/nonces. A native-looking terminal is not proof that shell integration works.

## Alternatives

| Approach | Fit | Main limitation |
| --- | --- | --- |
| IDE persistent-session settings only | Insufficient | Full restart may relaunch a shell rather than preserve its process |
| Profile that runs bare Herdr | Durable runtime but wrong UX | Full nested Herdr UI, not one independently mapped shell per IDE terminal |
| Profile plus small creation/attachment wrapper | Useful prototype/fallback | Reattach discovery, mapping, concurrency, cancellation and metadata require code anyway |
| Thin extension plus existing direct attach | Recommended first implementation candidate | Unix-only direct path and terminal fidelity need actual testing |
| Thin extension plus JSON control/Pseudoterminal | Promising second transport | Extension owns framing, decoding, resize, flow control, reconnect and terminal-mode details |
| Webview terminal or IDE fork | Not recommended | Duplicates rendering/accessibility/selection work and loses native terminal ergonomics |
| tmux-backed existing extension | Immediate alternative if Herdr is optional | Does not reuse Herdr workspaces, metadata or controller model |

An extension and a terminal profile are complementary. The extension contributes the profile and adds management. No global New Terminal override is required. The user can explicitly select Herdr as the default interactive profile after validation. Tasks, debug consoles and Cursor-owned command terminals need not share that default.

## Proposed interaction contract

1. Resolve selected workspace folder and explicit server identity. For a multi-root project, offer folder selection, not silently the first root.
2. Find/create a Herdr workspace keyed by canonical project root plus server/machine identity. Workspace labels alone are not identity. Decide how symlinks map. Keep separate git worktree directories separate by default.
3. First creation uses the returned root pane. Later New creates one new Herdr tab and attaches its root pane.
4. Store attachment identifiers, not ownership of a process. Treat the server as authoritative after reconnect. Persisted handles require revalidation after server restart.
5. Close view, close IDE, extension crash and transport loss release attachment only. No pane/tab/workspace close command in disposal handlers.
6. Reopen from the sidebar reattaches the same terminal. An already-open row reveals its view rather than duplicating it.
7. A distinct Terminate Session action shows the target and requires confirmation. Shell `exit` still ends the shell. Ctrl-C still interrupts the foreground command. Persistence must not disable intentional process control.
8. Reconnect restores views by identity. It must never run the New path. New-session creation needs idempotency or recoverable creation tokens to handle double clicks, cancelled providers and dropped responses.
9. If another controller owns the terminal, show In use and offer Observe or explicit Take over. Do not automatically pass --takeover.
10. Background creation uses no-focus where supported to avoid stealing focus from a separate Herdr TUI client. IDE focus does not automatically imply global Herdr focus.

The initial one-terminal-per-tab mapping is deliberately simple. If a tab later contains split panes, the sidebar should expose those panes as separate attachable leaves, not imply one terminal view can show the whole tab layout.

## Sidebar and deeper integration

Use a native TreeDataProvider, not a webview, for workspace → tab → pane/session rows. Keep the current project prominent and other workspaces opt-in.

Initial display: label/title, working directory, agent kind, idle/working/blocked/done/unknown state, and whether this extension has an open view. Global controller ownership is not exposed in the inspected PaneInfo fields, so a definitive global Attached/In use badge needs capability/API confirmation rather than inference from local views.

Later display: available metadata tokens such as model/context/cost, agent session identity, activity including outstanding agents/overnight state, worktree information and attention indicators. These fields may be absent, stale, provider-specific or version-dependent. Missing values stay unknown. Tokens are not proof that every agent supplies every metric.

Useful actions: New, Attach, Detach view, Rename, Copy attach command, Observe, confirmed Take over, confirmed Terminate, and Reveal project. Completion/blocked notifications should be opt-in and deduplicated. Avoid copying provider hooks, reading transcripts, exposing prompts in tooltips, or creating a second state classifier unless separately approved.

## Adversarial findings and decisions

- **Native UX is the main technical gate.** Rendered ANSI frames do not guarantee native shell scrollback/search, OSC 633 command boundaries, cwd links, shell completion, screen-reader behavior or Cursor output collection. Test before claiming normal-terminal parity. Pseudoterminal alone does not solve this.
- **Nested input handling.** Direct attach reserves Ctrl+B sequences. Validate keyboard chords, bracketed paste, mouse selection/wheel, alt-screen TUIs, Ctrl-C and resize across both editors.
- **Close semantics.** IDE trash commonly means kill. For managed terminals it must kill only the attachment process. Label explicit Herdr actions clearly even if the host toolbar cannot be renamed.
- **Restoration ownership.** Prefer explicit extension-owned reattachment with transient IDE views if reliable native restoration cannot be reconciled. Do not respawn create-on-start wrappers during revive.
- **Two windows.** Both can race to create a project workspace. Use a server-atomic ensure or appropriate cross-process locking and reconciliation. A process-local mutex is insufficient.
- **Remote context.** Run the workspace extension on the SSH/WSL host alongside Herdr. A local filesystem path is not a remote identity. Herdr saved-machine API forwarding does not automatically provide interactive terminal streaming. Dev-container recreation is outside ordinary IDE-close persistence.
- **Native Windows.** Direct attach is explicitly unsupported in inspected code. JSON control is a candidate, not a verified Windows solution. Start with Linux and test macOS/Remote SSH separately.
- **Server startup.** Need a noninteractive, detached, capability-checked startup policy. Do not let extension disposal own server shutdown or silently restart/upgrade an existing server. OS logout/service policies can still kill an otherwise independent daemon.
- **Security.** Trusted workspaces only for execution/creation. Use explicit executable paths and argument arrays, never interpolate project labels into shell commands. Keep local socket permissions and remote authority boundaries intact. Do not forward the entire IDE environment or unrelated inherited HERDR_* routing variables. Pass only necessary context to new panes.
- **Environment drift.** Reattaching a pre-existing shell does not refresh its environment, credentials or tools. Expose that distinction rather than silently restarting work.
- **Cursor automation.** Manual native terminals and Cursor's agent terminal/sandbox are not the same surface. Do not route agent tasks through Herdr or assume sandbox equivalence. Keep automated task/debug terminals ordinary in v1.
- **Performance.** Subscribe to metadata events and reconcile after reconnect. Bound queues and visible terminals. Do not continuously fetch all terminal contents for sidebar badges. Test one and at least fifteen sessions if render paths change.
- **Distribution.** One stable-API VS Code extension can target both editors, with a VSIX and chosen registries later. Verify the actual Cursor API baseline rather than assuming latest VS Code APIs exist there. No publication or registry account setup is authorized by this exploration.

## External evidence

Research used authenticated installed Firecrawl CLI 1.23.3: six searches, five initial scrapes and four follow-up scrapes. Search results are discovery, not acceptance evidence. Public URL HTTPS/userinfo/DNS checks were applied before scrapes, and CLI was launched through Python subprocess argument arrays. Hosted redirect handling follows the policy's public-research trust boundary, not independently inspected hops. At most five requests ran concurrently, with bounded result counts and command timeouts. All nine scrape commands returned exit 0. Full API scrape was about 1.15 MB. Raw scraped pages are transient and replaced by these source-linked findings. Search feedback was submitted after use without asserting refund success.

1. VS Code Terminal Advanced: https://code.visualstudio.com/docs/terminal/advanced
   Distinguishes window-reload process reconnection from full-restart process revive, which relaunches using the original environment. Built-in persistence is therefore not the required independent process keeper.
2. VS Code API: https://code.visualstudio.com/api/references/vscode-api
   TerminalProfileProvider, TerminalProfile, TerminalOptions, ExtensionTerminalOptions, Pseudoterminal, shell integration contracts.
3. Terminal Sessions by ViSuL: https://marketplace.visualstudio.com/items?itemName=visul.terminal-sessions
   Closest product precedent. Describes native tmux attach terminals, project-hashed/per-tab IDs, default profile, sidebar, reattachment and agent metadata. Supports VS Code/Cursor and remote-host operation by its own documentation. This is a small project, not an independently tested endorsement. Do not adopt its hooks/transcript collection or platform claims without review.
4. Code Mux: https://marketplace.visualstudio.com/items?itemName=jellydn.vscode-mux
   Contributes a profile using tmux/zellij and workspace grouping. Subsequent terminals attach the same multiplexer session, so its mapping differs from the requested distinct Herdr tab per New. Documents multi-root and layout-management limits.
5. George Honeywood, Persistent terminals in VS Code with tmux: https://george.honeywood.org.uk/blog/vs-code-and-tmux/
   Concrete settings/wrapper precedent showing why a multiplexer daemon, not IDE revive, preserves commands.
6. Zed tmux configuration: https://zed.tips/tips/tmux-terminal-sessions
   Community configuration for project-specific attach/create. Evidence of the wrapper approach, not a built-in universal guarantee.
7. Zed persistent terminal threads discussion: https://github.com/zed-industries/zed/discussions/57428
   Community discussion of persistence/resurrection and tmux-backed bridges. Not evidence that the proposed behavior has shipped generally.
8. JetBrains Terminal documentation: https://www.jetbrains.com/help/idea/terminal-emulator.html
   Saves tab names, cwd and shell history across project/IDE closure. Explicitly documents tab-close termination. Does not establish that arbitrary processes survive a full IDE exit.
9. Cursor Terminal: https://cursor.com/docs/agent/tools/terminal
   Documents agent terminal run-mode/sandbox boundaries and shell-theme compatibility. Does not establish that a custom multiplexed terminal is equivalent to its agent execution surface.

## Verification before implementation commitment

Run a bounded spike in disposable Herdr sessions, not the user's active workspace:

1. Start a shell and a child that records PID, start time and monotonic progress. Close IDE terminal, reopen attachment and verify the same process continued, not merely a new PID with restored text.
2. Repeat for IDE full quit, reload, extension-host failure, attachment process death and Remote SSH disconnect. Only the attachment may die.
3. New three times produces one project workspace and three tabs, including correct first-workspace root reuse. Simultaneous windows must not duplicate the workspace.
4. Reopen/restore/cancel/retry must not allocate new shells. Explicit terminate affects only the selected pane/session.
5. Test scrollback/search, links, long output, paste, Unicode, alt-screen agents, mouse, resize, clipboard, Ctrl-C, Ctrl+B, shell command detection and accessibility in actual VS Code and Cursor.
6. Test second-controller refusal, observer coexistence, explicit takeover, stale server IDs, server upgrade/reconnect and inaccessible remote.
7. Confirm metadata is sourced from Herdr, unknown fields stay unknown, TTL/revision changes refresh correctly and no shell-only session disappears from the tree.
8. Verify trust-mode refusal and hostile filenames. Confirm no external writes, install prompts or hook edits occur silently.

## User decisions and next route

No blocking credential or provisioning step is needed for this recommendation. Before a feature contract, confirm initial platform scope (recommend local Linux first, then macOS and Remote SSH), whether restoration should offer a picker or auto-open previously visible sessions, and whether the Herdr profile should become the default after an explicit user choice. Prototype terminal fidelity first. If it passes, route to feature authoring for the thin extension. If it fails, scope a neutral Herdr terminal protocol improvement before promising full native-terminal behavior. No implementation, installation, configuration migration or publication is performed here.
