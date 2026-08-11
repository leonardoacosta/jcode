---
name: firecrawl
description: "Use the installed Firecrawl CLI for live web search, known-URL scraping, site mapping/crawling, read-only browser interaction, local document parsing, monitoring plans, indexed research, AI extraction, diagnosis, traceable web deliverables, or durable Firecrawl application integration. Trigger for requests to find, fetch, scrape, crawl, research, monitor, or integrate web content with Firecrawl. Do not trigger for local file operations, Git work, deployment, or code editing that does not need Firecrawl."
---


# Firecrawl CLI

Use the installed CLI for all supported agent-side Firecrawl work. This skill starts after onboarding: `firecrawl` must already be installed and authenticated.

## Preconditions and authentication

Before an action, run `command -v firecrawl`, `firecrawl --version`, and `firecrawl --status`, then inspect `firecrawl --help` and the selected command's installed help. Installed help is the syntax source of truth.

If the executable is absent or authentication is invalid, stop or enter Path D. Never install, upgrade, invoke `npx`, create an account, reproduce raw PKCE, or change transport. Never place API keys in argv (including `--api-key`), command strings, shell history, logs, evidence, or source control. Use the CLI's secure credential store or an inherited environment variable supplied by the approved secret mechanism; never echo it.

## Choose one intent path

| Path | Select when | Required behavior |
| --- | --- | --- |
| **Path A — current-session web data** | The current task needs live web data. | Use the narrowest CLI command. A known URL starts with scrape. Do not add an SDK. |
| **Path B — shipped application integration** | Firecrawl will run in application, service, script, agent, or pipeline code after this session. | Load [references/app-integration.md](references/app-integration.md) only for this path. An official Firecrawl SDK may exist only in shipped code; agent research, validation, and smoke requests remain CLI-based. |
| **Path C — repeatable deliverable** | The result is a brief, report, dataset, or other artifact powered by web evidence. | Collect through the CLI, retain scrubbed source URLs or job identifiers, and synthesize the artifact. |
| **Path D — exceptional recovery** | The installed/authenticated precondition failed, or an installed CLI job failed and remains unresolved after ordinary command help. | Diagnose and remediate only through the CLI, then retry the smallest failed operation. |

An unsupported CLI capability does not enter Path D. Issue a capability gap report containing the CLI version, relevant help inspected, requested capability, and why no supported command applies, then stop unless the exceptional fallback gate below is satisfied.

## Central URL and argv safety gate

Apply this gate before every URL-bearing command, including every URL discovered by search, map, crawl, agent, application input, or redirect:

1. Build a literal argument vector and invoke it through a direct process API: never a shell. Never construct, interpolate, concatenate, or evaluate user input in a shell command. If no no-shell argv interface is available, stop.
2. When installed help proves end-of-options is supported, place `--` before positional user input. Otherwise reject any value with leading `-` or other leading-option confusion.
3. Accept only absolute `https://` URLs. Reject non-HTTPS schemes, malformed URLs, encoded control characters, fragments used as payloads, and URLs containing userinfo or embedded credentials.
4. Canonicalize the host and port. Reject localhost names, metadata-service names, IP literals, and DNS results in loopback, private, link-local, multicast, unspecified, carrier-grade NAT, IPv6 unique-local, documentation, benchmark, or any other reserved range.
5. Resolve and inspect every A and AAAA answer immediately before submission. Reject mixed public/private answers, resolution failure, excessive answer sets, DNS rebinding ambiguity, and any literal or DNS target that is not unambiguously public.
6. Revalidate the scheme, userinfo, hostname, port, and all A/AAAA answers for each redirect before following it. If the CLI cannot expose or constrain every redirect hop, stop with a capability gap rather than fetch.
7. For customer-controlled URLs, enforce a bounded, normalized hostname/port allowlist at both agent and shipped-code boundaries. Wildcards, suffix confusion, and provider-owned redirectors are denied unless explicitly bounded.

Firecrawl provider controls such as lockdown, proxy policy, or filtering are defense in depth, not substitutes for local validation, redirect checks, and customer allowlists.

## Route to the narrowest command

Inspect installed global and selected-command help immediately before execution.

| Intent | Command | Decision rule |
| --- | --- | --- |
| Discover sources without a canonical URL | `firecrawl search` | Search, safety-check returned URLs, then scrape selected results. |
| Extract a known URL | `firecrawl scrape` | Default after the central URL gate. |
| Locate URLs within a known site | `firecrawl map` | Bound host and count; validate every returned URL. |
| Extract multiple pages | `firecrawl crawl` | Bound host, path, page count, bytes, and duration. |
| Perform browser interaction | `firecrawl interact` | Default read-only; use only after scrape is insufficient and the consent gate below passes. |
| Convert a local supported document | `firecrawl parse` | Apply the local-file egress gate below. |
| Plan or inspect recurring checks | `firecrawl monitor` | Prefer a non-mutating plan; inspect subcommand help before state changes. |
| Search supported research indexes | `firecrawl research` | Inspect research subcommand help; validate returned URLs before fetching. |
| Run complex AI-directed extraction | `firecrawl agent` | Use only after deterministic commands are insufficient; retain the same safety bounds. |
| Diagnose preconditions or failed jobs | `firecrawl doctor` | Path D only, including `firecrawl doctor <job-id>` after help inspection. |

### Interact consent gate

`interact` is read-only by default. Before any action, preview the exact ordered actions, target origin, fields, data classes, and externally visible effect. Obtain explicit action-level consent immediately before login, credential entry, form submission, purchase, message, write, deletion, destructive action, or any other externally visible action. Consent for browsing or for one action does not authorize later actions. Never expose credentials in the preview or execute an unpreviewed action; stop when the site changes the proposed action sequence.

### Local parse egress gate

Parsing a local file sends its contents to Firecrawl. Disclose that remote egress before invocation. Classify the file; for confidential, personal, regulated, credential-bearing, or otherwise sensitive content, require explicit approval naming the file/data class before upload. Refuse when approval, minimization, or policy authority is missing. Apply private evidence handling and delete transient copies.

## Transport boundary and unsupported capabilities

Do not configure or use MCP or direct REST by default. Never create an agent-side SDK helper. A CLI invocation error means re-read installed global and relevant command help and correct argv; an invocation error alone never authorizes another transport. Only when the recorded CLI version plus captured global and relevant command help prove the required operation is unsupported may the agent present that evidence and request explicit approval naming one exact MCP server/integration or one exact direct REST endpoint/operation. Without that explicit named approval, stop with the capability gap report. After approval, use only that approved fallback for the proven unsupported operation; do not broaden the surface, substitute the other transport, enter Path D, or bypass any URL/argv, consent, secret, output-bound, or evidence gate in this skill. Path B's SDK allowance is only for shipped application code and does not authorize an agent-side SDK fallback.

## Path D recovery

Path D is limited to failed installed/authenticated preconditions and unresolved CLI-job recovery:

1. Run `firecrawl --status`, inspect `firecrawl doctor --help`, then run `firecrawl doctor`.
2. For a failed run identifier, run `firecrawl doctor <job-id>` with help-verified literal argv.
3. For authentication recovery, inspect current config/login help and use only secure-store or environment-based CLI remediation.
4. Retry only the smallest failed CLI operation after all safety gates pass.

Report credit, concurrency, or rate constraints; change scope or wait only with agreement. Do not use Path D for missing product capability.

## Harness defaults require named opt-in

Ordinary use, packaging, recovery, and evals never run `firecrawl setup defaults`. Only explicit user opt-in for a help-verified named harness permits the exact pair:

```text
firecrawl setup defaults --agent <harness>
firecrawl setup defaults --agent <harness> --undo
```

Use the same approved identifier. A plan-only request remains non-mutating. A live smoke additionally requires consent naming the harness, observable prior state, matching undo, and verified restoration.

## Operational evidence handling

Before capturing output, create a private temporary directory with mode `0700` under a `umask 077`; create evidence files with mode `0600`. Set explicit bounds for URLs/pages, bytes, duration, and retained excerpts. Derive objective assertions from raw output, then retain only scrubbed structured assertions and the minimum excerpts needed for review.

Scrub cookies, `Set-Cookie`, authorization headers, API keys, session identifiers, signed URLs, query tokens, userinfo, form values, PII, and sensitive page content from stdout, stderr, filenames, commands, URLs, and metadata. A regex scan is not sufficient by itself: inspect structured fields and redact by key and data class. Record retention and cleanup actions; delete raw/transient output as soon as assertions are complete. Large output is inspected incrementally and never flooded into context.

Record CLI version, help consulted, literal argv with secrets removed, exit code, bounds, redirect/DNS safety assertions, source URL or job identifier when safe, and cleanup status. Path B reports project tests and CLI smoke separately. Routing fixtures never execute live operations or mutate state.

## Package boundary

The skill and generated plugin contain only authored guidance, the Path B reference, eval assets, and plugin metadata. `firecrawl-kit` does not redistribute the Firecrawl CLI binary, official SDK bytes, `node_modules`, credentials, or external vendor artifacts. Retain generated inventory and package-check evidence proving this boundary.
