
# Encryption, `.env.keys`, and key handling

## The model

`dotenvx encrypt` generates a keypair, writes the **public** key into the `.env` file
itself and the **private** key into `.env.keys`, then replaces each value in `.env` with
its ciphertext. Because the public key lives in the committed file, anyone can *add* new
encrypted values without holding the private key — that asymmetry is the whole point.

```sh
dotenvx encrypt                 # encrypt .env in place
dotenvx encrypt -f .env.production
dotenvx set HELLO World         # add an encrypted value (auto-generates keys if absent)
dotenvx run -- node index.js    # decrypt + inject at runtime
dotenvx get HELLO               # print one decrypted value
dotenvx decrypt                 # ⚠ writes plaintext back to disk — avoid
dotenvx rotate                  # new keypair, re-encrypt
```

Resulting `.env`:

```ini
#/-------------------[DOTENV_PUBLIC_KEY]--------------------/
DOTENV_PUBLIC_KEY="0339d..."
HELLO="encrypted:BLx4..."
```

Resulting `.env.keys` — **never commit this**:

```ini
# .env
DOTENV_PRIVATE_KEY="[provided-by-dotenvx]"
# .env.production
DOTENV_PRIVATE_KEY_PRODUCTION="[provided-by-dotenvx]"
```

## Key naming per environment

`src/lib/conventions/keynames.js`: `.env` maps to `DOTENV_PUBLIC_KEY` /
`DOTENV_PRIVATE_KEY`; `.env.<environment>` maps to `DOTENV_PUBLIC_KEY_<ENVIRONMENT>` /
`DOTENV_PRIVATE_KEY_<ENVIRONMENT>`, uppercased. If the file already carries a
`DOTENV_PUBLIC_KEY*` variable under a custom name, that existing name wins.

## What is skipped during encryption

`src/lib/transforms/encrypt.js` skips two classes of key:

```js
if (isDotenvPublicKey(key) || isPlainKey(key)) continue
```

The public-key line itself, and anything matching `/_PLAIN$/`. This is the same `_PLAIN`
convention `--redact` uses, so a key suffixed `_PLAIN` is both left in cleartext and left
out of redaction — consistent, and worth using for genuine non-secrets like `NODE_ENV`.

Values that are already ciphertext are passed through untouched rather than
double-encrypted (`if (encrypted(value)) { transformedValues.push(value) }`), so
re-running `encrypt` is idempotent.

## CI and production injection

Don't ship `.env.keys`. Pass the private key as an environment variable instead:

```sh
DOTENV_PRIVATE_KEY_PRODUCTION="[provided-by-dotenvx]"
```

Multiple keys can be combined (`DOTENV_PRIVATE_KEY=... DOTENV_PRIVATE_KEY_PRODUCTION=...`)
when a run needs to decrypt more than one file.

## Restricted `.env.keys` permissions

`chmod a-r .env.keys` keeps working for `encrypt`/`set`, because those only need the
public key that lives in `.env`. `src/lib/transforms/encrypt.js` catches `EACCES`/`EPERM`
when reading the keys file and continues — this is code-confirmed, not just documented.
`run`/`get`/`decrypt` still need read access.

## Cipher — DOCS-ONLY

The `dotenvx` repo imports its primitives (`keypair`, `encrypt`, `scan`, `sealed`,
`publickeys`) from the separate `@dotenvx/primitives` npm package. The claims that this
is secp256k1 ECIES with a unique ephemeral key per secret and AES-256 come from the
dotenvx README/FAQ and the `.env.keys` docs page, **not** from code read in the main
repo. Treat the cipher choice as documented-but-unverified here; read
`@dotenvx/primitives` directly if the specifics are load-bearing for a decision.

## Storage of the private key

Options, in rough order of preference for a single-developer fleet: 1Password (and then
reference it back via `op://`, see `vaults-and-guards.md`), Dotenvx Armor (their hosted
key service — `dotenvx encrypt` will offer to POST the private key there instead of
writing `.env.keys` locally; `--no-armor` opts out), or a local gitignored `.env.keys`
with restricted permissions. Never a plaintext key in CI logs, a task description, or a
commit.
