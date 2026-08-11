---
name: dotenvx-secrets
description: Use dotenvx to inject, encrypt, and redact .env secrets — especially when running a coding agent (Claude Code, Codex) against real credentials. Triggers on dotenvx, `dotenvx run`, `--redact`, `--overload`, .env encryption, .env.keys, DOTENV_PRIVATE_KEY, `op://`/`bw://` secret references, 1Password service accounts, vault access, MCP secret injection, "secrets in Claude Code", "don't leak my API key", precommit .env guards, or migrating a repo off dotenv-cli. Covers what redaction does and — critically — what it does not protect against.
---


# dotenvx secrets

`dotenvx` is a drop-in `dotenv` replacement (BSD-3-Clause, dotenvx/dotenvx) that
injects `.env` values into a child process, encrypts `.env` files with secp256k1
keypairs, resolves vault references, and can redact secret values from the child's
output stream.

Its agent-relevant capability is `dotenvx run --redact -- <agent>`: the agent gets
the real credentials, but any occurrence of those literal values in its stdout/stderr
is replaced with `[REDACTED]` before reaching the terminal or transcript.

## Read this first: `--redact` is not a security boundary

Redaction is **best-effort exact literal substring matching** on the child process's
piped stdout and stderr. `src/lib/helpers/redactOutput.js` does
`result.split(sensitiveValue).join('[REDACTED]')` — no regex, no case-insensitivity,
no encoding awareness. It therefore does **not** catch:

- the secret base64-, hex-, or URL-encoded, JSON-escaped, uppercased, or split across
  markdown formatting the model inserted;
- the secret leaving through any channel other than that one child's stdout/stderr —
  outbound network calls the agent makes, files it writes to disk, telemetry, crash
  reports, or a grandchild process whose output isn't re-piped;
- partial disclosure ("my key starts with sk-proj-abc").

Treat it as a guard against an agent *literally echoing back the string it was handed*,
and nothing more. Never present it to a user as a reason it's now safe to hand an agent
production credentials. The right posture is still: scope the credential down, prefer a
short-lived token, and use `--redact` as defence in depth.

Two more behavioural facts worth stating before someone is surprised by them:

There is **no minimum length or entropy threshold** — `redactedValues.js` filters only
empty/null values, so `NODE_ENV=production` makes every occurrence of the word
"production" anywhere in the agent's output become `[REDACTED]`. Exempt those with the
`_PLAIN` key suffix (`/_PLAIN$/`, `isPlainKey.js`): `NODE_ENV_PLAIN=production`. The
same `_PLAIN` convention also excludes a key from encryption.

Values that were **already in the environment** are redacted too, not just injected
ones — `redactedValues.js` merges `processedEnv.injected` and `processedEnv.existed`,
so a pre-existing shell export that beat the `.env` file still gets filtered.

## Quickstart

```sh
curl -sfS https://dotenvx.sh | sh          # install
echo "HELLO=World" > .env

dotenvx run --redact -- claude -p 'Run `dotenvx get HELLO` and echo back just Hello VALUE'
# → Hello [REDACTED]

dotenvx run --redact -- codex exec '...' --skip-git-repo-check
```

Interactive sessions work too — when stdin/stdout/stderr are all TTYs on macOS or
Linux, dotenvx wraps the child in a pty via `script(1)` so the agent's TUI still
renders while output stays filtered (`ptyCommand.js`). On Windows, or where `script`
is absent, it falls back to plain pipes: redaction still applies, but an interactive
TUI may render poorly. Prefer `-p`/`exec` one-shots there.

## Portable convention

This fleet is mid-migration from `dotenv-cli` to `dotenvx` + `@t3-oss/env`, tracked by
the `fleet-env` informational validation row and fixed via
`scripts/bin/fleet-env-audit --fix <repo>`. That ratchet counts two drift classes:
`legacy_lines` (package.json scripts still on `dotenv-cli` syntax) and
`dotenvx_missing_overload`.

**Always pass `--overload` at fleet call sites.** By default dotenvx lets existing
environment variables win over the `.env` file; `--overload` inverts that so the file
is authoritative. A migrated call site without it is what the ratchet flags.

```jsonc
// package.json — the shape the ratchet expects
"scripts": {
  "dev": "dotenvx run --overload -- next dev"
}
```

## Reference

| Need | Read |
|---|---|
| What `--redact` matches, its stream plumbing, `_PLAIN`, over-redaction, and every known gap | `references/redaction.md` |
| `encrypt`/`decrypt`/`rotate`, `.env.keys`, `DOTENV_PUBLIC_KEY`/`DOTENV_PRIVATE_KEY`, per-environment keys, CI injection | `references/encryption.md` |
| `op://` (1Password) and `bw://` (Bitwarden) references, plus the `precommit`/`gitignore`/`prebuild` guards | `references/vaults-and-guards.md` |

## Rules

When `OP_SERVICE_ACCOUNT_TOKEN`, 1Password vault access, or MCP secret injection is involved,
read `references/vaults-and-guards.md` first. MCP never bypasses vault permissions. The fleet
default for a user-created service-account vault requires verified human `allow_viewing` plus
`allow_editing`; machine-only or view-only human access is incomplete, and managing access
requires an explicit user decision.

Never write a real secret value into a file you create, a commit message, a PR body,
or a task description — `--redact` filters the agent's stdout, not the files it writes.

Never run `dotenvx decrypt` and leave the result on disk. If you need a plaintext value
for one command, use `dotenvx get KEY` or `dotenvx run -- <cmd>` and let it stay in
process memory.

Before proposing that a repo commit its `.env`, check it is encrypted (`dotenvx encrypt`)
and that `.env.keys` is gitignored. `.env.keys` holds the private decryption keys and
must never be committed; park it in 1Password or Armor.

For deployed environments, the active deployment vendor is the broker for the
environment-specific dotenvx private key: for example, Vercel, GitHub Actions,
or Depot supplies `DOTENV_PRIVATE_KEY_<ENVIRONMENT>` through its secret facility.
Keep application secrets in their tracked, encrypted `.env.<environment>` file;
do not duplicate each application secret in the deployment vendor. A deployment
that cannot supply the selected private key cannot decrypt its environment file
and MUST fail closed before application startup.

When adding dotenvx to a repo, install the guard in the same change:
`dotenvx gitignore` then `dotenvx precommit --install`.
