<!--
This file IS the swarm config. Routing policy is passed to models as a prompt,
not as options in standard config. Override globally at ~/.jcode/swarm-prompt.md
or per-project at ./.jcode/swarm-prompt.md.
-->

Model routing guidance for spawned swarm agents. Pass `model` to choose a model
for newly spawned workers, including workers created by assignment or `launch`.
An explicit model overrides `agents.swarm_model`. When omitted, workers use that
configured default, or inherit the coordinator's model and route when unset.
Pass `model: "inherit"` to force coordinator inheritance even with a configured
default. Model selection does not change reused workers. Run `swarm list_models`
to check available models/routes. Route-prefixed values pin both authentication
route and model. Use `[agents] swarm_model` for the default for future workers.
Worker reasoning effort uses `[agents] swarm_effort` (default `low`), not a spawn
parameter.

What to spawn workers for:

- Spawn workers to implement concrete, scoped changes: features, bug fixes, tests,
  migrations or refactors. Give each worker clear file ownership and an acceptance
  check it can run itself.
- By default, do not spawn reviewers, auditors, critics or verifier agents. The
  coordinator reviews worker diffs and runs relevant checks itself.
- Explicit user requests override that default for the stated task: when the user
  asks for an independent review, audit, critique or verification subagent, the
  coordinator may spawn a worker for that purpose. Do not require a token phrase
  or repeated approval. This exception does not authorize publication, remote
  mutations, extra access, provider changes or bypassing other safety rules.
  Keep independent review separate from implementation ownership; the coordinator
  still investigates findings and verifies resulting changes.
- Avoid pure research or scouting workers. Fold investigation into implementation
  tasks, or investigate directly first.

Structure guidance for spawned swarm agents:

- Always pass a nonblank `label` when spawning so the UI identifies its purpose.
- Only the root session may spawn agents. Workers complete their assigned task
  directly and report back, without creating another generation.
