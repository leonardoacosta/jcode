## ADDED Requirements

### Requirement: Report detected terminal capabilities and active fallbacks
The system SHALL report detected terminal capabilities and active fallbacks.

#### Scenario: Success path
- **WHEN** the user runs `jcode debug terminal` **THEN** output SHALL list each capability with its detected value and active fallback

#### Scenario: Failure path
- **WHEN** a capability is forced off via environment **THEN** the report SHALL show the fallback and rendering tests SHALL pass without that capability

### Requirement: Explicit environment-only presentation fallbacks
The system SHALL support explicit image-off and ASCII-safe environment controls without changing default detection. It SHALL preserve user/chat/tool content, paths, URLs and supplied labels/titles instead of sanitizing the final buffer. It SHALL report bounded ASCII coverage and remaining Unicode chrome truthfully.

#### Scenario: Images explicitly disabled
- **WHEN** `JCODE_IMAGES` is `off`, `false`, `0` or `no`, ignoring case and surrounding whitespace
- **THEN** image detection SHALL return no protocol even with Kitty, iTerm2 opt-in or Sixel environment hints, and direct output and TUI previews SHALL not emit image payloads, including cached native/half-block rendering

#### Scenario: ASCII-safe bounded chrome
- **WHEN** `JCODE_ASCII_SAFE` is `on`, `true`, `1` or `yes`, ignoring case and surrounding whitespace
- **THEN** shared rounded-box borders, shared truncation markers, right-rail vertical borders, primary activity spinners and onboarding pill caps SHALL use ASCII symbols and retain existing terminal-cell width constraints
- **AND** Unicode content and supplied titles/labels SHALL remain unchanged apart from normal width-based clipping
- **AND** the report SHALL state that remaining generated Unicode chrome prevents a full ASCII-output guarantee

#### Scenario: Defaults preserved
- **WHEN** controls are unset or invalid, or ASCII-safe is explicitly false
- **THEN** existing detection and Unicode chrome SHALL remain unchanged, and glyph-safe color quantization SHALL remain independent of ASCII-safe selection
