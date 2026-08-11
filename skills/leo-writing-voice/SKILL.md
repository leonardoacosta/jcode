---
name: leo-writing-voice
description: Leo's writing voice — for Teams/Slack/email drafts, chat replies, and any user-facing prose written on Leo's behalf. Covers register (peer vs stakeholder), cadence (split-send, inline data), Southern-casual greetings, and structural rules. Load this BEFORE drafting any message to a human on Leo's behalf.
allowed-tools: Read, Glob, Grep
provenance:
- 'Plain text: every message in Leo''s 2026-04-21 Teams transcript, no `**bold**` or `*italic*`.'
- 'line 55: draft vs sent diff on 2026-04-21 group-chat update — Leo replaced 2 em-dashes with commas
  inline while keeping them in ARM-verification bullets.'
- 'Env names proper-case in prose: Leo''s 2026-04-21 15:47 send changed `"PROD"` → `"Prod"` in prose while
  preserving `KV-WHS-346-WS-CUS-DEV`.'
- 'Address stakeholders with a comma: John Brilhart 1:1 2026-04-21 19:33 — Leo sent `"Hey, got a minute"`,
  not `"Hey John, got a minute"`.'
- 'No sync-time offers: every peer-register send on 2026-04-21 removed these offers that were in Claude''s
  drafts.'
- 'Southern casual greetings: group chat 2026-04-21 15:30 `"Morning Yall, sorry for the delay"` + close
  `"Thank yall"`.'
- 'Chat register, not email: 2026-04-21 16:00 sent `"...deletion). no new apps or secrets to create."`
  — lowercase after period.'
- 'Drop Teams-hostile markdown: draft `"brownandbrowninc-*-SC-*"` sent as `"brownandbrowninc--SC-"`.'
- 'line 73: Draft vs sent diff 2026-04-27 — Leo replaced "still settling" framing with plain "broke my
  pieces" in CT stakeholder update.'
- 'line 79: Draft vs sent diff 2026-04-27 — Leo replaced two-option ask with single Wednesday ask.'
- 'line 85: Draft vs sent diff 2026-04-27 — Leo added "I''m Frantically trying to complete the bugs I
  have" to CT stakeholder update; Claude''s draft had no personal-state line.'
- 'line 92: Draft vs sent diff 2026-04-27 — Leo replaced "father" → "dad", "surfaces" → "pages"/"pieces"
  throughout CT update.'
- 'line 108: Explicit instruction from Leo in session 2026-04-27 — "stakeholders typically don''t care
  to know the minutiae or technical details. They care about what is being delivered, what is blocking,
  and how they can help."'
- 'line 118: Draft vs sent diff 2026-04-27 — Leo dropped "Morning Yall" + "before the sync" framing for
  "Wanted to give an update," in CT continuing-thread update.'
- 'Close: Draft vs sent diff 2026-04-27 — Leo dropped "Thank yall" on a continuing-thread CT update, kept
  the request paragraph as the natural endpoint.'
- 'line 145: Draft vs sent diff 2026-04-27 — Leo collapsed all engineering jargon into UX-language outcomes
  in CT stakeholder update, dropping primitive names, RAG terms, and security-fix specifics.'
- 'No commit-level granularity: Draft vs sent diff 2026-04-27 — Leo dropped the entire WIP commit-list
  breakdown, replaced with "matrix work needs updates".'
- 'line 153: Draft vs sent diff 2026-04-27 — Leo added "for Wednesday" anchor to doc-request paragraph
  after rescheduling the meeting to Wednesday.'
- 'One blocker per message: Leo dropped the entire §6b/stalled-queue/AFD-pivot paragraph from Claude''s
  Dasu 1:1 draft, keeping only the TASK0687833 heads-up.'
- 'Pattern: ask-then-deliver split-send: John Brilhart 1:1 2026-04-21 19:33-19:34 — Leo sent 2 separate
  messages, not the one-shot Claude drafted.'
- 'Zero context padding after "yes.": Leo''s 2nd send to John had zero context, just the MI data.'
- 'Skip greetings: Draft vs sent diff 2026-05-21 — Leo dropped `"Morning yall, sorry for the late ticket
  on this one."` opener from OfficeIndextoPIPs admin-consent ticket.'
- 'Skip apologies: same diff — Leo dropped the apology clause entirely.'
- 'Skip backstory and reasons: same diff — Leo deleted the entire context paragraph including team blocked-since
  dates and developer impact.'
- 'Skip closures: same diff — Leo dropped the closing `"Thank yall"`.'
- 'Skip references to other systems: same diff — Leo deleted the reference line `"Reference: ADO PIPS
  48266. Therese Lay can verify Futran is unblocked once consent lands."`'
- 'Do NOT pre-solve for the operator: same diff — Leo deleted the entire `"Direct admin consent URLs for
  whoever picks this up..."` block with 4 adminconsent URLs.'
- 'Legible name first, ID second: same diff — Leo kept the `346-OfficeIndextoPIPs-DEV appId 7aaa2287-...`
  format unchanged.'
- 'Bullet lists for any 2+ items: same diff — Leo kept all bullet lists intact.'
- 'Consolidate scope lists: same diff — Leo deleted the `"Stage additionally needs admin consent on the
  base scopes..."` paragraph and folded `User.Read` into the main scope bullet list.'
- 'Do NOT mention what you do/don''t have permissions for: same diff — Leo deleted the role-self-description
  sentence.'
- 'line 291: ...'
- 'line 297: <source> <date> — <specific behavior>.'
---

# Leo's Writing Voice

> **Last updated:** 2026-05-21 · **Evidence base:** 20+ Teams messages Leo sent on 2026-04-21 to Dasu, Daniel, John Brilhart + Nova's operator profile at `~/dev/nv/.nova/memory/user.md` + draft-vs-sent diff on 2026-04-27 CT stakeholder update + SNOW ticket draft-vs-sent diff on 2026-05-21 (OfficeIndextoPIPs admin consent ticket).
>
> **Rule for updates:** every addition to this skill MUST cite evidence — a specific message, diff between draft/sent version, timestamp, or explicit instruction from Leo. Don't encode rules from vibes; encode them from observations. See § Update Protocol at the bottom.

## When to load this skill

- Drafting a Teams/Slack/email message on Leo's behalf
- Replying to a chat thread
- Writing a SNOW ticket description (use SNOW register below, NOT stakeholder register)
- Writing a code-review comment
- Any prose that will ship unedited to a human

## Do NOT apply this voice to

- Code comments, commit messages, OpenSpec proposal.md files (follow project conventions instead)
- Technical documentation or specs (use factual/reference tone)
- Internal agent-to-agent prompts (use direct instructional tone)

---

## Register selection (always step 1)

Leo modulates by **relationship**, not by content. Identify which register applies before drafting:

| Signal | Register |
|---|---|
| Cross-team ask, ticket-referenced, one of several stakeholders | **Stakeholder** |
| First interaction in a chat / cold DM | **Stakeholder** (warmer, more context) |
| Direct collaborator, prior Teams history, they've helped each other before | **Peer** |
| Quick 30-second favor from someone senior in their domain | **Peer** |
| Explaining a rotation plan, setting ticket expectations | **Stakeholder** |
| Handing off concrete data for someone to execute | **Peer** |
| Filing a SNOW ticket (Cloud Access Request, RITM, IAM ask) | **SNOW** |

When in doubt between Teams registers, default to stakeholder (slightly more structured — safer). SNOW is its own register — never use stakeholder voice in a ticket body.

---

## Baseline rules (both registers)

**Plain text.** No bold, no italic, no markdown emphasis. Teams strips some of it; Leo's habit is to trust words over formatting.

**Commas over em-dashes inside sentences.**
- ✅ `"3 of 4 do (DEV/TEST/STAGE, I'll paste the list)"`
- ❌ `"3 of 4 do (DEV/TEST/STAGE — I'll paste the list)"`

Em-dashes are OK in data bullets where they separate name↔location: `"✅ KV-WHS-346-WS-CUS-DEV — RG-WHS-346-Wholesale-CentralUS-DEV"`.

**Env names proper-case in prose.** `Prod`, `Dev`, `Test`, `Stage` — not `PROD`/`DEV`. Exception: literal ARM resource names keep their casing (`KV-WHS-346-WS-CUS-DEV`, `RG-WHS-346-Wholesale-CentralUS-STAGE`).

**Address stakeholders with a comma.** `"Daniel,"` not `"Daniel —"`. In 1:1s, drop the name entirely — the thread already shows who's talking.

**No sync-time offers.** Drop phrases like `"I'll jump on a screenshare"`, `"happy to pair"`, `"whenever works for you"`, `"let me know if you're still blocked"`. Leo solves async by preference.

**Southern casual greetings** (stakeholder register). `"Morning Yall"` / `"Thank yall"` / `"Good morning!"` — Leo has explicit Southern plural.

**Chat register, not email.** Lowercase after periods OK. Apostrophes sometimes omitted. Run-ons fine. Write how you'd talk.

**Drop Teams-hostile markdown.** `*text*` renders as italic and bare `*` gets stripped. For identifier patterns, drop the asterisks.

**Self-blame plain, not softened.** When you broke something, say you broke it. Don't reach for "still settling" / "in flight" / "mid-flight" hedges when "I pushed changes that broke my pieces" is honest and shorter.
- ✅ `"today I pushed a few changes that broke my pieces"`
- ❌ `"a couple of my surfaces are still settling from today's changes"`



**Concrete dates beat optioned asks.** When proposing a reschedule or a deadline, pick one. Optioning ("next week or skip altogether") puts the work back on the recipient.
- ✅ `"Could we push this meeting for Wednesday?"`
- ❌ `"I'd push the meeting to next week or skip it altogether"`



**Personal-state disclosure is part of the voice, not a violation.** When stress, urgency, or current-context is real, voicing it lands as honesty, not unprofessionalism. One short clause is enough.
- ✅ `"I'm frantically trying to complete the bugs I have"`
- ❌ over-explaining the stress, or hiding it behind softened framing



**Colloquial vocabulary even with stakeholders.** Prefer everyday words over precise jargon when the precise word adds no clarity.
- ✅ `"dad"` not `"father"`
- ✅ `"pages"` / `"pieces"` not `"surfaces"` / `"modules"`
- ✅ `"stuff"` as casual filler is fine (`"I added a lot of quality of life stuff"`)



---

## Stakeholder register

For: Daniel, Dasu, cross-team handoffs, first-time contacts.

**Framing principle: stakeholders care about three things.** Before drafting, organize the message around these — anything else is filler.

1. **What's being delivered** — visible outcomes, in user-facing terms
2. **What's blocking** — what's stuck, half-landed, or not demo-ready, and why
3. **How they can help** — concrete asks, with dates if reschedules are involved

If a sentence doesn't serve one of those three, cut it. Implementation details, commit names, library names, architecture jargon, internal status terms — none of those land for stakeholders. They make the message longer without making it more useful.



**Opener:** apology-if-delayed + ticket ID on the **first message of a new thread / cold open**.
- ✅ `"Morning Yall, sorry for the delay, catching up on TASK0687833"`
- ❌ `"thanks for the ping, I have the answer"`

**For continuing-thread updates** (known channel, no fresh greeting needed): drop `"Morning Yall"` and start with the update directly.
- ✅ `"Wanted to give an update,"` then continue
- ❌ `"Morning Yall, wanted to give yall a heads up"` (over-formal for an in-progress thread)



**Structure:** allow bulleted breakdowns, data blocks with em-dashes, explicit context, named stakeholders addressed with a comma (`"Daniel, if the Portal is showing..."`).

**Phrasing:** permission-granting, not commanding.
- ✅ `"go ahead and drop it from the ticket"`
- ✅ `"we can drop it"`
- ❌ `"drop it"`

Softer framing:
- ✅ `"is correctly not in scope"` (factual)
- ❌ `"is correctly missing"` (absence)

**Softening hedges.** `"actually"` before a claim most likely to be contested. One hedge per message.
- ✅ `"his side is actually already done"`
- ❌ `"his side is done"` (too bald)

**Close** on the first message of a new thread / cold open: `"Thank yall"` / `"Thanks"`. Follow-ups, in-progress thread updates, and continuation messages skip the close.

**Translate engineering work to UX language for non-engineering stakeholders.** Replace primitive/library/architecture names with what the user actually sees. Stakeholders care about visible behavior, not implementation.
- ✅ `"async ui, assistant popups, shortcuts on the project page, alerts"`
- ❌ `"AsyncButton + useAsyncAction, ConfirmDialog, coaching CTAs, P1 security guards (corpus admin guard, cross-org auth)"`

Same for retrieval/backend work:
- ✅ `"James shipped a pretty big rewrite of the corpus search"`
- ❌ `"domain-decomposed retrieval, HyDE query rewriting, reranking, cross-ref traversal"`



**No commit-level granularity.** Don't share git commit names, file paths, or wip-flag references. High-level state only ("matrix work needs updates"), not git-state ("his last commit was a wip on matrix scope fixes / jurisdiction picker / live-run diagnostics").

**Link follow-up asks to the new commitment.** When rescheduling, tie any related request to the new date so the recipient sees the dependency.
- ✅ `"having those would help James's side for Wednesday"`
- ❌ `"having those would help us close the loop"` (no anchor)



**One blocker per message.** When a chat has one active issue, reply to just that issue. Hold unrelated follow-ups (§6b gaps, stalled tickets, pivot memos) for separate messages.

**State over dates in parentheticals.**
- ✅ `"his side is actually already done (new client secret is in place, 2 olds flagged)"`
- ❌ `"his side is already done from 2026-04-13"`

**Don't narrate side channels.** `"Update:"` not `"Update from 1:1:"` — content is what matters; naming the private channel breaks confidence.

### Stakeholder example

> Morning Yall, sorry for the delay, catching up on TASK0687833.
>
> Quick clarifier first: the 4 names you're seeing are Key Vaults, not App Services.
>
> [data block with em-dashes]
>
> Daniel, if the Portal is showing them as missing, most likely the sub selector is on a different subscription.
>
> Dasu, once Daniel confirms he sees them, you're clear to roll entraIdClientSecret into the 3 KVs.
>
> Thank yall

---

## Peer register

For: direct collaborators (John Brilhart-type), people who've helped Leo before.

**Pattern: ask-then-deliver split-send.** ALWAYS. Send 1: `"Hey, got a minute for X?"`. Wait for `"yeah whats up?"`. Send 2: the payload. **Never one-shot a peer with a wall of data.**

**No name in greeting.** 1:1 Teams already shows who's talking.
- ✅ `"Hey, got a minute..."`
- ❌ `"Hey John, got a minute..."` (redundant)

**Zero context padding after "yes."** Once peer opts in, trust them with the payload and only the payload. Drop the why, timeline, `"been blocked since X"`, off-ramps. They know their domain.

**Inline colon-separated data for ≤4 values.** Flow in one line with colons, not multi-line bullets. Feels conversational.
- ✅ `"adb-X: Display name: Y Client ID: Z Auth source: A Entitlements: B"`
- ❌ A formatted list with bullets and labels

For >4 values, hyphen-bullet list is acceptable.

**No close.** No `"thanks"`, no `"lmk"`, no `"Thank yall"`. End at the payload.

**No off-ramp once yes is given.** `"if it's not your thing lmk who to bug"` belongs in cold opens, not after opt-in.

### Peer example

> **Send 1:** Hey, got a minute for a databricks admin thing?
> **Send 2** (after "Yeah, whats up?"): Need our dc MI registered as a workspace SP on adb-2984522946546137: Display name: ID-WHS-346-DOC-CentralUS-DEV Client ID: 8fe9725c-ff3b-4eca-955c-0c4cdb119f18 Auth source: Microsoft Entra ID managed Entitlements: Workspace access + Databricks SQL access

---

## SNOW register

For: SNOW ticket bodies (Cloud Access Requests, RITMs, any IAM ask). The recipient is a queue operator, not a peer. They process tickets all day, they don't care about your story.

**Framing principle: outcome only, no narrative.** A SNOW ticket is a work order. The reader's job is to read the desired state and execute. Anything beyond the desired state slows that processing down.

**Skip greetings.** No `"Morning yall"`, no `"Hi"`, no recipient address. The ticket form has a `Requested For` field that handles attribution.

**Skip apologies.** No `"sorry for the late ticket"`, no `"sorry to bother"`. The queue doesn't care about your timing.

**Skip backstory and reasons.** Do NOT explain why the request matters. No `"Therese's team has been blocked since 5/15"`, no `"the Futran developers can't get past..."`, no business context. The IAM operator approves or denies based on policy, not sympathy.

**Skip closures.** No `"Thank yall"`, no `"Thanks"`, no sign-off. End at the data.

**Skip references to other systems.** No ADO ticket numbers, no `"Therese can verify"`, no `"see related thread"`. The ticket stands alone. If they need context they'll comment on the ticket.

**Do NOT pre-solve for the operator.** No URLs that "make their job easier", no step-by-step instructions, no "here's the button to click". The operator knows their tooling — telling them how to do their job reads as condescending.

**Legible name first, ID second.** When referencing a resource, lead with the human-readable name and trail the GUID/appId. Never lead with an ID.

**Bullet lists for any 2+ items.** SNOW renders bullets cleanly and operators scan them. Don't prose-bury resource lists.

**Consolidate scope lists.** If an env needs strictly more than what's in the unified list, ADD the missing scopes to the main list instead of writing a follow-up paragraph like `"Stage additionally needs..."`. One list, all scopes.

**Do NOT mention what you do/don't have permissions for.** No `"I'm an Application Owner but not an Application Administrator"`, no `"the button is disabled for me"`. The fact that you filed a ticket means you can't do it yourself, that's the entire signal needed.

### SNOW example

> The four app registrations that need admin consent:
>
> - 346-OfficeIndextoPIPs-DEV    appId 7aaa2287-bc8d-4001-8274-1618a6190582
> - 346-OfficeIndextoPIPs-QA     appId 3e954f4e-be78-4c16-a43e-d15061f5475f
> - 346-OfficeIndextoPIPs-STG    appId 89806fe8-33fb-4c02-8122-e28c31b492e7
> - 346-OfficeIndextoPIPs-PRD    appId e958376e-f27b-479e-8351-b9b2861d2d18
>
> Delegated Microsoft Graph scopes that need admin consent:
>
> - Mail.Read.Shared
> - Mail.ReadWrite
> - Mail.ReadWrite.Shared
> - Mail.Send
> - Mail.Send.Shared
> - Calendars.Read
> - Calendars.Read.Shared
> - User.Read

---

## Nova's operator profile (cross-context behavioral rules)

Source: `~/dev/nv/.nova/memory/user.md` (maintained by Nova, Leo's personal agent).

These are behavioral rules that apply across every interaction, not just Teams drafts:

- **Conversational for back-and-forth, concise/bulleted for reports and digests.** Match the mode to the task.
- **P0/P1 notifications only, once or twice a day.** Don't send every TTS notification — filter.
- **Diagnose first, then fix.** No band-aid fixes without understanding root cause.
- **Provide CLI commands only when explicitly asked.** Don't volunteer shell snippets unsolicited.
- **Action labels required:** `[You]` = Leo's action, `[Me]` = agent's action, `[Confirm -> Me]` = agent acts after Leo confirms. Use these when summarizing next steps or delegating.
- **Clear ownership distinction** between Nova's/Claude's tasks and Leo's tasks.
- **Leo dislikes**: filler, summaries-when-commands-were-asked, and proactive suggestions without being asked.

Apply these in any mode, not just Teams drafts.

---

## Draft recipe

1. **Identify register** (stakeholder vs peer vs SNOW). If unknown between Teams registers, default stakeholder. SNOW is its own thing — only use it for ticket bodies.
2. **Peer register:** draft the ask-send first (1 line, no name, `"got a minute"`). Prepare the payload-send separately. Don't include context/off-ramp unless asked.
3. **Stakeholder register:** structured paragraph or bullet breakdown, explicit context, permission-granting, comma-addressing, optional `"Thank yall"` close on first send.
4. **SNOW register:** outcome only. No greeting, no apology, no backstory, no close, no cross-system refs, no pre-solved URLs. Bullet lists for resources and scopes. Legible name before ID.
5. **Always:** plain text, commas over em-dashes inside sentences, no bold/italic, proper-case env names.
6. **Before sending the draft to Leo:** scan for violations — for Teams: bullet points where prose would work, `PROD` where `Prod` would work, em-dashes inside sentences, sync-time offers. For SNOW: any greeting/apology/reason/close/URL/role-disclosure remnants.

---

## Update Protocol

**This skill SHOULD be updated** when Leo rewrites an agent draft or gives explicit voice feedback. Updates follow:

1. **Capture the evidence first.** The exact draft Claude wrote + the exact message Leo sent. Timestamp the source (chat ID or session date). Without evidence, skip the update.
2. **Identify the pattern behind the diff.** A single word change isn't a rule; a repeated pattern across messages is.
3. **Add the rule in the appropriate section** (baseline / stakeholder / peer / nova). Put the
   backing evidence in the frontmatter `provenance:` array, labelled with the rule it supports —
   NOT inline beside the rule (`rules/CORE.md` § Comment Discipline).
4. **Update `Last updated:` date** in the front-matter.
5. **Never remove existing rules silently.** If a rule is superseded by new evidence, mark the old rule `~~struck through~~` with a note pointing at the new rule, don't delete.

### Citation format

A `provenance:` entry: `"<rule label>: <source> <date> — <specific behavior>."`

Sources accepted:
- `Teams 1:1/group chat with <person> <date>` (pull via Graph API with SOCKS if needed)
- `Draft vs sent diff <date>` (when Leo edits a drafted message before sending)
- `Explicit instruction from Leo in session <session-date>`
- `Nova operator profile (~/dev/nv/.nova/memory/user.md)` for behavioral rules
- `Pattern across N messages <date range>` (when a single pattern appears repeatedly)

**Not accepted as evidence:**
- Gut feeling / vibes
- "Feels right for peer register"
- "Matches Southern style" (without a specific observed instance)

---

## Related

- **Per-project expansions** may live under `~/.claude/projects/<project>/memory/user_writing_style_*.md` for project-specific register variants. Those should reference this skill as the canonical base and only encode project-local overrides.
- Leo's operator profile: `~/dev/nv/.nova/memory/user.md` (Nova-maintained)
- Original project-scoped memory (deprecated, see Migration Note): `~/.claude/projects/-home-nyaptor-dev-ws/memory/user_writing_style_teams.md`

### Migration Note (2026-04-21)

The ws-scoped memory file has been promoted to this global skill. The project memory now points here. If you're looking at this skill and want to propose a change, put the evidence in the project memory where the observation happened, then fold it into this skill with citation.
