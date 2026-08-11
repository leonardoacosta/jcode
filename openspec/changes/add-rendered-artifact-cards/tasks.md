## 1. Semantic Artifact Contract

- [ ] 1.1 Add serializable `RenderedArtifactKind` and `RenderedArtifact` shared types with recognized Markdown, Message, and Code kinds, optional title/language fields, and compatibility tests.
- [ ] 1.2 Extend `ToolOutput` with an optional artifact descriptor and ergonomic explicit-artifact builders while leaving generic metadata and untagged defaults unchanged.
- [ ] 1.3 Add an optional serde-defaulted artifact field to persisted tool results and verify legacy session JSON still deserializes unchanged.

## 2. Lifecycle Propagation

- [ ] 2.1 Propagate artifact metadata through every synchronous and streaming agent tool-result construction path without changing model-facing tool-result text.
- [ ] 2.2 Carry artifact metadata through `RenderedMessage`, protocol/history payloads, and `DisplayMessage` conversions for live, reconnect, and replay paths.
- [ ] 2.3 Add round-trip tests proving a fixture tool artifact retains kind, title, language, and body after save/render/restore while untagged output remains on the generic path.

## 3. Distinct TUI Cards

- [ ] 3.1 Add a shared artifact-card renderer using existing rounded-box, width, centered-mode, and ASCII-capability primitives.
- [ ] 3.2 Implement the document-blue Markdown card and warm-neutral Message card using the existing Markdown renderer and semantic selection maps.
- [ ] 3.3 Implement the terminal-green Code card using the existing code highlighting path, optional language title, and exact-source `CodeBlock` copy target.
- [ ] 3.4 Dispatch recognized artifact metadata before generic tool rendering and preserve safe generic fallback for absent or unsupported metadata.

## 4. Verification and Delivery

- [ ] 4.1 Add exact rendering tests for all three card identities, formatted content, narrow widths, ASCII mode, and distinction from reasoning and generic tool actions.
- [ ] 4.2 Add copy tests proving card chrome is excluded and code body/language are preserved exactly.
- [ ] 4.3 Run focused shared-type, agent/session, protocol, Markdown, and TUI test suites plus strict OpenSpec validation.
- [ ] 4.4 Build and run the changed client on an isolated socket, capture a transcript containing all three explicit artifacts alongside reasoning and a generic action, and verify the visual hierarchy and restore behavior.
- [ ] 4.5 Commit the scoped implementation, verify the automatic deploy/reload hook installs that exact commit, archive the OpenSpec change, and update the roadmap handoff if this work is tracked there.
