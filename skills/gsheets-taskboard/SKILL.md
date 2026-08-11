---
name: gsheets-taskboard
description: Priceless-fleet scoped (civalent, lv, modern-visa, tribal-cities — tribal-cities is the primary use case). Read and write Google Sheets task boards / trackers via a service-account-backed MCP server (mcp-gsheets) — no interactive OAuth. Use only when working in one of those priceless client repos and the user references a Google Sheet task board, defect tracker, or spreadsheet they want read or updated; asks to "check the sheet" or "update the board"; wants a new Sheet wired up for programmatic access; or is troubleshooting Google OAuth/Apps Script friction (blocked scopes, account mismatches, login flows that need a code pasted back). Not relevant outside the priceless fleet.
---

# Google Sheets Task Board Access

## Decision: how to reach a Google Sheet programmatically

| Approach | Verdict | Why |
| --- | --- | --- |
| **MCP server (`mcp-gsheets`) + service account** | **Use this** | No interactive OAuth ever. Works headless, works in any session, survives restarts. |
| `gcloud auth print-access-token` + raw Sheets API calls | Don't use | Google **hard-blocks** gcloud CLI's OAuth client from requesting Drive/Sheets scopes ("This app is blocked... sensitive info"). Not a retry-able glitch — gcloud's client isn't verified for those scopes, full stop. |
| Google Apps Script Web App (`doGet`/`doPost`, deployed as "Anyone") | Don't use unless MCP is somehow unavailable | Works in principle but is fragile in practice: deployments don't auto-update on code edits (need explicit "New version" + redeploy), "Who has access" settings don't always stick without an explicit redeploy click, and if the deploying browser session is stuck on the wrong Google account, the script silently fails with "You do not have permission to access the requested document" even when the sheet is nominally link-shared. Link-sharing ("Anyone with the link can edit") only governs **browser** access — it does not grant API/script access to an arbitrary executing identity. |

## Setup (one-time per machine, reusable across every future sheet)

1. **Check if the service account already exists** before creating a new one — reusing it is simpler than provisioning per-sheet:
   ```bash
   gcloud iam service-accounts list --project=<PROJECT_ID> | grep taskboard
   claude mcp list | grep taskboard-sheets
   ```
   If `tc-taskboard-mcp@<project>.iam.gserviceaccount.com` already exists and its MCP server is registered, skip to step 5 — you only need to **share the new sheet** with that same service account email.

2. **Create a dedicated service account** (only if none exists yet):
   ```bash
   gcloud iam service-accounts create <name>-taskboard-mcp \
     --display-name="<Project> Task Board MCP (Sheets read/write)" \
     --project=<PROJECT_ID>
   gcloud services enable sheets.googleapis.com drive.googleapis.com --project=<PROJECT_ID>
   ```

3. **Generate a key** (store outside any git repo, mode 600):
   ```bash
   mkdir -p ~/.config/<name>-taskboard-mcp
   gcloud iam service-accounts keys create ~/.config/<name>-taskboard-mcp/service-account.json \
     --iam-account=<name>-taskboard-mcp@<PROJECT_ID>.iam.gserviceaccount.com
   chmod 600 ~/.config/<name>-taskboard-mcp/service-account.json
   ```

4. **Register the MCP server** (user scope so it's available across all projects/sessions):
   ```bash
   claude mcp add <name>-taskboard-sheets \
     --scope user \
     -e GOOGLE_APPLICATION_CREDENTIALS=~/.config/<name>-taskboard-mcp/service-account.json \
     -e GOOGLE_PROJECT_ID=<PROJECT_ID> \
     -- npx -y mcp-gsheets@latest
   ```
   A brand-new MCP server's tools are **not** reachable in the current live session immediately, even though `claude mcp list` reports `Connected` — that only confirms the subprocess starts. Run `/reload-plugins` (lighter than a full restart) to register its tools, then confirm with `ToolSearch({query: "+<name> sheets"})`.

5. **Share the target sheet** with the service account's email as **Editor** (normal Share-dialog action — no OAuth prompt, no consent screen, because it's a robot identity being granted access, not a human logging in).

## Usage

Tools are named `mcp__<server-name>__sheets_*`. The common ones:

| Tool | Use for |
| --- | --- |
| `sheets_get_values` | Read a range (`Sheet1!A1:I10`) |
| `sheets_append_values` | Add rows to the end of a table (`insertDataOption: "INSERT_ROWS"` to avoid overwriting) |
| `sheets_batch_update_values` | Overwrite specific ranges (e.g. update a status cell in place) |
| `sheets_get_metadata` | Get sheet names/IDs within a spreadsheet |
| `sheets_create_spreadsheet` | Create a brand-new sheet (useful when starting a fresh board rather than fighting an existing one's permissions — see gotcha below) |

The spreadsheet ID is the long string in the URL between `/d/` and `/edit`.

## Status taxonomy pattern (separate "engineer says done" from "confirmed working")

A plain `In progress` / `Completed` / `Blocked` status set collapses two different claims into
one word: "the code is written" and "someone actually confirmed it works where the user will see
it." Those are not the same event — a fix can be code-complete, merged, even deployed, and still
be wrong in production. Default to a status set that keeps them separate:

| Status | Meaning | Set by |
| --- | --- | --- |
| `In progress` | Investigation or implementation ongoing | Engineer |
| `Ready for QA` | Code done (and, if applicable, deployed) — not yet confirmed working live | Engineer |
| `Verified` | Confirmed working against the real, user-facing surface (live prod, or wherever the report originated) | Whoever ran the check — human or agent, but only after an actual runtime check, not a re-read of the diff |
| `Redo` | Reached prod (or wherever it needed to land) and is STILL broken, or broke differently | Whoever ran the check |
| `Blocked` | Can't proceed — needs a decision, missing data, or an external dependency | Either |

This mirrors the harness's own completion-verification discipline (no "done" claim without fresh
runtime evidence) applied to a client-facing board instead of just internal task tracking. Adopt
this 5-state set as the default for a new board unless the user specifies otherwise; if an
existing board already uses a different vocabulary, ask before renaming statuses on live rows —
relabeling in place changes the meaning of every row that already carries the old label, which is
a judgment call the user should make, not an agent inferring "Completed obviously means Ready for
QA now."

## Known gotchas (from a real 2026-07-08 session that hit all of these)

- **Google blocks gcloud's OAuth client for Drive/Sheets scopes.** Don't attempt `gcloud auth application-default login --scopes=...spreadsheets...` — it produces "This app is blocked" and is not fixable by retrying or re-consenting. This is *specific to gcloud's own client_id*; it does not mean OAuth is impossible in general, just that gcloud CLI isn't the vehicle. The service-account path sidesteps this entirely (no OAuth consent screen involved at all).
- **`gcloud auth login` (plain, no custom scopes) is fine** and needed for service-account creation itself (`cloud-platform` scope only, not a blocked scope). If running in a remote/headless dev box with a browser-relay tool (e.g. `cmux`), the plain form completes automatically via the relay. The `--no-launch-browser` remote-bootstrap flow also works for any non-interactive command runner that can't do a live back-and-forth stdin exchange (prints a URL, then a self-contained follow-up command with the code embedded — no typing a code back into a still-running process).
- **A stuck/wrong active Google account in a browser tab is a real, distinct failure mode** from scope/consent issues. If Apps Script or any Google web tool is "stuck" on account X but the target document is owned/shared under account Y, you'll get permission errors indistinguishable at first glance from a scope problem. Diagnostic: check which account is active top-right; if switching is blocked, either share the resource explicitly with the stuck account, or start a private/incognito window signed in directly as the intended account.
- **If an existing sheet's ownership/sharing history is murky** (inherited from a client, unclear who granted what to whom), it's often faster to create a **fresh sheet you fully control** (in a shared Drive folder if one exists for the engagement) than to keep debugging inherited permissions.
- **A blank-looking new tab may not be blank.** `sheets_create_spreadsheet` (or manually creating one via the Drive "New" menu) can pull in a Google template-gallery default (e.g. a Content Calendar tab) instead of a truly empty sheet — check `sheets_get_values` on any unexpected extra tab before assuming it's empty scaffolding.
- **`sheets_format_cells` / `sheets_batch_format_cells` silently no-ops on Google Sheets "Table" objects.** ROOT CAUSE (confirmed 2026-07-08): if the range is a Sheets **Table** (the newer typed-column feature — visible as colored column-type icons + dropdown arrows in the header row, e.g. a person icon for a "Person" column, calendar icon for "Date"), the Table's own column-type layer overrides plain cell `numberFormat` writes. This MCP server only supports the classic range/cell API, not the newer `tables` API resource, so format calls report success but the Table's declared type wins on render — you'll also see a "This value does not match the column type date" tooltip in the UI on affected cells. **Fix: have the user right-click the table → Convert to range** (removes the Table structure, keeps the data) before attempting any format/type changes. After conversion, `sheets_format_cells` with `DATE_TIME` works correctly and sticks. A spreadsheet created via `sheets_create_spreadsheet` or the Drive "New" menu can silently be a Table (inherited from a template) even when it looks like a plain grid — check for the column-type icons/dropdowns if formatting calls aren't taking effect.
- **No dropdown/data-validation setter tool exists** in this MCP server (only `sheets_get_data_validation`, a reader). For an enum-like column (e.g. Status), use conditional formatting (`sheets_add_conditional_formatting`, note: `ranges` takes A1-notation strings like `"Sheet1!D2:D1000"`, not GridRange objects) as the closest achievable substitute, or have the user add real validation manually via Data > Data validation.
- **`sheets_delete_columns` does NOT shift conditional-formatting ranges.** Deleting a column left of a range that has conditional formatting leaves the CF rule pointing at the OLD column index — it silently stops matching real data instead of erroring. Confirmed 2026-07-08: deleted a column left of a Status column with CF rules; the rules kept targeting the pre-deletion column position. There is also no delete/update tool for existing CF rules in this MCP server (only add + get) — the fix is to add fresh correct rules at the new position; stale rules pointing at the wrong column are harmless no-ops (they just never match) rather than actively wrong, so leaving them is fine.
- **A "Convert to range" Table can leave a leftover `bandedRange` object attached to the sheet** (alternating-row Table styling) even after the typed-column validation layer is gone. `sheets_get_conditional_formatting` returns `bandedRanges` alongside CF rules — check both after a Table conversion. No removal tool exists for banded ranges in this MCP server; it's cosmetic (not a data-integrity issue) so leaving it is a reasonable default unless the user asks for it removed.
- **`sheets_delete_rows` on a range far below real data can (rarely, root cause unconfirmed) coincide with column-order corruption in the rows ABOVE the deleted range.** Confirmed 2026-07-08: after `sheets_delete_rows("Sheet!11:27")` to clean up a gap, rows 1-8 (untouched by the delete, well above row 11) came back with their first column's data relocated to column G — every other column shifted left by one. Rows written fresh after the delete were unaffected. Root cause not isolated (possibly interaction with the leftover banded-range object from a prior Table conversion — see above), but the practical lesson: **always re-read the full data range after any delete_rows/delete_columns call and verify column alignment against the header row before trusting the write succeeded**, especially on a sheet that was ever a Table. If corrupted, the fix is a single full-range `sheets_update_values` rewrite with explicit correct column order — don't try to patch individual cells.
