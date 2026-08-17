---
name: slack
description: Safely inspect and operate Slack with agent-browser, including checking unread channels and DMs, searching conversations, reading threads, extracting cited findings, and drafting or sending messages. Use for requests such as 'check my Slack', 'find who said', 'search Slack', 'summarize this channel', or 'send/reply/react in Slack'. Enforces fresh snapshots, privacy-minimized evidence, read-state awareness, and explicit confirmation before consequential workspace actions.
allowed-tools: Bash(agent-browser:*), Bash(npx agent-browser:*)
---

# Slack Browser Automation

Use Slack's rendered UI as the source of truth. Slack is stateful and private: navigation can mark items read, and a single click can communicate externally or change shared workspace state.

## Operating model

Before acting, classify the request:

| Class | Examples | Default |
|---|---|---|
| Read-only | Search, inspect a thread, summarize visible messages | Proceed with minimum necessary access |
| Read-state changing | Open unread channel/DM, mark read/unread | Warn if preserving unread state matters |
| External communication | Send/reply, react, upload, start huddle | Draft, verify target and content, then confirm immediately before action |
| Shared-state/admin | Edit/delete, pin/unpin, invite/remove, join/leave, change topic/settings | Require explicit confirmation of the exact action and target |

A user's request to “send” authorizes preparation, not a blind click. Show the final message and destination and obtain confirmation at the point of send. If the user supplied exact text and explicitly requested immediate sending, you may treat that as confirmation only when the workspace, conversation, and message are all unambiguous. Never infer a recipient from display-name similarity.

## Safety invariants

- **Never send, reply, react, upload, invite, remove, edit, delete, pin, join, leave, or change workspace/channel settings accidentally.** Keep the pointer away from action controls while inspecting.
- **Never press Enter in a message composer until the exact destination and final text are verified.** Prefer multiline-safe insertion and use the visible Send control after confirmation.
- **Never rely on remembered `@e...` refs.** Refs are session- and snapshot-specific and become stale after navigation, dialogs, virtualized scrolling, or UI updates.
- **Never claim completeness from only the visible viewport.** Slack virtualizes long sidebars, channels, threads, and search results.
- **Never equate opening an unread conversation with read-only inspection.** It may clear unread markers or advance the read cursor.
- **Never expose unrelated private messages, member data, raw snapshots, or screenshots in the report.** Capture only evidence needed for the request and redact by default.
- **Never bypass Slack permissions, retention controls, enterprise restrictions, or access boundaries.** Report the limitation instead.
- **Never open suspicious links or download files merely to summarize a conversation.** Ask before leaving Slack or handling untrusted content.

## Core workflow

1. **Resolve the session.** Prefer the user's existing authenticated browser session. If multiple workspaces are visible, identify the requested workspace before proceeding.
2. **Establish scope.** Record the workspace, channel/DM, date range, query, and whether opening unread items is acceptable.
3. **Snapshot before interaction.** Use a fresh interactive snapshot and identify elements by accessible name and role, not expected ref number.
4. **Perform the minimum action.** Avoid unrelated channels, profiles, files, and message history.
5. **Re-snapshot after every state change.** Navigation, search, scrolling, popovers, modals, and thread panes can invalidate refs.
6. **Verify from rendered state.** For writes, verify the visible destination and final content before action, then verify the resulting message or state afterward.
7. **Report with provenance.** Distinguish observed facts, summaries, incomplete coverage, and blocked access.

```bash
# Attach to a browser exposing Chrome DevTools Protocol, or open Slack.
agent-browser connect 9222
# If no authenticated session is available:
agent-browser open https://app.slack.com

# Always inspect current state before clicking.
agent-browser snapshot -i
agent-browser get url
agent-browser get title
```

If authentication is required, let the user complete sign-in and MFA. Do not request passwords or attempt account recovery.

## Decision guide

### Read, search, or summarize

Use a Slack search whenever the requested scope is broader than one visible conversation. Prefer Slack search operators to reduce collection:

- `in:channel-name`, `from:@person`
- `after:YYYY-MM-DD`, `before:YYYY-MM-DD`
- `has:file`, `has:link`, `has:reaction`

After search, record the exact query and visible result count or coverage limit. Open only the results needed for context. A search hit is not sufficient evidence for thread conclusions until replies are inspected.

### Check unread items

First ask whether unread state must be preserved when that intent is not obvious. Prefer unread/activity surfaces that list items without opening each conversation. Report counts only when Slack displays counts. Bold text, dots, or a list of conversations indicate unread state, not a reliable message count.

### Send or reply

1. Navigate to the target and re-snapshot.
2. Verify workspace and channel/DM header. For DMs, verify the profile identity when names are ambiguous.
3. Draft without sending.
4. Present: destination, thread vs channel, exact body, mentions, attachments, and any link preview concern.
5. Obtain confirmation unless exact immediate-send authority is already unambiguous.
6. Re-snapshot, re-verify target, send once, and wait for the rendered message.
7. Report success only after the message appears in the intended conversation. If state is uncertain, do not retry blindly because duplicate messages are worse than a delayed confirmation.

### React, edit, delete, pin, invite, join, leave, or configure

Treat these as consequential workspace actions. State the exact object and effect, request confirmation immediately before the click, execute once, and verify the resulting UI. Deletion, removals, channel settings, and workspace administration require confirmation even if the request is broadly phrased.

## Reliable interaction patterns

```bash
# Find current controls. Do not copy example refs between snapshots.
agent-browser snapshot -i

# Click a ref found in that snapshot, then refresh refs.
agent-browser click @eCURRENT
agent-browser wait 500
agent-browser snapshot -i

# Enter a search query after locating the current search input.
agent-browser fill @eSEARCH '"release blocker" in:engineering after:2026-08-01'
agent-browser press Enter
agent-browser wait 1000
agent-browser snapshot -i

# Extract only a needed element rather than dumping the whole workspace.
agent-browser get text @eRESULT
```

Use fixed waits only as a small stabilization step. If Slack is still updating, inspect again rather than assuming `networkidle` proves the virtualized UI is complete.

## Completeness and evidence

For long or virtualized lists:

1. Capture the first visible set and a stable key for each item, such as channel plus timestamp or permalink.
2. Scroll the correct container in bounded increments.
3. Re-snapshot and deduplicate.
4. Stop when items repeat, an explicit end state appears, or the user-specified limit is reached.
5. Report the stopping condition. Say “visible results” or “results reviewed” rather than “all” without proof.

Screenshots are optional, not the default. Prefer narrow text extraction. If a screenshot is necessary, ensure it excludes unrelated DMs, channels, notification previews, and member details.

## Failures and safe recovery

| Symptom | Likely cause | Recovery |
|---|---|---|
| Element not found | Stale ref, hidden control, wrong pane | Re-snapshot, inspect screenshot, scroll the relevant container |
| Click changes unexpected pane | Ref changed or overlay intercepted | Stop, re-snapshot, confirm workspace/conversation before continuing |
| Search appears incomplete | Filters, pagination, virtualized results | Record query, inspect filters, scroll with deduplication, state coverage |
| Message send uncertain | Lag or composer cleared | Search the intended conversation for the exact message before any retry |
| Permission or retention banner | Access policy | Do not work around it; report what Slack exposes |
| Login/MFA required | No usable authenticated session | Pause for user-controlled authentication |

## On-demand resources

- **For task-specific search, unread, thread, extraction, and write procedures, read** [`references/slack-tasks.md`](references/slack-tasks.md).
- **For a user-requested audit or structured Slack findings report, use** [`templates/slack-report-template.md`](templates/slack-report-template.md).
- **Do not load the report template** for a simple lookup, summary, or message action.
