---
name: electron
description: Automate and test Electron desktop applications through agent-browser and the Chrome DevTools Protocol (CDP). Use when launching an Electron app with remote debugging, connecting to an existing CDP port, selecting Electron windows or webview targets, inspecting desktop UI, or troubleshooting CDP automation for apps such as VS Code, Slack, Discord, Figma, and Notion. Trigger keywords include Electron, desktop app automation, remote-debugging-port, CDP, DevTools port, Electron window, webview, and agent-browser connect.
allowed-tools: Bash(agent-browser:*), Bash(npx agent-browser:*)
---

# Electron Automation with agent-browser

Treat an Electron app as a set of Chromium CDP targets, not as one browser tab. The reliable loop is:

1. Start a fresh app process with a loopback-only remote-debugging port.
2. Connect one named agent-browser session to that port.
3. List targets and select the intended window.
4. Snapshot, act through fresh refs, then re-snapshot after UI changes.
5. Close the app or restart it without remote debugging when finished.

## Choose the connection path

| Situation | Action |
|---|---|
| You can restart the app | Relaunch it with `--remote-debugging-port=<port>`, then `connect` |
| The app already exposes CDP | Connect to its port or full HTTP/WebSocket URL |
| Several Chromium/Electron processes are running | Use an explicit port, not `--auto-connect` |
| You must control several apps | Give each app a unique port and each connection a named session |
| The product strips or disables Chromium debugging flags | Use the app's documented debugging mode or test build; agent-browser cannot create a CDP endpoint that the app does not expose |

`--auto-connect` is convenient only when target identity is unambiguous. Explicit ports are safer for desktop automation.

## Launch with remote debugging

First quit every existing process for the app. Many Electron apps reuse the original process, so passing a flag to a second launch may open a window in a process that never received it.

### macOS

```bash
open -na "Slack" --args --remote-debugging-port=9222
open -na "Visual Studio Code" --args --remote-debugging-port=9223
```

`-n` requests a new instance. If the product prevents multiple instances, quit it fully and use `open -a ... --args` instead.

### Linux

```bash
slack --remote-debugging-port=9222
code --remote-debugging-port=9223
```

Package-specific launchers may have different binary names. Pass the flag to the actual Electron executable if a wrapper consumes unknown arguments.

### Windows PowerShell

```powershell
& "$env:LOCALAPPDATA\slack\slack.exe" --remote-debugging-port=9222
& "$env:LOCALAPPDATA\Programs\Microsoft VS Code\Code.exe" --remote-debugging-port=9223
```

Paths vary by installer and install scope.

### Confirm the endpoint before connecting

```bash
curl --fail --silent http://127.0.0.1:9222/json/version
agent-browser --session electron-app connect 9222
```

A successful `/json/version` response proves that a CDP endpoint is listening. It does not prove that the desired Electron window is the currently selected target.

## Select the correct Electron target

Electron can expose main windows, auxiliary windows, extension hosts, and web contents as separate targets. Inspect the target list after connecting:

```bash
agent-browser --session electron-app tab list
agent-browser --session electron-app tab t2
agent-browser --session electron-app snapshot -i
```

Use the stable `t<N>` id printed by `tab list`. Do not assume list index `2` means `t2`, and do not use unsupported URL-switch syntax. Re-run `tab list` after opening or closing app windows.

An embedded `<webview>` may appear as a separate CDP target when the app's Electron configuration exposes it. If it does not appear, agent-browser cannot cross into it from the embedder merely because the DOM contains a `<webview>` element.

## Interaction loop

```bash
agent-browser --session electron-app snapshot -i
agent-browser --session electron-app click @e5
agent-browser --session electron-app snapshot -i
agent-browser --session electron-app fill @e8 "search query"
agent-browser --session electron-app press Enter
agent-browser --session electron-app wait 500
agent-browser --session electron-app snapshot -i
```

Refs are snapshot-scoped. Re-snapshot after navigation, modal changes, target switches, window reloads, or substantial rerenders before reusing a ref.

For Monaco, CodeMirror, ProseMirror, Lexical, or other custom editors, focus the editor first and then use raw keyboard input:

```bash
agent-browser --session electron-app click @e12
agent-browser --session electron-app keyboard type "const ready = true;"
# If key handlers transform or reject text:
agent-browser --session electron-app keyboard inserttext "const ready = true;"
```

`keyboard inserttext` bypasses key events. Use it only when real keystrokes are not required by the app.

## Multiple apps without cross-talk

```bash
agent-browser --session slack connect 9222
agent-browser --session vscode connect 9223

agent-browser --session slack tab list
agent-browser --session vscode tab list
agent-browser --session slack snapshot -i
agent-browser --session vscode snapshot -i
```

Keep the session flag on every command. An omitted `--session` targets the default session and can silently act on the wrong app.

## Evidence and diagnostics

```bash
agent-browser --session electron-app screenshot ./electron-state.png
agent-browser --session electron-app screenshot --annotate ./electron-annotated.png
agent-browser --session electron-app console
agent-browser --session electron-app errors
agent-browser --session electron-app get url
```

Annotated screenshots help reconcile visual controls with refs. Accessibility snapshots can omit canvas-rendered or inaccessible controls, so use screenshots plus `eval` or coordinate input only after confirming refs and semantic locators are unavailable.

To request dark rendering for CDP-controlled content:

```bash
agent-browser --session electron-app --color-scheme dark snapshot -i
```

This emulates the page color-scheme preference. It does not necessarily change native title bars, OS chrome, or an app-specific theme stored in its settings.

## Troubleshooting decision tree

### Connection refused

1. Check `curl http://127.0.0.1:<port>/json/version`.
2. If it fails, confirm the app was fully stopped before relaunch.
3. Confirm the launcher passed the Chromium flag through.
4. Check port ownership with `lsof -nP -iTCP:<port> -sTCP:LISTEN` on macOS/Linux or `Get-NetTCPConnection -LocalPort <port>` on PowerShell.
5. Relaunch on a different unused port if necessary.

### Connected, but the wrong UI appears

1. Run `tab list`.
2. Select the target by its stable `t<N>` id.
3. Snapshot again after switching.
4. If the desired window is absent, open it in the app and list targets again.

### Snapshot is sparse or controls are missing

1. Take an annotated screenshot.
2. Try a full snapshot without `-i`.
3. Check whether the control is inside another target or webview.
4. For canvas or custom-rendered UI, prefer app keyboard shortcuts or carefully verified coordinates as a last resort.

### Typing fails

1. Click or focus the editor.
2. Use `keyboard type` for real key events.
3. Use `keyboard inserttext` only if the editor rejects synthesized keystrokes.
4. Re-snapshot to verify the resulting value instead of assuming success.

## Hard rules

- **NEVER expose the debugging endpoint to an untrusted network.** CDP can read app content, execute JavaScript, and act with the logged-in user's privileges. Use a local port and do not add `--remote-debugging-address=0.0.0.0`.
- **NEVER assume every Chromium-based desktop app honors Electron flags.** Vendor launchers, hardened builds, and non-Electron shells may reject or strip them.
- **NEVER connect by auto-discovery when multiple candidate apps are running.** A valid connection to the wrong target is more dangerous than a failed connection.
- **NEVER reuse refs after switching targets or changing the UI.** Stale refs can fail or invoke the wrong control.
- **NEVER treat a successful click or fill command as outcome proof.** Snapshot, read state, or capture evidence after consequential actions.
- **NEVER automate destructive, financial, credential, or message-sending actions without the user's explicit intent and an appropriate confirmation boundary.** Electron access inherits the user's authenticated desktop session.
- **NEVER leave remote debugging enabled longer than needed.** Stop the debug-launched process when the task ends.

## Scope boundary

agent-browser controls Chromium-rendered web contents exposed through CDP. It does not control native OS dialogs, menus rendered outside the web contents, system permission prompts, or arbitrary non-Electron desktop apps. Use an OS-level automation tool for those surfaces.
