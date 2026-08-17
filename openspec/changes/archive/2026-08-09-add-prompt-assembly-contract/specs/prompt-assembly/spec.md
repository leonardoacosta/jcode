## ADDED Requirements

### Requirement: Versioned layered assembly

Jcode SHALL assemble the system prompt from named layers with a fixed documented order, where each layer carries an id, a source (builtin, project file, global file, or runtime), a mode (`replace` or `append`), content, and a char count, under a `PROMPT_ASSEMBLY_VERSION` constant.

#### Scenario: Byte-identical assembly for unchanged inputs

- **WHEN** the prompt is assembled with the same inputs as the pre-change implementation
- **THEN** the joined static and dynamic text SHALL be byte-identical to the pre-change output.

#### Scenario: Replace applies to the base layer only

- **WHEN** a base replacement file or runtime override is present
- **THEN** only the base layer SHALL be substituted
- **AND** append layers (overlays, preferred tools, AGENTS.md, skills list) SHALL still apply in their fixed order
- **AND** a runtime `system_prompt_override` SHALL be modeled as a full-assembly replace producing its own digest and attribution.

#### Scenario: Version bump on contract change

- **WHEN** the layer set, order, or mode semantics change
- **THEN** `PROMPT_ASSEMBLY_VERSION` SHALL be incremented
- **AND** assemblies at different versions SHALL produce different digests even for identical text.

### Requirement: Prompt digest

Jcode SHALL compute a stable prompt digest over the contract version, static layer ids, source kinds, and static layer contents, and SHALL record it in the session record at capture time.

#### Scenario: Reproducible digest

- **WHEN** two assemblies run with identical inputs in separate processes
- **THEN** the digests SHALL be identical
- **AND** source paths differing while content is identical SHALL NOT change the digest.

#### Scenario: Digest sensitivity

- **WHEN** any static layer content, the layer order, or the contract version changes
- **THEN** the digest SHALL change.

#### Scenario: Digest visibility

- **WHEN** an operator inspects session status, `/context`, or session debug state
- **THEN** the prompt digest SHALL be visible.

### Requirement: Source attribution

Jcode SHALL attribute every static prompt layer with its origin and char count and SHALL surface the attribution in `/context` and session debug state.

#### Scenario: Layer origins listed

- **WHEN** attribution is inspected
- **THEN** each static layer SHALL list its origin as builtin, project file (with path), global file (with path), or runtime (with kind)
- **AND** its char count SHALL match the layer content.

### Requirement: Session-frozen static layers

Static prompt layers SHALL be captured once per session and reused for every turn of that session; dynamic layers (memory, active skill, turn reminder, swarm effort directive) SHALL remain per-turn.

#### Scenario: Mid-session file edit does not leak

- **WHEN** an overlay, AGENTS.md, or preferred-tools file changes after session capture
- **THEN** later turns of that session SHALL produce the same prompt and digest as before the edit
- **AND** a newly started session SHALL observe the edit.

#### Scenario: Resume captures fresh

- **WHEN** a session is resumed or reloaded
- **THEN** a fresh prompt snapshot SHALL be captured at first build after resume
- **AND** the recorded digest SHALL reflect the resumed capture.

#### Scenario: Dynamic layers stay dynamic

- **WHEN** memory content, the active skill, a turn reminder, or the swarm effort directive changes between turns
- **THEN** the change SHALL appear in that turn's dynamic prompt part
- **AND** the frozen static text and digest SHALL be unaffected.

### Requirement: Golden snapshot gate

The assembly SHALL have golden snapshot coverage for base, append, replace, project overlay, and runtime augmentation scenarios.

#### Scenario: Base golden

- **WHEN** only builtin sources are present
- **THEN** the assembly SHALL match the recorded base golden exactly.

#### Scenario: Append goldens

- **WHEN** global and/or project overlay files are present
- **THEN** the assembly SHALL match the recorded append goldens, including project-over-global precedence.

#### Scenario: Replace goldens

- **WHEN** a base replacement file is present
- **THEN** the assembly SHALL match the recorded replace golden
- **AND** an empty or whitespace-only replacement file SHALL fall back to the builtin base.

#### Scenario: Augmentation golden

- **WHEN** memory injection, a turn reminder, and a swarm effort directive are applied
- **THEN** the dynamic part SHALL match the recorded augmentation golden with each contribution in its documented position.

### Requirement: Pi compatibility mapping

Jcode SHALL document the mapping from Pi's prompt concepts (base, replacement, appended prompts, hook mutation) to the assembly contract without copying Pi files.

#### Scenario: Append and replace parity documented

- **WHEN** the contract documentation is read
- **THEN** Pi base prompt SHALL map to the base layer, Pi replacement prompt to base-layer replace mode, and Pi appended prompt to the append layers
- **AND** Pi hook-based prompt mutation SHALL be documented as unsupported (observer-only hooks).
