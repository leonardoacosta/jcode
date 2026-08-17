## 1. Standalone runtime

- [x] 1.1 Add a dependency-free Node server that serves the built SPA, exposes `/healthz`, and safely proxies `/api/command-center/*` to Jcode.
- [x] 1.2 Add focused tests for static assets, SPA fallback, request bodies, response streaming, upstream failure, traversal rejection, and configuration validation.

## 2. Installation and lifecycle

- [x] 2.1 Add concise `install-command-center.sh` at the repository root with prerequisite checks, locked build, staged release installation, and atomic `current` activation.
- [x] 2.2 Have the installer write a hardened systemd user unit and environment file, daemon-reload, enable/restart, and health-check the service.
- [x] 2.3 Add rollback to the previous release when post-activation startup or health verification fails.
- [x] 2.4 Attempt non-interactive user lingering enablement and report an actionable command when policy prevents it.
- [x] 2.5 Add installer contract tests covering unit content, first install, repeat install, failure rollback, and paths containing spaces.

## 3. Compatibility and documentation

- [x] 3.1 Keep the daemon-hosted asset mode available as an explicit fallback and document the standalone service as the preferred deployment topology.
- [x] 3.2 Document configuration, status/log commands, upgrades, rollback behavior, and uninstall steps.

## 4. Verification

- [ ] 4.1 Run Command Center formatting, lint, typecheck, component tests, server tests, and production build. Formatting, typecheck, server tests, and build pass; the pre-existing component suite currently has four failures in concurrently modified UI files outside this change.
- [x] 4.2 Run installer contract tests and strict OpenSpec validation.
- [x] 4.3 Install the real user service and verify enabled/active state, `/healthz`, SPA loading, API bridge behavior, failed-process restart, and re-run upgrade behavior.
- [x] 4.4 Verify the Jcode binary checksum is unchanged across a UI-only install.
