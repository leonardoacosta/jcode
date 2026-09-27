# Local desktop MCP integration audit

Scope: read-only scout on 2026-09-27. No live app input, MCP config contents, or credentials were read or changed. Run manifest was `running` with no cancellation state found at audit start. Existing worktree is dirty; no product files were edited.

## Findings

### Minimal integration path

Jcode already launches MCP servers over stdio and dynamically registers tools. The narrowest path is adding one named stdio server to a *disposable Jcode home* `mcp.json`, then use the existing MCP management tool's `reload` action in that isolated instance. No source change, `config.toml` edit, new transport, or global MCP config edit is needed for a first validation.

Example shape only; do not place secrets in this file:

```json
{
  "mcpServers": {
    "desktop-test": {
      "command": "/absolute/path/to/server",
      "args": [],
      "env": {},
      "shared": false,
      "enabled": true
    }
  }
}
```

Use an absolute executable path, minimal explicit environment, and a unique server name. Jcode accepts both `mcpServers` and legacy `servers`; server fields include `command`, `args`, `env`, `shared` (default true), transport/url fields for compatibility, and enabled/disabled switches. Local/global Jcode MCP config is separate from `config.toml`. Sources include `~/.jcode/mcp.json`, project `.jcode/mcp.json`, Claude Code global/project files, and legacy `.claude/mcp.json`; project sources merge later. `JCODE_DISABLE_CLAUDE_MCP` disables Claude config import. Avoid reading or altering existing user/Claude config during the trial.

Use `shared: false` for a stateful desktop service so the process is session-owned rather than pooled across sessions. Jcode's stdio client starts the configured process with piped stdin/stdout/stderr, logs stderr, and does not blindly pass provider credential environment variables to MCP children. Only explicitly configured env is appropriate; omit secrets.

### Reload / approval / exposure

MCP configuration is loaded on manager startup and on explicit `mcp` action `reload`; no file watcher was found. Reload disconnects clients, reloads merged sources, reconnects enabled servers, unregisters old `mcp__*` tools and registers tools discovered from fresh connections. The empty-config branch unregisters existing MCP tools but returns before manager reload. Config entries can also be connected ad hoc through `mcp connect`, so for the experiment avoid asking the agent to connect arbitrary commands; preconfigure a single fixed executable in isolated `JCODE_HOME` instead.

Approval nuance: MCP tool calls do **not** have a dedicated per-call confirmation prompt. They execute through the ordinary tool registry. `[tools] enabled`/`disabled` policy and session allow/disable policy can restrict whether MCP tools are exposed/accepted. `mcp` in an allow/deny set covers the `mcp__*` prefix. This is a tool availability gate, not human approval for each click/type. Default full profile can expose connected MCP tools. The server's own safety model and least privilege therefore matter. Do not connect it to a personal/live app.

### Screenshot/image path

MCP protocol parsing recognizes image blocks (`data`, `mimeType`). The adapter initially reduced those blocks to text, but the root-cause fix in `crates/jcode-base/src/mcp/tool.rs` now preserves valid image payloads as `ToolOutput.images`. It accepts PNG/JPEG/GIF/WebP, requires non-empty valid standard base64, and caps decoded bytes at 20 MiB (with a pre-decode encoded-size guard). Invalid or unsupported image content is omitted with a text marker. Validation checks MIME, base64 syntax, non-emptiness and size only. It does not decode/parse image formats, verify bytes match the declared MIME, or check dimensions.

Regression tests cover MCP image block → `ToolOutput.images` plus malformed, empty, unsupported-MIME and oversized rejection. The stdio integration launches the current Rust test executable as a local MCP fixture on Unix, connects through `McpManager`, calls `McpTool::execute`, and asserts the PNG payload appears in `ToolOutput.images`. It no longer depends on Python; the process test is `#[cfg(unix)]` because its launcher is a shell script. `cargo test -p jcode-base mcp::tool::tests --lib`: **5 passed, 0 failed** on Linux. Durable log: `overnight/runs/overnight_1790544423148_3438749930501896105/validation/mcp-tool-tests-portability.log`. Commits: `60db09a8d` (image fix), `50035dbee` (stdio execution + empty-payload rejection), `bf9d2db37` (Python-free fixture), `c080c32d1` (durable test log).

Validation checks only PNG/JPEG/GIF/WebP MIME allowlist, non-empty syntactically valid standard base64, and <=20 MiB decoded size. It does not parse/decode the image, prove bytes match declared MIME, or validate dimensions.

This proves stdio MCP → manager → `McpTool.execute` → `ToolOutput.images`, not a live model request. Static path evidence shows `ToolOutput.images` becomes `ContentBlock::Image` and Anthropic formatting appends it to the preceding tool result; existing `tool_output_to_content_blocks_preserves_labeled_images` covers the middle conversion. Two isolated `jcode run` attempts did not expose the synthetic MCP tool, so no model call included the image. The coordinator's bounded Jev `evaluate` reached omni but returned HTTP 404 `unknown_route` for `/v1/systemone`; Jev remains blocked. No shared daemon reload or config edit occurred. Debug tool output omits images and cannot prove image forwarding.

Unexposed-tool runtime logs: `/tmp/jcode-bg-tasks/087021ug32.output`, `/tmp/jcode-bg-tasks/143293lwby.output`. Earlier pre-portability focused test log: `/tmp/jcode-bg-tasks/7465761dif.output`.



### Runtime propagation follow-up (2026-09-27)

A synthetic one-pixel PNG stdio MCP server was prepared under a unique test directory and invoked via `target/selfdev/jcode run --no-update --socket ...` on a dedicated socket. The server returned a valid `image/png` MCP image block. The Jcode model reported `image-probe/capture_screenshot` was not exposed on both attempts, so no MCP tool call ran and end-to-end payload propagation is **not validated**. The first config was placed under a temp `JCODE_HOME`; the second was also copied to a disposable project `.jcode/mcp.json` and used with `--cwd`, but `jcode run` is a client and does not provide evidence that its isolated server registered this client-side project config. The shared daemon and real MCP config were not touched.

Static path evidence: `tool_output_from_result` builds `ToolOutput.images`; `crates/jcode-app-core/src/agent/tools.rs::tool_output_to_content_blocks` converts each to `ContentBlock::Image`; `crates/jcode-provider-anthropic/src/lib.rs::format_content_blocks` attaches image blocks to the immediately preceding tool result. Existing regression `tool_output_to_content_blocks_preserves_labeled_images` covers the middle conversion. Provider-specific live model receipt still needs a correctly configured isolated Jcode server/session with the MCP tool actually exposed. Jcode run on the built binary succeeded against the configured OpenRouter model; this bounded check did not invoke Jev `evaluate`, so it does not directly prove the earlier System One route diagnostic.

Synthetic run output logs: `/tmp/jcode-bg-tasks/087021ug32.output` and `/tmp/jcode-bg-tasks/143293lwby.output`. Original focused test log: `/tmp/jcode-bg-tasks/101502nolz.output`.


- Jcode's MCP stdio client, manager, reload management tool, and MCP tests already exist (`crates/jcode-base/src/mcp/{client,manager,protocol,tool}.rs`, `crates/jcode-app-core/src/tool/mcp.rs`).
- `jcode` and `npx` are on PATH; no global MCP inspector/desktop package was detected. Python MCP/Playwright modules were absent. Node/package-manager availability does not prove an npm package is already cached.
- Linux Wayland/X11 screenshot/input utilities exist (`grim`, `xdotool`), but no existing installed desktop MCP was identified. These host tools do not establish safe app targeting by themselves.
- Jcode README documents an existing remote macOS desktop tool, but it is deliberately constrained: status, app/window list, accessibility snapshot/find/property reads, ref-based click/type; no screenshots, shell, raw coordinates, clipboard, keyboard shortcuts, scripting, permission prompts or termination. It follows `[tools]` and session allow/disable policy. Not equivalent to a local screenshot-capable MCP.

## Recommended disposable-app validation loop

1. **Isolate Jcode first.** Use a temporary `JCODE_HOME` directory and a dedicated socket/daemon or one-shot run, with a throwaway provider/model route if needed. Do not use the active `~/.jcode/mcp.json`, project MCP files, or live daemon. Keep logs and session history under the temp home; remove the temp home only after collecting results.
2. **Constrain the desktop server.** Choose an MCP implementation that can be launched locally over stdio and can target exactly one disposable app/window by explicit app/window ID or a dedicated virtual display. Use a disposable app with synthetic data, no logged-in accounts, and no access to user documents. If the server cannot strictly target one window, run it inside a fresh nested compositor/VM/container with a dedicated display and no host input bridge. Do not rely on title matching alone when other windows may appear.
3. **No live input initially.** Start with `mcp list` and read-only tool calls: server status, app/window inventory, accessibility tree, properties, and screenshot only if the tool result implementation can be inspected as image data. Confirm the model receives expected *text*. Current adapter is known to discard MCP image payloads, so image confirmation requires a direct MCP client/inspector or a purpose-built image-capable Jcode bridge; do not infer model vision from a text `[Image: ...]` marker.
4. **Gate mutations explicitly.** Configure only the selected server as enabled and restrict Jcode tools to required names (or disable `mcp` entirely between probes). For any later click/type test, use a clearly labeled dummy app with one reversible field and a synthetic value. Verify action result in the app and verify no unrelated window got input. Stop server and isolated Jcode process after each run.
5. **Test cancellation and failure behavior.** Check MCP `list` reports the intended server/tools; invoke harmless read-only call; verify explicit `reload` removes/re-registers tools and disabled state prevents startup; test unavailable command or timeout in a disposable configuration, then restore only the temporary config. Capture process exit and sanitized logs. Never put credentials in MCP JSON or paste config contents into logs/report.

### Suggested validation layers

- **Protocol/unit:** run targeted Rust tests: `cargo test -p jcode-base mcp::protocol_tests`, `cargo test -p jcode-base mcp::manager`, and `cargo test -p jcode-app-core --lib tool::mcp` (confirm exact test filters/package targets with `cargo test -- --list` if workspace naming differs). Focus on stdio launch, config parsing/merge, reload registration, and image-block conversion expectation.
- **Disposable stdio smoke:** use a tiny local echo/read-only MCP fixture or selected desktop MCP in temp `JCODE_HOME`; initialize/list tools/call a harmless inventory tool; verify no provider credentials are inherited and server is terminated on exit.
- **End-to-end:** isolated Jcode process with only the fixed desktop MCP enabled; confirm tool appears, read-only accessibility/text reaches the model, then stop. Image result is a known blocker in current adapter and should be separately tested at raw MCP boundary until implementation changes.
- **Mutation phase, only after read-only passes:** disposable app on isolated display, single explicit target, one reversible interaction, verify exact target and postcondition. No current user input was performed.

## Evidence / caveats

Key source: `crates/jcode-base/src/mcp/protocol.rs` (schema and source merge), `crates/jcode-base/src/mcp/client.rs` (stdio process and environment), `crates/jcode-base/src/mcp/manager.rs` (reload), `crates/jcode-app-core/src/tool/mcp.rs` (management and registry refresh), `crates/jcode-base/src/mcp/tool.rs` (image textification), `crates/jcode-app-core/src/tool/mod.rs` and `agent/turn_execution.rs` (allow/disable policy), README MCP and remote-desktop sections.

The audit only checked whether selected MCP config paths exist, not their contents. Existing `config.toml` has unrelated user changes and remains untouched. The recommended exact desktop MCP package was not selected because no package identity or capabilities were specified and no installed inspector/server package was found. Select one only after confirming screenshot support, app/window targeting, read-only modes, stdio launch, platform support and its permissions model.

🌱 graft saved ~7,368,148 tokens this turn (5 calls).