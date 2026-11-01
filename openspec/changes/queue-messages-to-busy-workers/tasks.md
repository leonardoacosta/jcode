## 1. Queue

- [ ] 1.1 Add per-session pending message queue; enqueue instead of bail on busy (`jade_relay.rs:1214`)
- [ ] 1.2 Drain queue at turn end as next input
- [ ] 1.3 Return `queued` ack to sender; enforce bound

## 2. Verify

- [ ] 2.1 Tests: DM to busy worker delivered after turn; bound overflow notice
- [ ] 2.2 `openspec validate queue-messages-to-busy-workers --strict`
