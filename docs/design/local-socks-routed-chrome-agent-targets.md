# Local SOCKS-Routed Chrome Agent Targets

Status: design draft for user review
Authority: Jcode initiative `local-socks-routed-chrome-agent-targets`
Scope: homelab local filesystem and Jcode browser-provider integration only

## Goal

Provide two explicitly selectable Chrome targets, `chrome_bbadmin` and `chrome_o365`, usable both from the operator's terminal and by Jcode skills. Each target must use an isolated persistent Chrome user-data directory and must route traffic through a required homelab SOCKS endpoint, potentially supplied by an SSH dynamic forward.

No remote browser fleet, Mac, or external profile migration is in scope.

## Recommended architecture

```text
human CLI alias ─┐
                 ├─ shared local Chrome launcher/policy ── Chrome process
Jcode provider ──┘             │                         ├─ isolated profile
                               │                         ├─ localhost CDP
                               │                         └─ required SOCKS proxy
```

The shared launcher is the single owner of profile paths, proxy requirements, CDP binding, readiness checks, process groups, and cleanup. The human aliases and Jcode provider adapter must call this same launcher rather than reimplementing launch flags.

Jcode exposes only the two allowlisted target names. It does not accept arbitrary profile paths, arbitrary Chrome flags, arbitrary CDP endpoints, or a generic proxy supplied by a skill.

## Persistent profile layout

Default profile roots:

- `~/.local/share/jcode/chrome-bbadmin/`
- `~/.local/share/jcode/chrome-o365/`

The profile directories are created on first launch. Login is manual and occurs on the homelab. No cookies, tokens, or profile databases are copied from another machine.

## Proxy contract

The launcher requires a configured SOCKS endpoint before starting Chrome. It may be:

- an already-running local SOCKS listener, or
- an SSH dynamic forward started and supervised by the launcher or its companion service.

The initial page or URL does not determine routing. Once Chrome starts with the target policy, every tab and later navigation in that Chrome process inherits the explicit SOCKS proxy, isolated profile, and loopback CDP settings. A page's appearance does not show whether SOCKS is active, so verification must inspect the owned process launch policy and the SOCKS listener/readiness path.

DNS behavior must be selected deliberately. The default should use SOCKS5 hostname resolution where supported so LAN hostnames do not resolve outside the tunnel. This requires a verification probe before implementation is considered complete.

## Chrome and CDP contract

- Each running target binds CDP to `127.0.0.1` only.
- CDP ports are allocated safely and are not assumed to be globally fixed.
- The launcher records ownership and readiness metadata in a Jcode-owned runtime directory.
- Stale metadata or stale processes are detected without terminating unrelated Chrome instances.
- A target cannot attach to the operator's default Chrome profile.
- The launcher uses a dedicated Chrome user-data directory per target, even if Chrome's internal profile name is `Default`.

## Jcode provider contract

Add explicit local provider targets equivalent to:

- `chrome_bbadmin`
- `chrome_o365`

The provider adapter should:

1. Validate the target against the allowlist.
2. Ask the shared launcher to start or reuse the target.
3. Verify the expected profile identity, localhost CDP endpoint, and proxy readiness.
4. Attach through CDP and expose normalized browser operations.
5. Preserve browser affinity for the Jcode session.
6. Release or terminate only processes owned by the target launcher.

The current generic isolated Chrome provider remains unchanged. Skills must opt into one of these targets explicitly.

## Skill contract

Skills that need authenticated LAN browser access may request a named target through the normalized browser interface. They must not know profile paths, CDP ports, Chrome flags, SSH commands, or SOCKS credentials. Skills default to the ordinary isolated browser unless they explicitly request `chrome_bbadmin` or `chrome_o365`.

Documentation should define when each alias is appropriate and require explicit user intent for account-affecting actions. Existing hard-deny and confirmation rules remain in force.

Manual O365 sign-in readiness and visible-state checks are recorded in [O365 Local Chrome Manual Sign-In Readiness](./o365-local-chrome-manual-sign-in-readiness.md).

## Azure CLI and REST authorization boundary

The named browser targets are browser-only account contexts. A logged-in Azure Portal tab in `chrome_bbadmin` or `chrome_o365` must never be treated as authorization for Azure CLI or direct Azure REST calls.

Allowed local workflow:

1. Use the named browser target only for visible portal inspection, manual sign-in, and human-visible account context checks.
2. Use `az login`, `az account set`, `az account get-access-token`, or an equivalent Azure SDK credential chain for CLI and REST access.
3. Keep Azure CLI token-cache files in the normal local Azure CLI cache, outside the repository and outside Chrome profile directories.
4. For REST calls, obtain tokens from Azure CLI or an Azure SDK credential provider in the same local process that performs the request.
5. Treat CLI/REST mutations as account-affecting actions. Require explicit user confirmation immediately before deleting resources, changing permissions, changing tenant or subscription configuration, sending mail, or modifying production services.

Forbidden workflow:

- Do not read Chrome cookies, local storage, IndexedDB, profile databases, browser cache files, extension storage, CDP network headers, HAR authorization headers, bearer tokens, refresh tokens, passwords, or hidden session state.
- Do not copy browser-derived credentials into `az`, `curl`, SDK configuration, environment variables, repo files, logs, telemetry, generated artifacts, prompts, or commits.
- Do not claim that portal sign-in proves Azure CLI sign-in. Verify CLI identity with safe local commands such as `az account show` before read-only CLI diagnostics.
- Do not bridge `chrome_bbadmin` and `chrome_o365` identities. Each browser target remains isolated, and CLI account selection must be explicit and independently verified.

REST integrations must expose only resource IDs, tenant IDs, subscription IDs, scopes, request methods, and response metadata needed for diagnosis. They must redact authorization headers and token-shaped values before logging. If no Azure CLI or SDK credential is available, the safe result is an authentication-required error, not a browser-token extraction attempt.

## Human CLI contract

Provide two thin aliases:

```text
chrome_bbadmin [approved Chrome arguments]
chrome_o365 [approved Chrome arguments]
```

They call the shared launcher, preserve the same proxy/profile policy, and expose a useful status/error message. Arbitrary security-sensitive overrides such as `--user-data-dir`, `--proxy-server`, remote CDP binding, or disabling proxy checks are rejected.

## Lifecycle

Use on-demand supervised processes:

- Profiles persist.
- Chrome processes are started when requested.
- Jcode may reuse a live process it owns for the same target.
- Closing the final owning session releases or terminates the process according to the selected lifecycle policy.
- A cleanup command removes only launcher-owned processes and metadata.

The first implementation should prefer termination on final release to minimize exposure of authenticated sessions. A later explicit keep-open mode may be added if needed.

## Security and failure behavior

- Fail closed when SOCKS readiness cannot be proven.
- Never bind CDP to a non-loopback address.
- Never accept arbitrary paths or arbitrary provider names from skills.
- Keep runtime metadata and any local secrets mode `0600`.
- Do not log cookies, authorization headers, profile contents, SSH credentials, or full command lines containing secrets.
- Distinguish unavailable proxy, profile lock, stale process, exited owned process, CDP failure, authentication-required state, and application-context failure.
- A visible authenticated page or `Sign out` control proves only that the browser profile has a visible session. It does not prove Azure CLI or REST authorization.
- An application can show `Sign out` and still fail during initialization, such as Teams reporting `CREATE_USER_CONTEXT_FAILED_GENERIC`.
- Never kill unrelated Chrome processes.
- Treat profile directories as sensitive local data and exclude them from repository paths and backups.

## Requirements and acceptance scenarios

### R1. Isolated profiles

Given either alias, when Chrome starts, it uses only that alias's persistent user-data directory. Starting the other alias does not expose or reuse its cookies, local storage, extensions, or locks.

### R2. Enforced SOCKS routing

Given the SOCKS endpoint is unavailable, launching either alias fails before Chrome starts. Given it is available, the owned Chrome process includes the explicit SOCKS proxy flag and an observable external-IP and LAN-host probe confirms traffic uses the expected route and DNS behavior. The initial URL is not part of the routing guarantee, and page content alone is not sufficient evidence of proxy use.

### R3. Human aliases

Given a terminal user invokes either alias, the shared launcher starts the correct profile with the same routing and CDP policy. Forbidden overrides are rejected.

### R4. Jcode targets

Given a Jcode session selects either named target, Jcode can attach to the correct running Chrome through localhost CDP and use normalized browser operations. Selecting an unknown target fails validation.

### R5. Lifecycle safety

Given a session closes, only launcher-owned Chrome processes may be released. Stale metadata and unrelated browser processes remain untouched.

### R6. Skill integration

Given a skill explicitly requests a named target, it can use the target without knowing profile paths or ports. Without explicit selection, it uses the ordinary isolated browser.

### R7. Authenticated data safety

Profile data never enters the repository, logs, telemetry, generated artifacts, or commits.

### R8. Azure authorization separation

Given a named browser target is authenticated to Azure Portal, when a workflow needs Azure CLI or REST access, it authenticates through Azure CLI or Azure SDK credentials instead of browser state. Browser cookies, browser storage, CDP network credentials, and browser-derived tokens are never read or exported.

### R9. Visible authenticated session and application readiness

Given a human signs into `chrome_o365`, a visible `Sign out` control or authenticated Microsoft 365 shell proves the profile has a visible authenticated session. If a Microsoft app then reports an initialization error, the session state and application readiness must be reported separately. If the owned process exits, the target must be relaunched through the named launcher rather than attaching to an unrelated browser.
- No remote browser or Mac changes.
- No automatic migration of existing Chrome profiles.
- No automated login or credential harvesting.
- No Azure CLI or REST authorization from browser cookies, storage, CDP traffic, bearer tokens, refresh tokens, or profile files.
- No generic arbitrary-profile browser API.
- No proxy bypass or direct-network fallback.
- No exposure of CDP beyond localhost.
- No skill-specific implementation of SSH, SOCKS, or Chrome launch logic.

## Implementation tasks

1. Discover the repository's current browser-provider interfaces and local runtime conventions.
2. Design the shared launcher configuration and runtime metadata schema.
3. Implement profile allowlisting, proxy readiness, safe Chrome launch, localhost CDP, and ownership tracking.
4. Add human aliases that call the shared launcher.
5. Add first-class Jcode target registration and CDP attachment.
6. Add normalized error mapping and session lifecycle handling.
7. Add skill documentation and target-selection guidance.
8. Add Azure CLI/REST integration documentation and guardrails that use Azure CLI or SDK credentials only, never browser credentials.
9. Add unit tests for allowlists, flags, proxy fail-closed behavior, port binding, stale state, ownership, and Azure authorization separation.
10. Add local integration tests with a test SOCKS endpoint and disposable profiles.
11. Run a live homelab smoke test with manual login, proxy route verification, LAN-host access, CDP attachment, CLI identity separation, and cleanup.
12. Verify post-login visible session state separately from Microsoft application initialization errors and owned-process lifecycle failures.

## Verification gates

- Static/config tests prove the two aliases are the only exposed authenticated targets.
- Unit tests prove forbidden overrides, proxy absence, non-loopback CDP, and unrelated-process preservation.
- Unit or documentation lint tests prove Azure CLI/REST workflows use Azure CLI or SDK credential sources and never browser state.
- Integration tests prove both aliases can be launched with disposable profiles and a controlled SOCKS endpoint.
- Live smoke test proves external-IP routing, LAN reachability, DNS behavior, correct profile isolation, Jcode attachment, and cleanup.
- Repository checks prove profile directories, cookies, runtime metadata, and secrets are not tracked or emitted.

## Decisions and defaults

- Use first-class provider integration as the target architecture.
- Share one launcher between Jcode and human CLI aliases.
- Support both human terminal use and agent use.
- Use on-demand supervised Chrome processes.
- Keep profile data persistent but process lifetime bounded.
- Use explicit allowlisted names rather than arbitrary profile paths.
- Fail closed when SOCKS readiness is unavailable.
- Bind CDP to loopback only.

## Open gates before implementation

1. Confirm the exact local SOCKS/SSH command and endpoint contract.
2. Confirm the desired DNS behavior and the LAN-host verification target.
3. Confirm whether first release should terminate Chrome immediately on final agent release or allow a short idle grace period.
4. Confirm the repository's canonical implementation surface after provider-interface discovery.
