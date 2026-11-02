## ADDED Requirements

### Requirement: Remote sessions copy to the viewing client

When the TUI runs in a remote session, the clipboard MUST be written only through OSC 52 on the TUI's terminal. Native clipboard paths on the host MUST NOT be used. A session is remote when `HERDR_ENV`, `SSH_CONNECTION`, `SSH_TTY`, or any `MOSH_` variable is set, or when `JCODE_CLIPBOARD=osc52` is set. `JCODE_CLIPBOARD=native` MUST force the local path.

#### Scenario: Herdr session on a host with a display
- **WHEN** `HERDR_ENV` and `WAYLAND_DISPLAY` are both set and the user copies text
- **THEN** the TUI emits OSC 52 and does not invoke `wl-copy`, `xclip`, `xsel`, or `arboard`

#### Scenario: Plain SSH without Herdr
- **WHEN** only `SSH_CONNECTION` is set and the user copies text
- **THEN** the TUI emits OSC 52

#### Scenario: Local desktop session
- **WHEN** no remote markers are set
- **THEN** the existing native-first order is unchanged

#### Scenario: Forced native override
- **WHEN** `JCODE_CLIPBOARD=native` is set and `SSH_CONNECTION` is set
- **THEN** the TUI uses the native path

### Requirement: Links can be copied without terminal selection

The TUI MUST provide a `/links` picker that lists link destinations from the current transcript, newest first, and a key that copies the most recent destination. The copied text MUST be the exact destination, with no wrapping, padding, or border characters.

#### Scenario: Copy a wrapped long URL
- **WHEN** a URL longer than the terminal width is rendered and the user copies the last link
- **THEN** the clipboard text equals the URL exactly

#### Scenario: No links available
- **WHEN** the transcript has no links and the user invokes the copy-last-link key
- **THEN** the TUI shows a "No links" notice and does not write to the clipboard

### Requirement: Mouse capture can be toggled

The TUI MUST provide a `/mouse` command and a key that switch mouse capture on or off, and MUST show the current state in a status notice.

#### Scenario: Switch capture off for native selection
- **WHEN** the user switches mouse capture off
- **THEN** the TUI disables mouse reporting and the client terminal's native drag selection works until capture is switched back on

### Requirement: Remote link opens reach the viewing client

In a remote session, opening an HTTP(S) link MUST copy it through OSC 52 and show the full URL in a status notice that does not wrap. If `JCODE_LINK_OPEN_SSH_HOST` is configured and the Chrome open succeeds, the copy still happens.

#### Scenario: iOS client with no Mac host
- **WHEN** the session is remote, no open host is configured, and the user opens an HTTPS link
- **THEN** the TUI copies the URL through OSC 52, shows "Copied link", and does not launch a browser on the host
