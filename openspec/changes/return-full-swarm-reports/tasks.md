## 1. Reports

- [ ] 1.1 Store full report body per member; return it from `summary` before tool calls
- [ ] 1.2 Remove 60-char line truncation in report rendering (`communicate.rs:2505`)
- [ ] 1.3 `await_members`: include report bodies; mark ready-without-report `incomplete`
- [ ] 1.4 Count reported-then-stopped members as done; suppress late timeout wake-ups

## 2. Verify

- [ ] 2.1 Tests: summary returns report; ready w/o report incomplete; reported+stopped no timeout
- [ ] 2.2 `openspec validate return-full-swarm-reports --strict`
