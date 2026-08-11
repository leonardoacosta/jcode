# Portability Map

Keep workflow decisions in the portable skill and mechanics in each harness binding.

| Aspect | Portable semantic | Harness-owned primary |
| --- | --- | --- |
| Invocation | Merge a delivery branch directly into the target with local review | Command name, skill trigger, argument parsing, UI metadata |
| Repository inspection | Resolve root, branches, remote, clean state, immutable SHAs, and mergeability | Native shell/Git tool |
| Release summary | Describe `target..source` commits, files, churn, and feature/spec identifiers | Native shell plus model synthesis |
| Existing health | One read of last-known deployment state and an existing smoke command | Repository/plugin detector when already available; otherwise skip |
| Quality gates | Run the repository's canonical local validation contract | Existing gate runner or documented commands |
| Correctness review | Cover the pinned diff and scale coverage to churn | Native reviewer/subagent primitive or an inline review |
| Architecture review | Inspect the complete pinned diff independently | Native specialist/subagent primitive or a separate inline pass |
| Blocker verification | Focused independent re-check of each proposed blocker | Native reviewer/subagent primitive; sequential is valid |
| Confirmation | Auto-proceed at 10 commits or fewer; explicitly confirm larger batches | Native question/approval UI |
| Serialization | Prevent concurrent mutation of the target when repository support exists | Existing repository lock/lease helper; otherwise synchronous revalidation |
| Merge | Revalidate the live source, merge the pinned SHA with `--no-ff`, push, verify | Native shell/Git tool |
| State and resume | Reuse only evidence bound to unchanged repository and SHAs | Harness-native checkpointing when available; otherwise safe rerun |
| Telemetry and notifications | Optional observations that do not affect correctness | Harness hooks, event APIs, TTS, or status UI |
| Output | Report gates, reviews, merge proof, and excluded production-push steps | Harness-native final response formatting |

## Binding rules

- Load this skill as the source of workflow semantics.
- Name the native capabilities used by the binding, but do not copy the workflow body into the
  binding.
- Keep telemetry, TTS, command frontmatter, model selection, and UI presentation outside the
  portable contract.
- Do not require concurrency. Parallel native reviews are an optimization; equivalent independent
  sequential reviews preserve semantics.
- Do not add a process wrapper, custom sandbox, runtime manifest, fingerprint protocol, report
  schema, or agent-result file format merely to make harnesses look alike.
- Test observable invariants: exact reviewed SHA, gates-before-merge, blocker behavior, large-batch
  confirmation, no force push, push verification, branch restoration, and merge-only scope.
