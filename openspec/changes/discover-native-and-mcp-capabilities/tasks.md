## 1. Implementation
- [ ] 1.1 Add a read-only registry query returning native and connected MCP tool metadata without starting lazy servers
- [ ] 1.2 Add `/tools [query]` rendering typed rows with ellipsis truncation
- [ ] 1.3 Add the additive remote request and legacy-server notice

## 2. Verification
- [ ] 2.1 Tests: mixed listing, empty query, collision, lazy server not started, remote legacy notice, widths 0/1/20/40/80
- [ ] 2.2 `cargo test -p jcode-app-core mcp_collision` and `cargo test -p jcode-tui`
- [ ] 2.3 `debug_socket` tester frame check, local and remote
- [ ] 2.4 `openspec validate discover-native-and-mcp-capabilities --strict`
