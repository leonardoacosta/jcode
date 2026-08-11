
# Vault references (`op://`, `bw://`) and commit guards

## 1Password references

Put the reference, not the secret, in `.env`. dotenvx resolves it at inject time.

```sh
echo "HELLO=op://Personal/hello/password" > .env
dotenvx run -- sh -c 'echo Hello $HELLO'
# Hello World
```

Requires the 1Password CLI (`op`) installed and signed in.

### Authentication and access boundaries

MCP does not change the identity used by 1Password. When `OP_SERVICE_ACCOUNT_TOKEN` is
present, `op` runs with that service account's vault scope instead of interactive desktop-user
access. A local MCP launched with desktop/user authentication can technically read that user's
Personal vault, but exposing general vault-read tools to a model expands the secret-reading
surface and is not the fleet default.

1Password service accounts cannot access built-in Personal, Private, Employee, or default
Shared vaults. Their vault access and permissions are immutable after creation. Use a
purpose-built user-created vault; create a new service account when machine scope must change.
See [1Password service account limitations](https://www.1password.dev/service-accounts/get-started).

### Fleet default: human view and edit access

Every user-created vault created or used by a service account MUST also grant its intended
human owner `allow_viewing` and `allow_editing`. Machine-only or view-only human access is not
a completed fleet setup. Do not grant `allow_managing` by default. Resolve the active human
from account metadata rather than hardcoding an email address or UUID, and stop for
clarification if more than one active human matches.

The following flow passes only non-secret metadata in argv and suppresses raw access-list
output:

```sh
vault_name="automation-secrets"
human_name="Intended Human Owner"

vault_id="$(op vault get "$vault_name" --format json | jq -er '.id')"
human_id="$(
  op user list --format json |
    jq -er --arg name "$human_name" \
      '[.[] | select(.name == $name and .state == "ACTIVE")] |
       if length == 1 then .[0].id else error("expected exactly one active human owner") end'
)"

op vault user grant \
  --vault "$vault_id" \
  --user "$human_id" \
  --permissions allow_viewing,allow_editing

op vault user list "$vault_id" --format json |
  jq -e --arg user "$human_id" \
    '.[] | select(.id == $user) |
     (.permissions | index("allow_viewing") != null and index("allow_editing") != null)' \
  >/dev/null
```

Creating an item is not completion by itself. Verify item metadata without printing fields,
then verify human access again after first provisioning:

```sh
op item get "$item_name" --vault "$vault_id" --format json |
  jq -e '.id and .title and .vault.id' >/dev/null

op vault user list "$vault_id" --format json |
  jq -e --arg user "$human_id" \
    '.[] | select(.id == $user) |
     (.permissions | index("allow_viewing") != null and index("allow_editing") != null)' \
  >/dev/null
```

Vault sharing remains a permission change even when the intended human is the operator. Never
infer a different recipient, and never print item field values while validating. See
[1Password vault access management](https://support.1password.com/create-share-vaults-teams/).

### MCP usage

Prefer explicit `op://<vault>/<item>/<field>` references resolved with `op run` for one
specific MCP server. This gives the process only the named secrets and does not expose a
general-purpose vault-reading tool to the model. A service-account-backed MCP remains subject
to the same service-account vault restrictions.

### How it resolves

`src/lib/helpers/resolveOnePassword.js` shells out with an argv array, not a shell
string:

```js
const stdout = await execFileAsync('op', ['read', value, '--no-newline'], {
  encoding: 'utf8',
  windowsHide: true
})
parsed[key] = stdout
```

Because it's `execFile` with an argv array, a reference containing shell metacharacters
is passed verbatim as one argument — the repo's own test asserts that
`op://vault/item/password; echo unsafe` is not shell-interpreted.

### When `op` is missing

```js
const message = error && error.code === 'ENOENT'
  ? `1Password CLI is not installed and could not resolve ${key}`
  : `1Password CLI failed to resolve ${key}`
```

On failure the key is pushed to `unresolved` and `parsed[key]` is **not** reassigned —
the literal `op://...` string stays as the value. Your program receives the reference
string rather than the secret, which usually surfaces as a confusing downstream auth
error. Pair vault references with `--strict` so the run exits non-zero instead of
limping onward with a bogus value.

A useful interaction with redaction: an unresolved reference was never injected as a real
secret, so there is nothing for `--redact` to filter in that failure path.

Disable resolution entirely with `--no-1password` or `DOTENVX_NO_1PASSWORD=true`.

## Bitwarden references

Same shape: `bw://<item>/<field>` where field is `username`, `password`, or `uri`,
resolved via `bw get <field> <item>`
(`src/lib/helpers/resolveBitwardenPassword.js`). Uses `BW_SESSION` if set, otherwise
prompts for the master password when a TTY is available. On a missing session it tells
you to run `export BW_SESSION="$(bw unlock --raw)"`. Disable with `--no-bitwarden` or
`DOTENVX_NO_BITWARDEN=true`.

## Commit guards

### `dotenvx gitignore`

Appends a pattern (default `.env*`) to `.gitignore`, creating it if absent, and also to
`.dockerignore`, `.npmignore`, and `.vercelignore` **if those already exist**
(`src/cli/actions/ext/gitignore.js`).

### `dotenvx precommit`

Fails if any `.env*` file that would be committed is neither gitignored nor encrypted.
`src/lib/services/precommit.js`:

```js
const output = childProcess.execSync('git diff HEAD --name-only').toString()
return files.includes(filePath)
```

Note the check is `git diff HEAD`, i.e. **any working-tree change against HEAD — staged
or unstaged** — not strictly the index. It also fails safe: if the directory isn't a git
repo, or the git call errors, the file is treated as "will be committed" and checked
anyway.

The gate itself:

```js
const encrypted = sealed(src)
if (!encrypted) {
  throw new Errors({
    message: `${file} not encrypted/gitignored`,
    help: `fix: [dotenvx encrypt -f ${file}] or [dotenvx gitignore --pattern ${file}]`
  }).custom()
}
```

`.env.example` and `.env.x` are exempt.

### `dotenvx precommit --install`

Writes an executable (mode 755) `.git/hooks/pre-commit`
(`src/lib/helpers/installPrecommitHook.js`):

```sh
#!/bin/sh
if command -v dotenvx 2>&1 >/dev/null
then
  dotenvx precommit
elif npx dotenvx -V >/dev/null 2>&1
then
  npx dotenvx precommit
else
  # ...
  exit 1
fi
```

Two things to flag when recommending this. It is opt-in and **per clone** — `npm install`
does not install it, so a teammate or a fresh worktree has no guard until they run it.
And if `dotenvx` is reachable via neither PATH nor `npx`, the hook `exit 1`s, blocking
commits rather than silently passing; that's the safe default but it will confuse anyone
who hasn't installed dotenvx.

If a hook already exists, dotenvx appends to it rather than clobbering it, and no-ops if
`dotenvx precommit` is already present.

### `dotenvx prebuild` — DOCS-ONLY

Documented as a Docker-build guard (`RUN dotenvx prebuild` in a Dockerfile) that stops
plaintext `.env` files from being baked into an image. Only the CLI registration was
confirmed in source this pass; the exact mechanism (delete vs. error-on-presence) was not
read. Verify before relying on it.
