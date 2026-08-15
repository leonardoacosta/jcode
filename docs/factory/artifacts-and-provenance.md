# Artifacts and provenance

> Status: proposed contract

Artifacts are durable outputs, not merely transcript fragments. The factory should preserve:

- intent and specification;
- plan and task graph;
- workspace identity and source revision;
- worker trace and tool calls;
- patches, generated files, and screenshots;
- test, build, security, and evaluation results;
- approvals, dispositions, merge or deployment receipts;
- limitations and unresolved questions.

Each material artifact should have a stable identifier, producer, input references, revision or digest, status, and links to evidence. Session state may be ephemeral; artifacts must remain inspectable and replayable.
