# Daemon skill automations and bulletin

Status: implementation checkpoint, not production-verified. See `verification.md` for remaining gates.

## Local startup

```sh
jcode --no-update automations init
jcode --no-update server start
jcode --no-update automations status
jcode --no-update automations open
```

`init` captures the current directory, configured daemon socket, optional model, and a private loopback port. It does not install a service. An already-running older daemon needs an explicit update/restart before it can recognize this configuration. `open` prints a sensitive five-minute single-use pairing link. Open it only in the intended browser. Browser sessions expire after 24 hours. Log out to revoke the current session.

Choose an installed skill, optional arguments, and either an interval in seconds (minimum 60) or comma-separated weekdays (Monday 0 through Sunday 6), local HH:MM time, and IANA timezone. Calendar previews show the next three occurrences. Nonexistent daylight-saving times are skipped, repeated times run at their earlier instant. Missed downtime occurrences are skipped. Runs are serial with a 30-minute deadline. Cancellation retains ownership until execution acknowledges shutdown.

Results retain the latest 1,000 terminal records, at most 64 KiB per final response. Output is escaped, and known credentials are masked, but arbitrary skill output may contain sensitive information. Review skill permissions and recurring model costs before enabling a schedule.

## Private Tailscale HTTPS

```sh
jcode automations tailnet --https-port 8443 --confirm
jcode automations open --tailnet
```

Requires an installed, logged-in Tailscale node, existing HTTPS readiness, and network policy permitting the intended device. Provisioning refuses occupied/unowned routes and does not log in, broaden ACLs, or enable Funnel. Browser pairing is required even for reachable tailnet devices. Confirm the URL on a second device before claiming remote readiness.

```sh
jcode automations tailnet --https-port 8443 --disable --confirm
```

Removal checks the owned route and preserves unrelated Serve configuration. Ownership receipts remain after uncertain failures for manual inspection rather than destructive retries.

## Linux user service

```sh
jcode automations service --confirm
# Explicitly permit operation after logout:
jcode automations service --confirm --enable-linger
```

Use the appropriate command on initial installation. Installation refuses to compete with an unmanaged live daemon or overwrite an unowned unit. It uses the existing launcher, preserving its selected build channel. Without linger, the service is not login-independent. A prior installation receipt must be inspected before reconfiguration. No password or account changes occur.

```sh
jcode automations service --uninstall --confirm
```

Uninstall preserves automation definitions/history. It does not remove Tailscale serving, which has a separate scoped removal command. Pause automation definitions before rollback. Keep the feature disabled until the remaining acceptance gaps in `verification.md` are resolved.
