# Herdr IDE plugin ecosystem exploration

Date: 2026-09-23. Exploration only. No repository scaffold, formal change, plugin installation, publication or configuration mutation performed. The user's request to create a personal repository was followed by a request to first check alternatives and the active explore instruction prohibits implementation/formal change during exploration.

## Intent and approved constraints

Evaluate whether existing Herdr-store projects replace or complement the proposed native VS Code/Cursor persistent-terminal extension. Choose a repository name suitable for an eventual genuine Herdr companion plugin. Proposed local root is `~/dev/personal/herdr-ide` (currently absent).

Previously approved behavior: project/server-qualified workspace identity; one new Herdr tab per New; IDE-created sessions always take over; externally created sessions observe-only; IDE close releases attachments, never processes; separate task/debug/Cursor agent execution; remote extension host alongside Herdr; no uninterrupted-process promise across reboot. Missing creation provenance should observe. Prevent displaced IDE views from entering competing automatic takeover loops.

## Internal prior art and current evidence

- `/home/nyaptor/.jcode/research/herdr-ide-integration-2026-09-23.md`: initial settings/extension research.
- `/home/nyaptor/.jcode/research/herdr-cursor-spike-20260923/report.md`, events.jsonl and tests.js: actual Mac Cursor 3.21.18 / Herdr 0.9.1 lifecycle validation. Verified same shell/child surviving native terminal disposal, isolated full editor exit and attachment SIGKILL. Automatic takeover, observer transport and native observer Pseudoterminal tested. No durable provenance/profile provider/sidebar implemented. Visual inspection blocked by unavailable display capture; shellIntegration false.
- Local Herdr Rust source: public JSON metadata/events and existing CLI terminal attach/control/observe.
- Local `/home/nyaptor/dev/herdr/src/app/api/plugins/manifest.rs`: Herdr plugin manifest has id/name/version, minimum version, platform constraints, builds, startup hooks, actions, panes, events and link handlers. It is not a VS Code extension manifest.
- Local `/home/nyaptor/dev/herdr/website/src/pages/plugins.astro`: public full catalog source is `https://assets.herdr.dev/plugins/index.json`; initial HTML only shows a slice.
- Local marketplace worker discovery query: `topic:herdr-plugin is:public`.
- No relevant memory match and no local checkouts of the three named alternatives found. Prior sessions are context, while current source and receipts provide evidence.
- Personal organization AGENTS routing was read earlier in this session. Architecture advice follows narrow adapters, no new runtime ownership or stack migration.

## Context7

Consulted `/websites/code_visualstudio_api` for extension manifest, publishing and remote extension placement. Editor package uses package.json, a publisher-scoped extension identity, engines.vscode, and VSIX packaging. Workspace host placement is a separate concern from a Herdr plugin manifest. Context7 snippets include legacy examples, so choose an API floor from actual feature usage and test supported editor versions rather than copying example version numbers.

## Research method and coverage

Used installed authenticated Firecrawl CLI 1.23.3 through no-shell Python argv calls. Submitted only public HTTPS URLs after DNS public-address checks. No code, credentials or personal files uploaded. Maximum five concurrent scrapes. No competitor code was installed or executed.

Initial catalog response was cached at 2026-09-22T21:00:30.642Z. Re-fetched with --max-age 0: snapshot generated 2026-09-23T14:30:26.246Z, 1,319 plugins across 1,286 repositories. Searched repo names, descriptions and manifest metadata for Cursor, VS Code, IDE/editor and persistent terminals. This is broad metadata coverage, not an audit of all repository source. No claim that a direct competitor cannot exist elsewhere.

Sources inspected:
- https://herdr.dev/plugins/
- https://assets.herdr.dev/plugins/index.json
- https://herdr.dev/docs/marketplace/
- https://github.com/alex-devdone/herdr-cursor-open
- https://github.com/alex-devdone/herdr-cursor-open/blob/main/herdr-plugin.toml
- https://github.com/timofey-TK/herdr-open-in-editor
- https://github.com/timofey-TK/herdr-open-in-editor/blob/main/open_in_editor.py
- https://github.com/vonzelle-vzt/herdr-extensions
- https://github.com/vonzelle-vzt/herdr-extensions/blob/main/install.sh
- https://github.com/scott-the-programmer/vscode-devcontainers-herdr
- https://github.com/gabriel-laet/herdr-cursor
- https://github.com/pinkpixel-dev/quota
- https://github.com/endoumame/herdr-vscode

Two bounded Firecrawl searches also surfaced names outside the catalog. Search feedback submitted without claiming refund success. GitHub file rendering elided the middle of open_in_editor.py, so inspected code substantiates path resolution and editor-launch contracts, not a full relay/security audit. Competitor README performance, reversibility and safety claims remain vendor claims unless directly examined.

## Named alternatives

| Project | Actual direction | Fit against this request |
| --- | --- | --- |
| alex-devdone/herdr-cursor-open | Herdr focused pane directory → local Cursor/VS Code/derivative or remote-aware editor | Complements the feature, does not provide persistent native IDE terminal tabs or session tree |
| timofey-TK/herdr-open-in-editor | Herdr worktree/workspace/pane path → VS Code or Zed; reverse SSH relay for remote open | Complements the feature, does not supply the persistent terminal client |
| vonzelle-vzt/herdr-extensions | Editor, panels and IDE-like workflow inside Herdr terminal UI | Alternative working environment, opposite direction from keeping Cursor as the IDE |

### alex-devdone/herdr-cursor-open

README describes a local macOS editor picker, selected Herdr-machine handling, remote cwd lookup and a watcher pulling remote request files over SSH. Manifest inspected: id `shestmintsev.cursor-open`, version 0.1.0, macos-only, open action and startup watch.sh. README includes JetBrains support beyond the name. Valuable lessons: editor launch must happen on the display host; pane IDs are server-scoped; remote requests must not replay old actions on reconnect. Not a base to fork for TypeScript terminal/profile/sidebar work. Avoid copying watcher semantics before a transport/security review.

### timofey-TK/herdr-open-in-editor

README and visible Python source select worktree checkout_path, then workspace cwd, then focused pane cwd. Launches code/zed locally, or sends an open request through a loopback relay and SSH reverse tunnel (documented remote port 47831). Python source uses argument arrays and rejects nonabsolute selected paths. Tests are provided but not run here. The path-selection contract is useful prior art; its reverse relay is unnecessary for v1 when the extension itself runs on the remote host. Do not assume localhost or an SSH tunnel alone resolves all confused-deputy/authentication concerns.

### vonzelle-vzt/herdr-extensions

An installer and terminal-IDE arrangement: installs herdr-edit or supported editor, tooling and fonts, registers panels and writes keybindings/skin. README describes 15 panels while catalog description says 12, illustrating why catalog text is discovery rather than current behavioral proof. Bootstrap install.sh clones/symlinks the CLI and explicitly does not invoke the config-mutating install command automatically. This is not a VSIX despite the name. Wrong base for our client and would broaden dependencies/configuration substantially.

## Additional alternatives and name conflicts

- `endoumame/herdr-vscode`: real VS Code extension for inline code review and batching comments to an existing Herdr agent. Uses workspace extension host and refuses untrusted workspaces per README. Closest platform-level prior art, but not a persistent-terminal client. It complements our extension. Name already used, avoid `herdr-vscode`. Not present in the fetched Herdr catalog, showing why catalog-only name checks are insufficient.
- `gabriel-laet/herdr-cursor`: Cursor cloud-agent roster/streaming inside Herdr, not desktop IDE terminals. Avoid `herdr-cursor`.
- `scott-the-programmer/vscode-devcontainers-herdr`: host/container state relay and agent identity tracking. It does not attach Herdr shells as native IDE terminal tabs. Potential later integration, not v1 dependency. README explicitly identifies widened socket exposure for nondefault bridge binding.
- `pinkpixel-dev/quota`: one repo with desktop app, VS Code package and separate `herdr-plugin/` package. Concrete distribution precedent for one repository with multiple install surfaces, not terminal competition.
- `azizuysal/herdr-workbench`: already in fresh catalog, project sidebar inside Herdr. Avoid that name.
- `alexkarpandrus/herdr-dock`: surfaced by awesome-herdr search. Name collision is discovery-level, functionality not separately audited. Avoid `herdr-dock` rather than spending more research to reuse an already occupied name.
- Earlier research `visul/terminal-sessions` remains the closest product behavior precedent, with tmux rather than Herdr as process owner.

No direct replacement for the full approved native-terminal workflow was identified in the inspected projects and catalog metadata. This is a bounded finding, not an ecosystem uniqueness claim.

## Store contract and packaging

Official marketplace docs say: public GitHub repo, `herdr-plugin` topic, and a parseable `herdr-plugin.toml` on the default branch. Manifest can be root or nested. One repo card may contain multiple installable plugins. Automatic refresh is documented as 30 minutes. Forks, archived repos and invalid/missing manifests are excluded. Listing is automatic and unreviewed, not certification.

VSIX and Herdr plugin are separate artifacts. `herdr plugin install owner/repo[/subdir]` does not inherently install an IDE extension. Do not add a dummy manifest just for discoverability or silently install editor extensions from Herdr startup.

Recommended eventual repo surfaces:
- `extension/`: native VS Code/Cursor profile, attachments, session tree, reconnect and lifecycle management.
- `herdr-plugin/`: optional genuine companion with an explicitly invoked health/setup diagnostic and, later if approved, a safe contextual open action. Keep it small and reuse the existing ecosystem for generic open-in-editor instead of duplicating remote relays.
- `docs/`: approved design, competitor comparison and sanitized runtime verification.

A diagnostic companion should report supported Herdr capabilities, connection/version mismatch and clear extension installation instructions. It should not mutate personal config or publish/install anything on startup. This companion is a proposal, not approved extra implementation. It need not ship with the first extension release. If it does not have a real useful function yet, defer the Herdr-store listing rather than manufacturing one.

## Naming recommendation

Recommend local repository `~/dev/personal/herdr-ide` and display name **Herdr IDE**, with subtitle **Persistent Herdr terminals for VS Code and Cursor**.

Reasons: editor-neutral, describes the bridge rather than a launcher, leaves room for session metadata, and avoids observed names herdr-vscode/herdr-cursor/herdr-workbench/herdr-dock. No exact repo-name collision was found for herdr-ide in the fetched catalog or bounded web search. This is not trademark clearance, namespace reservation, or proof of registry availability. Do not invent the publisher ID, GitHub owner or license. Those can be decided before public publishing, not needed for local exploration.

Alternative: a more narrowly descriptive `herdr-ide-sessions`, at the cost of a longer name. Avoid broad `herdr-extensions`, already occupied and ambiguous.

## Adversarial concerns

- A remote plugin executes on the server, not necessarily on the desktop holding Cursor. Do not launch remote GUI commands from generic plugin actions and call it local integration.
- Store install and IDE extension install have different trust boundaries and update cadence. Document both versions and compatibility rather than implying one install enables everything.
- Origin (IDE vs external) is durable identity metadata, not the current owner. Unknown origin observes. Reconnect cannot become an unconditional takeover race across editors.
- Open-editor plugins do not convey a safe terminal identity by opening a directory. Contextual deep links require server-qualified IDs, validation and safe replay behavior.
- Local example source is Rust; proposed extension is ordinary TypeScript targeting the editor's Node extension host, not T3 or a Bun/Effect service. No new framework or background runtime required.
- Preserve scope: no editor, LSP, debugger, reverse relay, provider SDK or transcript collection in v1.
- Documentation should clearly mark independent community integration, not an official Herdr extension.
- Source reuse requires a file-level license review and attribution. No competitor code copied.
- Do not carry machine-specific test paths, PIDs or private logs into a public repo without sanitizing evidence.

## Actions, decisions and routing

Research did not require credentials, external provisioning, paid account changes or any user-only action. A local repository can be created after this exploration closes and naming/scope are accepted. Public GitHub creation, registry publication, publisher identity and license remain separate decisions/authorization. No need to block local work on those external steps.

Recommended next route: accept `herdr-ide`, create a local-only documentation baseline carrying the exploration and sanitized spike evidence, then use feature authoring for the persistent-terminal extension contract. Treat the optional Herdr companion/listing as a separate deliverable rather than letting store requirements drive a premature runtime component.

Verification gates remain: native terminal visual/scrollback fidelity, command-shell integration limits, origin/provenance routing, takeover/reconnect concurrency, workspace creation idempotency, first-tab reuse, multi-root/remote identity, trust checks and packaging tests in actual VS Code and Cursor. Preserve the Mac lifecycle evidence without presenting untested surfaces as complete.
