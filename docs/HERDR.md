# Herdr integration contract

Jcode has built-in terminal routing for Herdr. When a headed session launch is requested from a client with `HERDR_ENV=1` and `HERDR_PANE_ID`, Jcode splits the calling pane to the right, focuses the new pane, and starts the resumed Jcode session there. `HERDR_BIN_PATH` is honored when present.

This covers visible swarm spawns, resume-in-new-terminal, self-development launches, and restart restores because they all use the shared terminal launcher. A configured `[terminal].spawn_hook` still takes precedence.

## Current compatibility

Jcode already:

- forwards `HERDR_ENV`, `HERDR_SOCKET_PATH`, `HERDR_PANE_ID`, `HERDR_TAB_ID`, `HERDR_WORKSPACE_ID`, `HERDR_BIN_PATH`, `HERDR_SESSION`, and `HERDR_AGENT` from the requesting client to server-side spawn and focus paths;
- recognizes Herdr as a masking terminal multiplexer for Mermaid graphics capability detection;
- exports stable lifecycle observer hooks for `session_start`, `session_end`, `turn_start`, and `turn_end`;
- exports `JCODE_HOOK_SESSION_ID`, `JCODE_HOOK_CWD`, event fields, and a JSON `JCODE_HOOK_PAYLOAD`;
- resumes a native session with `jcode --resume <session-id>`.

## Recommended first Herdr integration

The initial Jcode integration uses Herdr's **custom harness** path for live visibility: Jcode reports semantic state through `pane.report_agent` with `source = "custom:jcode"` and `agent = "jcode"`. This makes Jcode appear in Herdr's agent panel without requiring a Herdr binary update. Official native session restore still requires Herdr-side `jcode` support, because `pane.report_agent_session` alone stores a session reference but does not create a visible custom agent row.

Jcode ships a hook adapter at `scripts/jcode-herdr-agent-state.sh` for this safe first integration. Add it as an additional hook command rather than replacing existing user hooks:

```toml
[hooks]
session_start = ["/path/to/jcode/scripts/jcode-herdr-agent-state.sh session"]
turn_start = ["/path/to/jcode/scripts/jcode-herdr-agent-state.sh session"]
turn_end = ["/path/to/jcode/scripts/jcode-herdr-agent-state.sh session"]
session_end = ["/path/to/jcode/scripts/jcode-herdr-agent-state.sh session"]
```

When composing with existing hooks, keep every command in the array:

```toml
[hooks]
session_start = [
  "~/bin/my-existing-session-hook",
  "/path/to/jcode/scripts/jcode-herdr-agent-state.sh session",
]
turn_start = [
  "~/bin/my-existing-turn-start-hook",
  "/path/to/jcode/scripts/jcode-herdr-agent-state.sh session",
]
turn_end = [
  "~/bin/my-existing-turn-end-hook",
  "/path/to/jcode/scripts/jcode-herdr-agent-state.sh session",
]
session_end = [
  "~/bin/my-existing-session-end-hook",
  "/path/to/jcode/scripts/jcode-herdr-agent-state.sh session",
]
```

The adapter exits successfully without side effects unless it is running inside Herdr (`HERDR_ENV=1`) with `HERDR_SOCKET_PATH`, `HERDR_PANE_ID`, and `python3` available.

If the inherited `HERDR_PANE_ID` is stale and Herdr replies `pane_not_found`, the adapter queries `session.snapshot` and retries only when exactly one live pane has `cwd` or `foreground_cwd` equal to `JCODE_HOOK_CWD`. This keeps moved/restored panes working without guessing when multiple panes share a directory.

On Jcode `session_start`, the Herdr hook sends one newline-delimited JSON request to `HERDR_SOCKET_PATH`:

```json
{
  "id": "herdr:jcode:<unique-request-id>",
  "method": "pane.report_agent",
  "params": {
    "pane_id": "<HERDR_PANE_ID>",
    "source": "custom:jcode",
    "agent": "jcode",
    "seq": 1,
    "state": "unknown",
    "agent_session_id": "<JCODE_HOOK_SESSION_ID>",
    "message": "jcode session active"
  }
}
```

The sequence must be monotonically increasing for the source.

Herdr should restore the session with:

```text
jcode --resume <agent_session_id>
```

Jcode session IDs are opaque strings and fit Herdr's ID-based session reference model. No transcript path is needed.

On Jcode `session_end`, the adapter sends `pane.release_agent` for the same `("custom:jcode", "jcode")` source/agent pair. This clears normal closed sessions.

## Hook mapping

| Jcode hook | Herdr action now | Rationale |
| --- | --- | --- |
| `session_start` | Active: `pane.report_agent` with state `unknown` and `agent_session_id = JCODE_HOOK_SESSION_ID`. | Creates a visible custom Jcode harness row while carrying the native session id. The message uses the two-line agent layout. |
| `session_end` | Active: `pane.release_agent`. | Clears custom Jcode lifecycle authority on normal close. |
| `turn_start` | Active: `pane.report_agent` with state `working`. | Shows active Jcode work in Herdr's agent panel and rollups. The message uses `[status] [project]` on row 1 and the custom session name on row 2. |
| `turn_end` | Active: `pane.report_agent` with state `idle`. | Marks Jcode ready after a completed turn. |
| `pre_tool` | No-op for Herdr authority. | It is a gate for tool policy, not a complete agent-state transition. |
| `post_tool` | No-op for Herdr authority. | It observes tool completion only; text streaming, approvals, and interrupts happen outside this boundary. |

## Required Herdr-side work

A first-class integration cannot be shipped only as a remote detection manifest. Herdr currently hard-codes known agent kinds, official session sources, restore commands, and install targets. The upstream implementation needs:

1. Add `jcode` to `IntegrationTarget`, CLI parsing, labels, command discovery, recommendations, status, install, and uninstall handling.
2. Install a config-safe Jcode session hook adapter without overwriting an existing user hook. If Herdr cannot safely compose the single Jcode hook command, coordinate a small multi-hook or native-emitter addition in Jcode first.
3. Accept `("herdr:jcode", "jcode")` as an official session source.
4. Persist its ID session reference and map it to `jcode --resume <id>` during restore.
5. Add Jcode process detection and a bundled screen manifest for idle, working, and blocked UI states.
6. Keep screen-manifest detection authoritative until Jcode exposes complete blocked, approval-result, interrupt, and exit transitions.
7. Add integration versioning, replacement-source handling, schema/UI wiring, install/uninstall tests, restore-plan tests, detection fixtures, and documentation.

Herdr versions that do not include `jcode` in their built-in agent kind list may accept `pane.report_agent_session` with `{"type":"ok"}` but still omit Jcode from the agent panel. Use the custom harness state path above for visibility until Herdr ships official Jcode recognition.

Relevant upstream files as of Herdr commit `eacea2daf0b72973173b728936b27478374f2cd2`:

- `src/integration/{mod.rs,registry.rs,targets.rs,actions.rs,version.rs}`
- `src/integration/assets/`
- `src/api/schema/integrations.rs`
- `src/agent_resume.rs`
- `src/detect/mod.rs`
- `src/terminal/state.rs`

## Future full lifecycle authority

A later Jcode/Herdr protocol can report `working`, `idle`, `blocked`, and `unknown` through `pane.report_agent`, then call `pane.release_agent` on process exit. Do not enable this authority from turn hooks alone. It needs explicit Jcode events for permission/question blocking, approval resolution, cancellation/interrupt, reconnect/reload transfer, and abnormal termination so Herdr never displays a stale working or idle state.

### Working subagent summary metadata

The adapter accepts optional hook metadata for aggregate child-agent visibility:

- `JCODE_HOOK_SUBAGENTS_WORKING`
- `JCODE_HOOK_SUBAGENTS_BLOCKING`
- `JCODE_HOOK_SUBAGENTS_NON_BLOCKING`

The equivalent lowercase keys are also accepted in `JCODE_HOOK_PAYLOAD`. If the total is omitted, it is derived from the blocking and non-blocking counts. The Herdr machine state remains `working`; the counts are carried in the human-readable message so older Herdr versions continue to work unchanged.

### Agent row layout

The custom agent message is rendered as two lines:

```text
● <project name>
  <custom session name>
```

The project name is the basename of `JCODE_HOOK_CWD`. The session label comes from `JCODE_HOOK_SESSION_NAME`, which Jcode derives from the custom/generated session title and falls back to the session id. The status remains machine-readable in `pane.report_agent.params.state`.

Official references:

- <https://herdr.dev/docs/integrations/>
- <https://herdr.dev/docs/socket-api/>
- <https://herdr.dev/docs/agents/>
- <https://herdr.dev/docs/session-state/>
