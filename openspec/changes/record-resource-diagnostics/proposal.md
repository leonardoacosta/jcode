---
execution:
  version: 1
  depends_on: []
---
# Proposal

Roadmap owner: [P10: Distribution/platform support/performance/diagnostics](../../../docs/plans/PRODUCT_ROADMAP.md#p10-distributionplatform-supportperformancediagnostics)

## Why

`src/bin/session_memory_bench.rs` and `scripts/analyze_runtime_memory_log.py` exist, but neither records the serving binary version, hardware or workload with results, so numbers cannot be reproduced on the actual serving build as the P10 gate requires.

## What Changes

- Bench output includes binary path and version, git revision, CPU, memory and workload parameters.
- Output is JSON with home paths and secrets redacted.

## Non-scope

New benchmarks, performance targets, telemetry upload.

## Capabilities

### New Capabilities

- `resource-diagnostics`: record binary identity, hardware and workload with every resource measurement.

## Impact

`session_memory_bench.rs`, a metadata helper using `jcode-build-meta`.

## Resolved defaults

1. Redact the home path or keep it? Options: keep (exact paths); replace with `~`. Selected default: replace with `~`; keeps reproducibility without leaking usernames.
