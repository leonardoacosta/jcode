## 1. Queue

- [x] 1.1 Add per-session pending message queue; enqueue instead of bail on busy (`jade_relay.rs:1214`)
- [x] 1.2 Drain queue at turn end as next input
- [x] 1.3 Return `queued` ack to sender; enforce bound

## 2. Verify

- [x] 2.1 Tests: DM to busy worker delivered after turn; bound overflow notice (unit FIFO+overflow; turn-end drain: `pending_messages_drain_as_one_turn_after_busy_turn_ends`)
- [x] 2.2 `openspec validate queue-messages-to-busy-workers --strict`
