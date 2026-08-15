# Isolation and execution

> Status: proposed target with partial local support

Workers should run in an explicitly owned environment: a Git worktree, container, micro-VM, or remote backend. The environment contract should define filesystem scope, network access, credentials, tool permissions, resource limits, source revision, and cleanup policy.

Isolation enables parallelism, safe retries, reproducibility, and rollback. Pi's current documentation is a useful reminder that an agent shell may require external sandboxing. OpenHands demonstrates a platform model where local, Docker, VM, and cloud backends can be selected without changing the operator surface.
