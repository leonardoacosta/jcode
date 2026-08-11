# Notification Architecture (v5) — Full Reference

> Read this before wiring a new hook or command to spoken notifications. Router: `../SKILL.md`.

Claude Code imports notification capability from Herald's `notify@herald` plugin:

```
notify@herald plugin
  ├─ /notify                         operator status, history, mute, test, voices, send
  ├─ SessionStart hook               guidance + cross-shell PATH export
  ├─ bin/say_notify                  thin zsh-compatible adapter
  └─ lib/notify.sh                   say_notify (project detection + bounded timeout)
       ↓ preloaded by BASH_ENV; adapter delegates here
     $HERALD/bin/notify.sh           mute gate + argument handling + ssh transport
       ↓
     herald notify                   voice resolution, Kokoro synthesis, history append
       ↓
     configured playback host        audio playback
```

The plugin is registered as the local `herald` marketplace and installed as `notify@herald`.
Herald is the source of truth; central-claude owns only the import/preload configuration and
call sites. There is exactly one speech helper and one pipe.

## Caller contract

**Signature:** `say_notify [-p <project-code>] "<text>"`.

- The project code selects a configured voice from `projects.toml`. Omit it and the helper
  detects `$CLAUDE_PROJECT_DIR`, then the git toplevel, then a `~/dev/<code>` prefix.
- **Always returns 0.** A missing plugin, pipe, configuration, synthesis service, or playback
  host cannot break a hook, command, or session exit.
- **Bounded.** `$SAY_NOTIFY_TIMEOUT` (15 seconds by default) caps the helper; Herald separately
  bounds synthesis and transport.
- **Foreground only.** Do not use `run_in_background: true`; background task-result delivery can
  trigger another response and create a notification loop.

`settings.json` sets `env.BASH_ENV` to `scripts/lib/bash-env.sh`, which resolves
`$HERALD/plugin/lib/notify.sh`. Because Claude's Bash tool uses the operator's zsh login shell on
this host, the plugin's SessionStart hook also writes its `bin/` directory to `CLAUDE_ENV_FILE`;
the exported `bin/say_notify` adapter loads that same function. `say_notify` therefore resolves
without a source line in both shells:

```bash
say_notify "The deployment finished successfully."
```

Do not add a source line at a BASH_ENV call site. A sourceable CLI library that genuinely runs
outside that environment may resolve the same plugin helper once behind an idempotent guard.

## Operator control

Use `/notify` instead of editing state or probing the transport by hand:

- `/notify status` — environment, pipe/binary, mute, and Kokoro health configuration
- `/notify history [n]` — newest append-only history rows
- `/notify mute [duration]` / `/notify unmute` — shared Herald mute expiry
- `/notify test` — audible fixed phrase plus the resulting history row
- `/notify voices` — effective mappings from `herald notify voices --json`
- `/notify <text>` — send a message through the single pipe

Mute is Herald state, so it applies to every harness caller. A valid suppressed attempt still
appends one `muted` record; the other closed outcomes are `delivered`, `synth_failed`,
`transport_timeout`, and `transport_failed`.

## Debugging a missing notification

Check in order:

1. `/notify status` — catches an unset `$HERALD`, missing plugin source, pipe, or binary.
2. `/notify history 10` — authoritative delivered/failure/muted evidence; stderr from a
   fire-and-forget hook is not observability.
3. `/notify voices` — confirms the effective provider-qualified voice and speed.
4. `/notify test` — performs the bounded synthesis/playback round trip and prints its own row.
5. If transport fails, verify the deployment-configured playback route and remote player.

Never reproduce secret values while debugging. Service addresses remain environment/config
facts with documented Herald defaults; they are not baked into Go or generated compose output.
