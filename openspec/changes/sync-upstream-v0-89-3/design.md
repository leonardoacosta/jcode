# Synchronize upstream v0.89.3

## Baseline and target

Observed clean main: cb9fb8ac7b5ff6ac87f815374bb8ff9144ab4527, tracking origin/main at the user's fork. Upstream release commit: de65ade33d514b31a43885318179b3622f321170. Common ancestor: 7a3479cd7040a2197e66fff6e20d2a4739e6a77e. Observed divergence: 130 local-only and 188 upstream-only commits. Recheck all facts before execution, since other sessions can change the repository.

## Approach

Merge the pinned release into existing main. Rebase would rewrite local history. Reconstructing the fork from upstream would risk dropping custom work. A merge retains both ancestries and matches the earlier no-additional-integration-branch policy.

Create a uniquely named recovery ref at the actual execution baseline. Record baseline and target before merging. Configure `upstream` for https://github.com/1jehuang/jcode.git and fetch upstream master and the release without replacing existing local tags. If an existing upstream remote points elsewhere, stop rather than overwrite it. Keep main tracking origin/main. Future updates fetch upstream and explicitly merge the selected verified release, rather than pulling upstream into main implicitly.

## Conflict and error handling

Inventory upstream and local changes before resolving conflicts. Review conflict resolutions per capability, especially provider credentials and routing, MCP transport, agents dispatch, test isolation, prompt and workflow customizations, and build/update configuration. Do not choose one whole side blindly. Record any behavior that cannot coexist and seek a decision rather than silently deleting it.

Do not start a merge with unowned working changes or another active Git operation. Coordinate active agents before touching shared files. If integration fails, retain the recovery ref and diagnostics. Abort only this merge when safe. Never hard-reset unrelated work.

## Verification and delivery boundary

Verify both the recorded local baseline and release commit are ancestors of the resulting merge. Inspect the complete merge diff and retained local-only capability changes. Run formatting checks, affected regression tests, and the repository's relevant full test gate using coordinated selfdev tooling. Build the TUI with selfdev and exercise the newly built binary on a dedicated socket or tester so the old shared daemon cannot invalidate evidence. Include MCP, provider routing, agents dispatch, and source update checks in regression coverage where supported by real tests.

Commit only owned integration changes after validation, using configured Git identity. If Git merge staging would include unrelated active-agent changes, stop and isolate those changes first. Do not push or switch the shared daemon as part of source synchronization. Report commit ID, source version, validation results, remaining limitations, and a repeatable future-update command sequence. Deployment is a separate approval boundary.
