# Recovery

| Failure | Resume action |
| --- | --- |
| Dirty working tree | Finish or remove unrelated work outside this workflow, then restart preflight |
| Source differs from remote | Update the source branch intentionally, then restart from the new SHA |
| Nothing to merge | Stop successfully without creating an empty merge |
| Existing deployment is definitively unhealthy | Fix it, or use the explicit deploy-health override only when verified unrelated |
| Smoke or quality gate fails | Fix on the source branch and rerun gates and review for the new SHA |
| Confirmed review blocker | Fix on the source branch and rerun gates and review for the new SHA |
| Review is inconclusive | Retain the blocker or perform another native review; do not treat uncertainty as clearance |
| Source advanced after review | Discard review state and review the new source SHA |
| Large-batch confirmation is declined | Stop with source and target unchanged |
| Merge lock is unavailable | Wait for the other merge to settle, then restart the final revalidation |
| Target update is not a fast-forward | Stop and reconcile target drift before merging |
| Merge conflict | Abort on target, restore the starting branch, resolve on source, then restart |
| Push fails or remote proof is missing | Do not report completion; inspect remote state and retry only when the result is known |

On every failure path, release any acquired repository lock and restore the starting branch when it
still exists. Cached phase evidence becomes stale when the repository identity, source SHA, target
SHA, gate definition, or relevant working tree changes.
