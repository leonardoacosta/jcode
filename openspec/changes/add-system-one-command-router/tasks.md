# Tasks

## 1. Catalog and typed decision boundary

- [ ] 1.1 Add versioned command/result DTOs, bounded input/output validation and an explicit native/MCP action catalog using existing workspace conventions; verify unit tests reject unknown IDs/fields/versions, action arrays, invalid targets and executable/tool-name injection.
- [ ] 1.2 Add trusted session/workspace/artifact context handles and catalog snapshots, with interface/read-only/bounded local recipe entries and explicit risk/authorization classes; verify fixtures cover one selected file, multiple targets, unsupported sidebar file type, named-test recipes, escaping paths and non-admitted MCP tools.
- [ ] 1.3 Document catalog admission, semantic-versus-tool identity, fixed-argv constraints and separate Desktop ownership; verify docs do not advertise unsupported source-view surfaces or infer MCP safety from annotations.

## 2. System One classification and pre-chat integration

- [ ] 2.1 Reuse the existing System One resolver to make one deadline-bounded classification call with a closed schema and explicit chat abstention; verify timeout/error/oversize/malformed/tied/low-confidence cases make zero dispatcher calls and trigger no classifier retries.
- [ ] 2.2 Integrate at the shared new-user-message boundary while retaining explicit-command precedence and preserving original chat input; verify submission tests cover supported interfaces, explicit slash commands, multi-step requests, automatic messages and tool-output injection with exactly-once chat fallback.
- [ ] 2.3 Freeze a held-out routing evaluation suite covering paraphrases, quoted commands, questions versus commands, negation, ambiguous targets, unsafe actions and MCP examples; verify zero unauthorized executions in that suite, publish precision/abstention/false-execution metrics, and document the calibrated confidence threshold and measured p50/p95 classification latency against the proposed 300 ms deadline.

## 3. Authorization-preserving dispatch

- [ ] 3.1 Route admitted commands through the existing normal authorization/execution contract rather than assuming debug Direct execution parity; verify tests demonstrate disabled tools, permissions, workspace restrictions and confirmation behavior match normal tool calls.
- [ ] 3.2 Revalidate selection identity/revision, catalog version, availability and exact MCP server/tool binding immediately before dispatch; verify selection changes, tool disappearance, alias collision/deny bypass and workspace mismatch execute nothing.
- [ ] 3.3 Bind named tests and formatter actions to registered bounded direct-argv recipes without generated shell text; verify allowed examples execute once and arbitrary argv/shell fragments, unsupported formatter recipes and escaping paths are refused.
- [ ] 3.4 Document confirmation/fallback rules and actionable errors for unavailable tools and unsupported surfaces; verify a clear destructive/external request never becomes an automatic default and unavailable confirmation goes to chat before dispatch.

## 4. Invocation lifecycle, UI and privacy

- [ ] 4.1 Add atomic session/message-scoped execution claims and retained receipts through existing session state; verify duplicate delivery/reconnect causes one execution, a distinct repeated user message can execute separately, and cancellation uses existing tool semantics.
- [ ] 4.2 Render action/target/running/result/failure receipts in existing supported session/UI surfaces without a chat-model response; verify the motivating selected-file flow is immediate when supported and truthful fallback occurs when the source-panel capability is absent.
- [ ] 4.3 Prevent automatic chat replay after dispatch timeout/error or uncertain side effects; verify public workflow tests show zero duplicate execution and explicit chat continuation receives prior receipt context.
- [ ] 4.4 Add content-free routing telemetry and a feature toggle initially disabled, with operator documentation; verify raw user messages, arguments, credentials, file contents and MCP results are absent from emitted classifier telemetry and disabled mode makes no classifier calls.

## 5. Integration acceptance

- [ ] 5.1 Run focused routing/dispatcher/MCP/cancellation/session tests and the repository's relevant Rust build/test gates against final bytes; verify native interface/read/test/format actions, admitted MCP read actions, ambiguity fallback, permission confirmations, reconnect and failure scenarios through public entrypoints, recording exact commands and observed results.
- [ ] 5.2 Independently review final source and held-out evaluation results for unauthorized execution, scope escalation, hidden Direct-mode bypass and chat-duplicate races; fix findings, rerun affected gates, and run strict non-interactive OpenSpec validation before any implementation completion claim.
