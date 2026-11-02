## 1. Implementation

- [ ] 1.1 Add item state and acknowledgement to `ScheduleQueue`
- [ ] 1.2 Startup recovery marks dispatched-without-ack items uncertain and overdue items missed; require explicit manual retry
- [ ] 1.3 Bounded history in `schedule list`
- [ ] 1.4 Tests: crash-before-ack stays uncertain without redelivery; lost ack after execution does not duplicate work; explicit retry is recorded; acknowledged item not redelivered; overdue shown as missed

## 2. Verification

- [ ] 2.1 `cargo test -p jcode-app-core` passes for the new tests
- [ ] 2.2 `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] 2.3 `openspec validate recover-missed-scheduled-runs --strict`
