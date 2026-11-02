## 1. Pane identity

- [ ] 1.1 Capture pane ID from the Herdr launch and store on the member (serde default)
- [ ] 1.2 Worker panes report their own resume argv

## 2. Focus

- [ ] 2.1 `/agents` focus-pane action; hidden without a pane ID

## 3. Verify

- [ ] 3.1 Tests for pane ID capture and persistence
- [ ] 3.2 Runtime check: spawn visible worker in Herdr, focus it, restore after Herdr restart
- [ ] 3.3 `openspec validate focus-worker-herdr-pane --strict`
