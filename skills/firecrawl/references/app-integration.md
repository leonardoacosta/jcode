
# Path B: Firecrawl in shipped application code

Read this reference only when Firecrawl will run after the current session. Agent research, validation, and live smoke requests still use the installed CLI and every central safety gate in `SKILL.md`.

## Inspect before editing

Identify the repository's language, package manager, lockfile, dependency policy, environment/secret conventions, third-party service boundary, URL authorization model, test runner, and required quality gates. Define the narrowest durable capability before adding a dependency.

## Official SDK and supply-chain gate

An official Firecrawl SDK may be added only to shipped code and only when it supports the target language. Before editing:

1. Verify the current package name, publisher, source repository, and official status from authoritative metadata.
2. Select and record an exact version; never use a range, floating tag, unpinned Git reference, or install script fetched ad hoc.
3. Commit the target repository's exact lockfile and verify the resolved package integrity.
4. Run the ecosystem's package audit and the repository's dependency/security checks.
5. Review direct and transitive dependency licenses, security advisories, install scripts, native binaries, provenance, and unexpected dependency growth. Stop on an unresolved license, integrity, or security finding.
6. Retain package/version, lockfile diff, audit output, transitive license/security review, and approval evidence.

Follow existing dependency, service-layer, error-handling, and secret-injection patterns. Keep SDK construction injectable so deterministic project tests use a fake without credentials or network.

The canonical skill and generated `firecrawl-kit` do not redistribute the Firecrawl CLI binary, SDK packages, `node_modules`, credentials, or any external CLI/SDK bytes. They contain only guidance, this reference, eval assets, and plugin metadata. Retain exact package inventory evidence.

If no official SDK supports the language, stop and report the gap. Never substitute agent-side SDK code, MCP, direct REST, or raw HTTP.

## URL authorization and SSRF boundary

Treat every application URL as untrusted. Reuse or add a centralized validator that implements the skill's HTTPS, userinfo, argv-equivalent input, literal/DNS address, and per-redirect checks. Resolve every A and AAAA answer and reject private, local, link-local, metadata, reserved, mixed, or ambiguous results.

For customer-controlled fetching, require a bounded normalized hostname/port allowlist tied to the authorized customer or tenant. Revalidate authorization and DNS on every redirect and at execution time. Wildcards, suffix-only comparisons, provider redirectors, and Firecrawl provider controls are not substitutes. If the SDK cannot expose or constrain redirect hops sufficiently, stop and report the capability gap.

## Secrets and sensitive data

Supply `FIRECRAWL_API_KEY` only through the repository's approved environment/secret manager. Never pass it in argv, URLs, logs, test snapshots, telemetry, or committed configuration. Commit only an example variable name when repository convention requires it.

Minimize submitted content and scrub operational evidence as defined in `SKILL.md`. For sensitive customer content, require the product's authorization and data-processing policy to permit Firecrawl egress.

## Integration sequence

1. Inspect repository, authorization, dependency, and test surfaces.
2. Define the durable capability, customer allowlist, output bounds, retention, and narrowest operation.
3. Validate representative inputs through installed CLI help without changing transport.
4. Complete the official SDK supply-chain gate and add the exact pinned dependency plus lockfile.
5. Implement at the existing service boundary with injected construction, centralized URL validation, redirect revalidation, bounded output, and typed/scrubbed errors.
6. Run deterministic project tests without a real key or network, including unsafe URL/DNS/redirect and authorization cases.
7. Run normal lint, type, package-audit, license/security, and project gates.
8. With runtime consent, run one separate small installed-CLI credential/capability smoke through the CLI.
9. Report project tests, supply-chain evidence, and CLI smoke as separate evidence streams.

## Evidence contract

### Project and supply-chain evidence

Record exact SDK version, authoritative package metadata, lockfile integrity, dependency diff, audit result, transitive license/security review, test command, exit code, URL-safety assertions, and confirmation that no real API key or network was used by mocked tests.

### CLI smoke evidence

Record CLI version, help consulted, secret-free literal argv, exit code, HTTPS/DNS/redirect safety assertions, bounded target/result assertions, private evidence modes, scrub/retention/cleanup status, and non-empty output. Run through the installed CLI, never application code or an SDK helper.

Project tests, dependency review, and CLI smoke are independently required; none substitutes for another.
