# Execution evidence

Approval: full written revision approved 2026-09-26T06:36:28Z. No outstanding approval gate.

## Upstream prerequisite, 2026-09-26

Fresh disposable environment, not the previously installed review environment:

```sh
uv venv --python 3.12 scratch/jev-profile-proof/venv
uv pip install --python scratch/jev-profile-proof/venv/bin/python 'jev-ultrafast @ git+https://github.com/browser-use/jev-ultrafast@1231850a0bf1a0c0341fe408ef1668dbbfdfac46'
python3 scratch/jev-profile-proof/proof.py
python3 scratch/jev-profile-proof/dead.py
```

Resolved Python 3.12.14, jev-ultrafast 0.1.0, browser-harness 0.1.13. Public upstream MIT license. Actual installed dependency exposes `BU_CDP_WS`, `BU_CDP_URL`, `BH_RUNTIME_DIR`, `BH_TMP_DIR`, `require_existing_daemon`.

Real `/usr/bin/chromium` launched with `--headless=new --remote-debugging-port=0` and distinct disposable `--user-data-dir` directories. The fixture ran on an ephemeral loopback HTTP port. Real `jev_ultrafast.browser.Browser.evaluate` observed:

| Launch | Before | After |
| --- | --- | --- |
| alpha, initial | null | alpha |
| beta, initial | null | beta |
| alpha, fresh Chrome and Python process | alpha | alpha |
| beta, fresh Chrome and Python process | beta | beta |

Both browser directories started empty. Each subprocess used an explicit WebSocket endpoint read from its owned Chrome `DevToolsActivePort` and a separate private harness runtime directory. All four invocations returned fixture title `Profile isolation fixture`. Proof exited 0. Detailed log: `scratch/jev-profile-proof/proof.log`.

A separate fresh runtime used `BU_CDP_WS=ws://127.0.0.1:1/devtools/browser/dead`. `ensure_daemon(wait=1)` raised `RuntimeError`. The harness log recorded remote connection failure, with neither `DevToolsActivePort not found` nor `chrome://inspect` local-discovery paths. Proof exited 0. No personal profile or shared Jcode daemon was used or changed.

These are upstream capability checks, not complete Jcode acceptance. Jcode lifecycle, tool/CLI, cleanup, attachment, Rust tests and isolated binary acceptance remain pending.
