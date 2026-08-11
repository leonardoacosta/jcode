# Mac Browser Fleet Verification Report

Date: 2026-08-11
Commit under verification: `6da6f4f` plus durable setup commit `864dd0f`
Environment: Linux homelab source checkout plus the configured `mac` SSH host, an arm64 macOS machine with Chrome and Edge installed.

## Interpreted acceptance boundary

The implementation follows the user-approved design: ordinary Mac Chrome and Edge tabs use a Manifest V3 extension and native messaging, explicitly managed instances may use loopback CDP, homelab Jcode selects the fleet only with `browser: "mac"`, transport is a reverse SSH Unix socket, and the Mac remains the authority for approval, leases, hard denies, and emergency stop. Before approval, the only implementation choice inferred was using a dedicated provider rather than changing local `browser: "chrome"` behavior. The approved OpenSpec subsequently made that choice explicit.

## Requirement traceability

| Requirement | Concrete check and observed result | Status |
|---|---|---|
| Mac browser fleet discovery | Extension inventory/action suites passed as part of all 37 Node tests. Broker generation handling passed in `protocol_broker_cdp.rs`. Through the real public Jcode `browser: "mac"` interface, `list_tabs` returned 15 managed Chrome page targets from the Mac with normalized browser, window, tab, and generation references. Ordinary-profile extension discovery remains pending browser approval. | Managed Chrome live pass, ordinary-profile approval pending |
| Explicit remote fleet routing | `scripts/dev_cargo.sh test -p jcode-app-core browser_tests:: --no-default-features` passed 15 tests with one opt-in live smoke ignored. Tests proved explicit `mac` routing, local Chrome profile preservation, schema fields, bounded request mapping, and stale/approval error meaning. | Pass |
| Private authenticated SSH transport | The built `jcode-mac-browser-fleet` binary was started through its public CLI and contacted over its public Unix-socket protocol. Authenticated health succeeded, invalid authentication failed without revealing the secret, and `stat` reported socket mode `600`. On the real Mac, launchd ran the broker and both the Mac socket and homelab forwarded socket reported mode `600`. Jcode queried the broker through SSH before and after a launchd broker restart. | Installed boundary pass |
| Mac-owned confirmation policy | `cargo test --manifest-path crates/jcode-mac-browser-policy/Cargo.toml` passed 10 table-driven policy tests. Broker tests proved read-only health succeeds while mutation requests return `approvalRequired`. The Mac-local approval UI is not implemented. | Engine pass, UI incomplete |
| Scoped expiring autonomy leases | Policy tests cover scope matching, the 15-minute maximum, monotonic expiration, restart/policy/target invalidation, and emergency-stop revocation. No installed Mac lease workflow was available. | Engine pass, live blocked |
| Immutable hard-deny boundaries | Policy table tests cover incognito, password managers, settings, extensions, privileged URLs, payment/banking confirmation, account security, authentication/recovery, and emergency stop. Category-level errors are secret-safe. | Pass |
| Capability-faithful hybrid control | All 37 extension tests passed. `cargo test -p jcode-mac-browser-fleet --test protocol_broker_cdp` passed six managed-CDP trust, framing, loopback restriction, multi-source merge, generation, and replay tests. On the real Mac, the loopback Chrome endpoint reported Chrome 151 and Jcode returned 15 filtered page targets through the broker and SSH Unix-socket bridge. A public navigation request returned `approval required` rather than bypassing Mac authority. | Managed read-only routing live, approved mutation incomplete |
| Safe Mac lifecycle and setup | `cargo test -p jcode-mac-browser-setup` passed 4 lifecycle/rendering tests covering install/status/remove, mode-0600 secret, operator-file backup, managed CDP launch arguments, XML escaping, manifest and SSH rendering, profile preservation, and unrelated SSH preservation. On the real arm64 Mac, the native broker and setup binaries were rebuilt from the exact current sources, launchd ran the broker with a loopback-only Chrome CDP endpoint, and both broker and forwarded sockets reported mode `600`. | Installed managed-CDP pass, extension approval pending |
| Browser fleet verification | Package formatting, policy/broker/setup tests, all extension tests, Jcode browser-provider tests and checks, `git diff --check`, and strict OpenSpec validation passed. Through Jcode's public `browser: "mac"` interface, status reported ready and `list_tabs` returned 15 real managed Chrome page targets. A targeted navigation remained blocked with `approval required`, proving the policy boundary was preserved. Ordinary Chrome/Edge extension attachment and approved mutation execution remain incomplete. | Real read-only acceptance pass, controlled mutation blocked |

## Public-interface observation

The real broker executable accepted this public invocation shape:

```bash
jcode-mac-browser-fleet broker \
  --socket PATH \
  --peer-secret PATH \
  --policy PATH
```

Observed protocol results:

- Correct secret and `listBrowsers`: `ok: true`, health generation `1`, 15 filtered managed Chrome page targets.
- Jcode `browser: "mac"`, `list_tabs`: returned those 15 targets with normalized `browser_ref`, `window_ref`, `tab_ref`, and `generation` fields.
- Jcode navigation against a current target: rejected with `Mac browser fleet approval required on the Mac`.
- A separate Mac-local authority socket was installed with mode `600`; it is not part of the SSH-forwarded peer socket.
- A two-minute `navigate` lease was granted locally for a dedicated ordinary-origin target. Jcode navigation through the public `browser: "mac"` path returned `accepted` and changed that target through CDP.
- A subsequent real Chrome `/json/list` observation found the same target at the expected `https://example.org` origin, confirming the accepted response represented an actual browser-state change rather than only protocol acknowledgement.
- Revoking the lease immediately restored `approval required` for the same target.
- Emergency stop overrode a newly granted lease. Releasing emergency stop left no active elevated authority.
- After a Jcode server reload removed the ephemeral forward, the SSH StreamLocal socket was recreated at mode `600`; `browser list_tabs browser=mac` returned 17 live targets and a mutation still returned approval-required, confirming bridge recovery and no residual authority.
- A forged `grantLease` authority envelope sent through the SSH-forwarded peer socket was rejected as `malformed`; the separate Mac-local authority status remained unchanged, proving remote peers cannot mint leases through the forwarded channel.
- Incorrect secret: `ok: false`, error kind `unauthenticated`, diagnostic `fleet authentication failed`.
- The rejected response contained no peer secret.
- The bound Unix socket mode was `600`.

This proves the executable, argument parser, filesystem boundary, authentication boundary, response encoding, real arm64 launchd lifecycle, Unix-socket SSH forwarding, public Jcode provider routing, live managed Chrome target discovery, normalization, approved navigation, immediate lease revocation, emergency-stop precedence, approval-required behavior, stale-generation behavior, and read-only recovery after broker restart. It does not prove ordinary-profile extension attachment, selector-based interactions, or Edge target routing.

## Remaining acceptance handoff

A Mac-attached session must still:

1. Approve and load the unpacked Chrome/Edge extension, then install native-host manifests with the real extension ID.
2. Complete or supply the Mac-local approval, lease, revocation, and emergency-stop surface.
3. Exercise ordinary-profile Chrome and Edge independently through Jcode's public `browser: "mac"` interface.
4. Verify approved mutation, lease use and revocation, hard denial, cleanup, and no remaining lease or process.

Until those steps pass, the repository and installed Mac broker provide a live, secure read-only managed Chrome fleet plus a verified foundation for explicitly approved automation. They are not yet a completed unrestricted browser-steering product.

## Runtime deployment findings (2026-08-11)

Deploying the extension-bridge build to the real Mac exposed four defects that the deterministic suites could not see, because each only manifests against a real macOS kernel, a real sshd, or a real launchd lifecycle. Each was fixed test-first, with a regression test that fails against the old behavior.

| Defect | Symptom on the Mac | Fix |
| --- | --- | --- |
| Socket path over the `sun_path` limit | Broker exited with `could not bind authority socket` while every installed file looked correct | Sockets moved to `~/.jcode/mac-fleet/`; `rendered_socket_paths_stay_within_the_unix_domain_socket_limit` pins the 103-byte budget |
| `~` in the `RemoteForward` listen path | Every reverse tunnel failed with `remote port forwarding failed for listen path` | Render an absolute `/home/<user>/...` path; `ssh_include_forwards_to_an_absolute_remote_socket_path` forbids tilde paths |
| Stale native-host defaults | Chrome launches the host with no arguments, so it silently targeted the old socket and secret | Shared `default_broker_socket_path` and `default_native_secret_path` helpers; `native_host_defaults_match_the_installed_broker_socket_layout` keeps both crates aligned |
| Unsupervised reverse tunnel | A manual `ssh -N` died on sleep, network loss, or session reload, leaving a stale socket and a dead bridge | `dev.jcode.mac-browser-fleet-tunnel` LaunchAgent with `KeepAlive`; `tunnel_launch_agent_keeps_the_reverse_forward_alive` pins the supervision flags |

Observed after the fixes, through Jcode's public interface:

- `browser list_tabs browser=mac` returned 10 live managed Chrome targets.
- Killing the broker: launchd relaunched it and discovery recovered.
- Killing the tunnel: the forwarded socket was rebound automatically within 15 seconds and discovery recovered.

Operator note: `jcode-mac-browser-setup install` writes the SSH include file but deliberately does not edit `~/.ssh/config`. The `Include ~/.ssh/jcode-mac-browser-fleet.conf` line must be present in that config, and an unrelated rewrite of the file will silently drop the tunnel.
