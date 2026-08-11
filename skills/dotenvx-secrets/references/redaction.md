
# `dotenvx run --redact` — mechanism, guarantees, and gaps

Every claim here was verified against `dotenvx/dotenvx@main` source, not README prose.
Citations are `path` within that repo.

## Flag declaration

`src/cli/dotenvx.js`, in the `run` command chain:

```js
.option('--redact', 'redact injected values except keys ending in _PLAIN', false)
```

Boolean, defaults `false`. Sibling flags on the same command that matter here:

```js
.option('-o, --overload', 'override existing env variables (by default, existing env vars take precedence over .env files)')
.option('--mask [characters]', 'inject masked values, optionally setting visible characters')
.option('--strict', 'process.exit(1) on any errors')
.option('--no-1password', 'disable 1Password secret reference resolution')
.option('--no-bitwarden', 'disable Bitwarden secret reference resolution')
.option('--no-native', 'disable OS secret store features')
.option('--no-armor', 'disable Dotenvx Armor features')
```

`--mask` is a different thing from `--redact`: mask alters the value *injected into the
child*, redact alters what the child *prints back*. Masking an API key means the agent
gets a broken key; redacting means it gets the real one and can't echo it verbatim.

## What counts as a sensitive value

`src/lib/helpers/redactedValues.js` (full file):

```js
function redactedValues (processedEnvs) {
  const result = new Set()
  for (const processedEnv of processedEnvs || []) {
    const values = {
      ...(processedEnv.injected || {}),
      ...(processedEnv.existed || {})
    }
    for (const [key, value] of Object.entries(values)) {
      if (isPlainKey(key)) continue
      if (value === undefined || value === null || value === '') continue
      result.add(`${value}`)
    }
  }
  return [...result]
}
```

Two consequences.

`existed` is merged in alongside `injected`, so a variable already exported in the
shell — one that beat the `.env` file under default precedence — is still redacted from
output. This is the correct behaviour and is easy to miss.

There is no length, entropy, or shape filter. The only exemption is the key-name pattern
in `src/lib/helpers/cryptography/isPlainKey.js`:

```js
const PLAIN_KEY_PATTERN = /_PLAIN$/
```

So `NODE_ENV=production`, `DEBUG=true`, or `PORT=3000` will blank out every occurrence of
"production", "true", and "3000" in the agent's output. Rename them with the suffix —
`NODE_ENV_PLAIN=production` — to opt out. The same suffix also excludes a key from
`dotenvx encrypt`.

## The matching semantics

`src/lib/helpers/redactOutput.js`:

```js
function normalizedValues (values) {
  return [...new Set((values || [])
    .filter(value => value !== undefined && value !== null && `${value}`.length > 0)
    .map(value => `${value}`))]
    .sort((a, b) => b.length - a.length)
}
...
let result = value
for (const sensitiveValue of values) {
  result = result.split(sensitiveValue).join(redact(sensitiveValue))
}
```

and `src/lib/helpers/redact.js`:

```js
function redact (str) {
  if (!str || str.length < 1) return ''
  return '[REDACTED]'
}
```

`String.split(x).join(y)` is a global literal replace. There is no `RegExp`, no
`.toLowerCase()`, no decoding step anywhere in the module. The longest-first sort exists
so a short secret that happens to be a substring of a longer secret doesn't shadow it.

## The stream plumbing (this part is actually careful)

`redactOutput.js` also ships `partialMatchLength`, `safeBoundary`, and
`createRedactedStreamWriter`, built on a `StringDecoder`:

```js
const write = (chunk) => {
  pending += Buffer.isBuffer(chunk) ? decoder.write(chunk) : `${chunk}`
  const holdbackLength = partialMatchLength(pending, values)
  let boundary = pending.length - holdbackLength
  boundary = safeBoundary(pending, boundary, values)
  const output = pending.slice(0, boundary)
  pending = pending.slice(boundary)
  writeToStream(output)
}
```

It holds back any suffix that could be the start of a secret until the next chunk
arrives, so a secret straddling two `data` events — or split mid-UTF-8-codepoint — is
still caught. The repo's own tests cover "redacts values split across chunks" and
"redacts a unicode value split inside a character". The chunk-boundary class of bug is
genuinely handled; the weakness is entirely in the matching semantics above.

## Which streams

`src/lib/helpers/executeCommand.js`:

```js
child = execute.execa(executedArgs[0], executedArgs.slice(1), {
  stdio: ['inherit', redactStdout ? 'pipe' : 'inherit', redactStderr ? 'pipe' : 'inherit'],
  buffer: false,
  env: { ...process.env, ...env }
})

if (redactStdout && child.stdout) { /* createRedactedStreamWriter(process.stdout, ...) */ }
if (redactStderr && child.stderr) { /* createRedactedStreamWriter(process.stderr, ...) */ }
```

Both stdout and stderr are filtered. stdin is the hard-coded literal `'inherit'` and is
never touched — correct, since that's input going in, but note that terminal echo of
anything *you* type is unfiltered.

## Interactive sessions and the pty wrapper

`src/lib/helpers/ptyCommand.js`:

```js
function ptyCommand (commandArgs, platform = process.platform) {
  if (!['darwin', 'linux'].includes(platform)) return null
  let script
  try { script = path.resolve(which.sync('script')) } catch (e) { return null }
  if (platform === 'darwin') return [script, '-q', '/dev/null', ...commandArgs]
  return [script, '-qefc', commandArgs.map(shellQuote).join(' '), '/dev/null']
}
```

Used only when stdin, stdout, and stderr are all TTYs. Returns `null` on Windows or when
`script(1)` is missing, in which case dotenvx falls back to plain pipes — redaction still
applies, but a full-screen agent TUI may render badly. On those platforms prefer
`claude -p '...'` / `codex exec '...'` one-shots.

## The gap list, stated plainly

`--redact` protects against one failure mode: the wrapped process printing the exact
byte sequence of a secret to its own stdout or stderr. It does not protect against:

1. **Any transformation.** base64, hex, URL-encoding, JSON escaping, case changes,
   line breaks or markdown inserted mid-token by the model, or partial disclosure.
   The README's own "matching is exact, so transformed or derived values are not
   redacted" is code-accurate.
2. **Exfiltration through other channels.** The agent's own HTTPS requests, files it
   writes, logs, telemetry, crash reports. dotenvx sees one child's output pipes and
   nothing else.
3. **Grandchildren that bypass the pipe.** A process that opens `/dev/tty` directly
   rather than using the fds dotenvx redirected.
4. **Over-redaction noise.** Short or common values blanking out unrelated text, which
   can silently corrupt the agent's output — including output you are piping into
   another tool. Use `_PLAIN` suffixes on non-secret config to keep this under control.

For a coding agent handling real credentials, the load-bearing controls remain credential
scoping and short TTLs. `--redact` is a useful last layer on top of those.
