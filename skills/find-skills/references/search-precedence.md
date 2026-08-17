
# Corpus Search Precedence

Three corpus lookup paths, ordered by precision (highest first):

1. **Repository routing guidance** — if the current repository has a hand-curated domain-to-skill
   table, check it first. It is higher precision than keyword search because its mappings were
   reviewed explicitly.
2. **A generated skill index** — when the installation exposes a JSON index of skill names and
   frontmatter descriptions, query the `description` field specifically:
   ```bash
   jq -r '.skills[] | select(.description | test("keyword"; "i")) | .title' path/to/skill-index.json
   ```
3. **`grep -rl "keyword" <skill-roots>/*/SKILL.md`** — last resort, whole-file text search.

**Why not just grep the whole corpus first?** A skill's frontmatter `description` is the field
the harness actually uses to decide whether to surface it — a hit there means the skill's own
author considers this its trigger. A hit only in body prose or a code example proves nothing
about intent (a skill mentioning "docker" in a troubleshooting aside isn't a docker skill). Rank
`description`-field hits above body-text hits for exactly this reason.

**If the generated index shows nothing**: don't conclude the skill doesn't exist — indexes can
lag a same-session skill creation. Fall back to step 3 or inspect the active skill roots directly
by name before concluding nothing exists.
