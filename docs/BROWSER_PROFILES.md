# Local browser profiles

Browser automation uses `jev-ultrafast` commit `1231850a0bf1a0c0341fe408ef1668dbbfdfac46` and `browser-harness==0.1.13`, with local Chromium on Unix. Windows currently returns an unsupported-host error. Setup is optional and explicit. Normal startup and tool calls never install packages.

```sh
jcode browser setup
uv venv ~/.jcode/browser-runtime --python 3.12
uv pip install --python ~/.jcode/browser-runtime/bin/python 'jev-ultrafast @ git+https://github.com/browser-use/jev-ultrafast@1231850a0bf1a0c0341fe408ef1668dbbfdfac46'
export JCODE_BROWSER_PYTHON="$HOME/.jcode/browser-runtime/bin/python"
export JCODE_BROWSER_CHROME=/usr/bin/chromium
jcode browser-profiles '{"action":"create","profile":"work","lifetime":"persistent"}'
jcode browser-profiles '{"action":"default","profile":"work"}'
jcode browser status
```

The browser tool exposes `create`, `attach`, `list`, `inspect`, `select`, `default`, `detach`, `close`, `recover`, `status`, `goal`, `observe`, and `evaluate`. Profile labels match `[A-Za-z0-9][A-Za-z0-9_-]{0,63}`. Creation requires `lifetime: temporary|persistent`. No profile or default is implicitly created.

`goal` runs the upstream Agent loop. `observe` reads an upstream page snapshot. `evaluate` evaluates a JavaScript expression through upstream Browser. These actions require an HTTP(S) `url`. Every operation launches an owned Chrome process in the selected profile and closes its owned browser afterward. Profile bindings persist until session close. A call cannot switch an already-bound session to another profile. `default` affects new sessions only.

System One endpoint, credentials and model come from Jcode's existing resolver for `goal`. They travel over subprocess stdin, not command arguments or diagnostics. Website text remains untrusted. Selecting an attached label authorizes existing signed-in sessions, not purchases, message sending or other consequential actions.

Attach existing state without copying it:

```sh
jcode browser-profiles '{"action":"attach","profile":"existing","path":"/absolute/chrome-user-data","directory":"Default"}'
jcode browser-profiles '{"action":"select","profile":"existing"}'
jcode browser-profiles '{"action":"close"}'
jcode browser-profiles '{"action":"detach","profile":"existing"}'
```

An existing browser lock causes a precise refusal. Jcode never removes Chrome locks, changes external permissions, kills an unrelated browser or discovers another personal browser. Duplicate registrations of the same canonical user-data directory and profile identity are rejected, including aliases. Distinct directories such as `Default` and `Profile 1` may share a root, with root-level locking preventing simultaneous launches. Attached state is never passed to managed deletion.

Managed state lives under `$JCODE_HOME/browser-profiles` (normally `~/.jcode/browser-profiles`). Version-1 metadata uses stable UUID paths, labels, ownership/lifetime, an explicit default and session bindings. Directories are private (`0700`), metadata is `0600`, writes are atomic and conflicting operations use file locks. Corrupt metadata fails closed without replacing the corrupt file. Restore only a verified backup after investigating.

Temporary profiles belong to their creator session and cannot become global defaults. True owner closure cleans up after browser handles release. Reload/disconnect while processing does not imply owner closure. `recover` checks recorded owner PID/start identity and browser locks, retaining uncertain state as pending cleanup. Recovery on platforms without `/proc` retains state rather than guessing.

Deletion is absent from the agent tool and requires a separate user CLI flag:

```sh
jcode browser-profiles '{"action":"delete","profile":"work"}' --confirm-delete
```

Active profiles refuse deletion. Use `--session ID` to address an explicit CLI session. Attached registrations use `detach`, which only removes metadata. Old bridge files remain untouched and may be removed manually after verification.
