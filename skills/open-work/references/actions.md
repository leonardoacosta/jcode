# Action Policy

The fixed class order is `archive -> disposition -> dispatch -> apply`. Skip an empty, declined, or
unsupported class without reordering later independently confirmed classes.

## Archive

Archive confirmed complete proposals sequentially with `openspec archive <id> --yes`. Stop this class
on the first failure and report completed, failed, and remaining proposal IDs.

## Disposition

Write confirmed human-only dispositions sequentially. Remove stale `hitl:*` labels, leave exactly one
current label, and add one matching bounded comment:

- answered: `hitl:answered` and `HITL answered: <answer>`
- parked: `hitl:parked` and `HITL parked: <reason>`
- converted for agent work: `hitl:afk-converted` and `HITL afk-converted: <instruction>`

An answer or conversion requires actual text from the operator. Stop the class on the first unsafe
partial write and report the durable state rather than guessing recovery.

## Dispatch

Dispatch only confirmed agent-ready Beads. Run same-repository items sequentially, allow independent
repositories in parallel, and use at most five workers total. Cross-repository items require a
separate explicit confirmation before any mutation in that repository. Follow repository claim,
test, closure, and persistence rules; inventory authority alone grants none of these operations.

## Apply

Invoke one consolidated shared apply workflow with the confirmed proposal set. Do not fan proposals
out into independent apply runs and do not reproduce apply logic inside this skill. The shared apply
workflow owns dependency planning, conflicts, gates, and persistence.

## Failure accounting

For every selected class, report completed, failed, skipped, and remaining IDs. A class failure stops
that class at the first unsafe continuation. Later classes may proceed only when they were separately
confirmed and are independent of the failure. Never convert a partial result into claimed success.
