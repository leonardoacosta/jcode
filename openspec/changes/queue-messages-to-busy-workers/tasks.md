## 1. Queue

- [x] 1.1 Add per-session pending message queue; enqueue instead of bail on busy (`jade_relay.rs:1214`)
- [x] 1.2 Drain queue at turn end as next input
- [x] 1.3 Return `queued` ack to sender; enforce bound

## 2. Verify

- [ ] 2.1 Tests: DM to busy worker delivered after turn; bound overflow notice (unit: FIFO + bound overflow pass; turn-end drain not covered end-to-end)
- [x] 2.2 `openspec validate queue-messages-to-busy-workers --strict`
