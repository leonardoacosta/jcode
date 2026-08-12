## Why

Jcode now has useful evaluation evidence across the OAuth model-routing smoke run, blind judge receipts, provider-native telemetry, and the Fable → Sol → human disposition → Luna → cold-review microsite remediation workflow. The existing Atlas explains the evaluation system and summarizes one result, but it does not give readers one auditable place to inspect every run, finding, disposition, limitation, and evidence pointer without reconstructing the history from JSON files, OpenSpec artifacts, session records, and review messages.

## What Changes

- Add a brown field-manual Agent Evaluation Evidence Atlas to the existing command-system microsite.
- Add an evaluation index that leads with a decision brief, then exposes a findings ledger, run explorer, review DAG, telemetry, and evidence map.
- Add one versioned `agent-evals.json` manifest covering measured model-routing evidence and the cross-provider microsite review/remediation workflow.
- Give every evaluation, run, candidate, reviewer, finding, disposition, telemetry record, and evidence source a stable identifier and traceable status.
- Preserve provider-native token and timing semantics, judge disagreement, steering evidence, limitations, and the boundary that evaluations never mutate production routing automatically.
- Add deterministic manifest, provenance, digest, link, truthfulness, accessibility, and real-browser acceptance checks.
- Integrate the page into the System Atlas and WS documentation portal, verify the deployed Entra-gated route, and notify the user through ntfy after acceptance.

## Capabilities

### New Capabilities

- `agent-evaluation-atlas`: A static, evidence-backed evaluation ledger and run explorer for Jcode's model-routing experiments and cross-provider review workflows.

### Modified Capabilities

- `command-system-docs`: The System Atlas gains a linked Agent Evaluations destination and source-inventory coverage for the new page and manifest.

## Impact

- Adds `docs/diagrams/jcode-command-system/agent-evaluations.html` and `docs/diagrams/jcode-command-system/agent-evals.json`.
- Updates the existing Atlas index/navigation, `sources.json`, shared styles, static validator, and browser validator.
- Reads frozen evaluation evidence under `evals/model-routing/`, digest-bound OpenSpec artifacts, and sanitized review evidence without changing their authority.
- Updates the WS documentation copy only after Jcode acceptance passes, then uses the existing Azure Static Web Apps pipeline.
- Does not change model routing, provider credentials, evaluation execution, production command behavior, or Recon authority.
- Must not modify unrelated active OpenSpec task ledgers or the untracked brainstorming directories already present in the worktree.
