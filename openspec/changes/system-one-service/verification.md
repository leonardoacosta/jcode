# Verification record

Status: proposal awaiting approval and implementation.

## Requirement-to-acceptance mapping

| Requirement | Observable acceptance | Tasks |
| --- | --- | --- |
| Modular service crate | Crate compiles, dependency tree excludes app-core/TUI/protocol | 1.1, 1.2 |
| Provider resolution | Both provider routes, credential precedence, empty/missing keys | 3.1, 3.2 |
| Typed response validation | Valid and missing answers, cache parity | 2.1, 2.2, 4.2, 4.3 |
| Evaluate migration preserves behavior | Cache hit/miss parity, existing tests pass, schema unchanged | 5.1–5.5, 6.2 |
