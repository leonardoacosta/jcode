# Slack Task Playbooks

Load this reference only when a Slack task needs detailed execution guidance. The parent skill's confirmation and privacy rules always apply.

## Locate a conversation safely

1. Confirm the workspace from the visible workspace name or URL.
2. Search by exact channel name or user display name.
3. For ambiguous DMs, open profile details and compare available identity information. Do not guess.
4. Re-snapshot after navigation and verify the conversation header before extracting or composing.

Channel names can repeat across workspaces. Display names can repeat within a workspace. A correct visible name alone is not enough when the action sends or changes data.

## Unread inventory

Opening an unread conversation may mark it read. If preserving state matters, use Activity/Unreads views and sidebar indicators without opening individual items.

Procedure:

1. Snapshot the current Slack view.
2. Locate Activity, Unreads, or the workspace's equivalent view by accessible name.
3. Capture only Slack-displayed counts. Do not derive message counts from the number of visible tree items.
4. Record channels and DMs with unread indicators.
5. Scroll the relevant list with deduplication until items repeat or an explicit end state appears.
6. Report whether conversations were opened and whether read state may have changed.

Evidence should be a minimal list of conversation names and Slack-displayed counts. Avoid a full-sidebar screenshot because it can reveal unrelated private channels and DMs.

## Search conversations

Translate the request into the narrowest useful Slack query:

| Intent | Query shape |
|---|---|
| Exact phrase | `"phrase"` |
| One channel | `term in:channel-name` |
| One sender | `term from:@person` |
| Date window | `term after:YYYY-MM-DD before:YYYY-MM-DD` |
| Files or links | `term has:file` or `term has:link` |
| Reactions | `term has:reaction` |

Search procedure:

1. Snapshot and locate Search by accessible name.
2. Open Search, re-snapshot, locate the input, and fill the exact query.
3. Submit the query, wait briefly, then re-snapshot.
4. Record the query and any visible filters or result count.
5. For each relevant result, capture sender, conversation, timestamp, text needed for the answer, and permalink if visible.
6. Open the result when surrounding messages or thread replies affect interpretation.
7. Continue scrolling only until the requested limit or a proven end state.

Slack search ranking is not chronology and may omit content unavailable under retention or permissions. Report “reviewed N results” rather than “all results” unless the UI proves completeness.

## Read a thread or summarize a channel

1. Establish the requested date range or message limit.
2. Start at the requested anchor, latest visible message, or unread marker.
3. Capture author, timestamp, and message text for relevant messages.
4. Open thread panes when a message has replies. A parent message without its replies can reverse the apparent outcome of a discussion.
5. Separate decisions, proposals, unresolved questions, and action items.
6. Attribute action items only when an owner or commitment is explicit.
7. Note edits, deleted-message placeholders, missing history, retention boundaries, and inaccessible canvases/files.

Do not infer sentiment, employee performance, or personal attributes from message volume or reactions. Do not label a person “most active” unless the user explicitly requests a bounded quantitative analysis and the reviewed coverage supports it.

## Extract cited findings

For each finding retain:

- Workspace and channel/DM
- Sender as displayed
- Slack-rendered timestamp
- Short relevant excerpt or faithful summary
- Thread context status: checked, not applicable, or not reviewed
- Permalink when visible
- Coverage limitation

Prefer short excerpts over copying entire private conversations. Quote only what is necessary for the user's purpose.

## Draft and send a channel message

1. Navigate to the channel and verify the workspace and channel header.
2. Determine whether the message belongs in the channel or a thread.
3. Draft outside the send action. Resolve placeholders and ambiguous mentions.
4. Show the user:
   - workspace and destination
   - channel post or thread reply
   - exact body
   - mentions and attachments
5. Obtain confirmation unless the user already gave exact immediate-send authority with an unambiguous target.
6. Re-snapshot and re-check the destination.
7. Fill the composer. Do not use Enter to submit unless Enter behavior is known and intended.
8. Verify the composed text visually, then click the visible Send control once.
9. Wait and verify the rendered message in the intended destination.

If the outcome is uncertain, search the intended conversation for the exact text before retrying. Never blindly retry a send.

## Draft and send a DM

Apply the channel-send procedure plus identity verification:

- Confirm whether the destination is a person, multi-person DM, or Slack Connect conversation.
- When display names collide, inspect the profile and ask the user to disambiguate.
- Surface external-organization indicators before sending.
- Treat sensitive information as out of scope unless the user explicitly provided it for this recipient.

## Reply in a thread

1. Open the parent message's thread pane.
2. Verify the parent author, timestamp, and excerpt.
3. Verify the composer is labeled as a reply, not the main channel composer.
4. Draft, confirm, send once, and verify inside the thread.
5. If Slack offers “also send to channel,” leave it off unless the user explicitly requested amplification.

## Reactions and other workspace actions

Reactions communicate publicly and may trigger workflows. Before adding or removing one, confirm the exact message and emoji. Verify the count/state after one click.

Require immediate explicit confirmation for:

- Editing or deleting a message
- Pinning or unpinning
- Uploading a file
- Inviting or removing a person
- Joining, leaving, archiving, or renaming a channel
- Changing topic, description, permissions, notifications, or workspace settings
- Starting a huddle or workflow

State whether the action is reversible. Never use a destructive action as a way to test UI access.

## Virtualized lists and scrolling

Slack often renders only a window of content. For any completeness claim:

1. Extract visible stable keys, preferably permalink or conversation plus timestamp.
2. Scroll the correct pane in bounded increments.
3. Re-snapshot and deduplicate keys.
4. Watch for repeated boundary items and explicit end markers.
5. Stop on end marker, repeated window with no new keys, requested limit, or access boundary.
6. Include the stopping condition in the answer.

Avoid CSS selectors tied to Slack's internal class names unless accessible refs cannot address the pane. Internal class names change frequently.

## Safe troubleshooting

### Element missing

- Re-snapshot because refs may be stale.
- Check whether a modal, menu, thread pane, or search overlay has focus.
- Scroll the relevant pane, not the whole page.
- Use a screenshot only if it will not expose unrelated private content.

### Search has no results

- Verify the exact query and date format.
- Remove one filter at a time.
- Confirm the correct workspace.
- Report retention or permission boundaries rather than concluding the message never existed.

### Send appears to fail

- Do not click Send repeatedly.
- Inspect the composer and recent messages.
- Search for the exact body in the intended destination.
- If still uncertain, report uncertainty and ask before retrying.

### Authentication or access blocked

- Let the user perform login, MFA, SSO, or approval.
- Do not ask for credentials.
- Do not switch to another workspace or account to evade the restriction.
