# Promote merged v0.89.3 locally

## Approved approach and alternatives

Use a fresh frozen-source build, isolated validation, then coordinated activation of launcher and shared daemon. Launcher-only activation leaves daemon behavior behind. Immediate reload skips acceptance evidence and risks active sessions. Preserve the v0.88 immutable binary for rollback.

## Source and validation boundary

Record actual main HEAD, dirty paths and active agents before building. Exclude generated client_sessions files from source commits and do not delete unrelated data. Build only the approved committed source plus explicitly recorded build metadata. If source changes during validation, rebuild and repeat affected checks before activation.

Run base/core/root regression suites and formatting on the candidate source. Record previously known limitations separately: full all-features/clippy and oversized-test budget are not proven clean. Start the exact built binary in a fresh private JCODE_RUNTIME_DIR and explicit socket, clearing inherited JCODE_SOCKET. Verify version and daemon request handling. Exercise one small real-provider response and read-only MCP discovery/config access using the approved local settings, without logging secrets or mutating remote data. If auth, network, MCP or runtime acceptance fails, repair within scope or stop before activation. Do not substitute a mock or old shared daemon result for these checks.

## Activation and rollback

Inventory active sessions and identify the pending test-reload-hash activation owner. Do not overwrite a live owner's activation or manually kill the shared daemon. Coordinate owners and use supported selfdev activation/reload mechanisms. Retain the exact previous shared-server/current targets and immutable binary. After candidate acceptance, promote both channels and verify the real running shared daemon version, launcher resolution, session continuity/reconnection and read-only MCP access. Roll back via the supported channel/reload mechanism if activation acceptance fails. Never reset credentials or change shared MCP configuration to make the check pass.

## Completion boundary

Report frozen source hash, binary/version, before/after channel targets, test and live smoke results, active-session coordination and rollback target. Commit only owned evidence artifacts. No release publishing or remote push. A blocked session owner or unsuccessful live check leaves promotion pending, not complete.
