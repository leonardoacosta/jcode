
# UI Microcopy

> Reference for the `frontend-design` skill — the text *inside* the UI.
>
> **Scope = UI microcopy only:** error messages, CTA labels, empty-state text, form labels.
> General prose voice for Teams, Slack, email, and chat replies belongs to repository-local
> writing guidance. Load that guidance instead for human-facing prose.
>
> Adapted from `d-o-hub/anti-ai-slop` `copy-rewrites.md` (audited 2026-05-24).

## The principle

AI copy is enthusiastic, hollow, circular, and aggressively affirmative. UI microcopy should be
the opposite: specific, active, short, and written for one person. Say what happened, why, and
exactly what to do next.

## Error Messages

| Before (slop) | After |
|---|---|
| "Oops! Something went wrong. Please try again later." | "Couldn't save — the server timed out. Your work is still here. Try again, or [download a backup]." |
| "Invalid input. Please check your entries." | "Email addresses need an @ sign — like name@example.com" |
| "An error occurred." | Name the error + the recovery action. Never blame the user for the product's failure. |

Rule: explain what happened, why, and the exact next step. Never "Invalid input."

## CTA Labels

| Before | After |
|---|---|
| "Click here" | "Download the report" |
| "Submit" | "Send the invite" |
| "Get started for free" | "Start the 14-day trial" |

Rule: the label describes the outcome, not the UI gesture.

## Empty States

| Before | After |
|---|---|
| "No items yet! Click + to start your journey 🚀" | "No campaigns yet. [Create your first campaign] — takes about 2 minutes." |
| "Nothing here." | Say what the item IS, why you'd want one, and the one specific action. |

Rule: one specific next action + what the user gets. (Empty-state STATE handling — when to show
it, loading vs empty — is `state-handling`'s domain; this row is about the words.)

## Form Labels

| Before | After |
|---|---|
| Input with placeholder "Email" and no label | A real `<label>`, always. Placeholders disappear on focus and fail accessibility. |
| "Settings" button that saves | "Save settings" — use the verb for the action slot |
| Confirm copy restating the question | "Delete [Item Name]? This can't be undone." |

## Voice checklist (UI text)

- **Specific > general** — "Saves 3 hours/week" not "Saves time"
- **Active > passive** — "We deleted it" not "It was deleted"
- **Short > long** — read it aloud; cut every word that doesn't earn its place
- **No hollow affirmations** — delete "Absolutely!", "Great question!", "I'd be happy to help"
- **No corporate superlatives** — "powerful", "seamless", "next-generation" → a concrete claim
- **No emoji** unless the context is genuinely casual (and even then, mean it)
