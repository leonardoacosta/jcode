# Repository Agent Instructions

## Repository ownership boundary

The immediate working directory, `/home/nyaptor/dev/jcode`, is the only repository agents may modify for work started here.

- Make edits, commits, branch changes, pushes, generated artifacts, and configuration changes only in this repository.
- Treat `/home/nyaptor/dev/jcode/source/jcode` as a read-only reference checkout. Never edit, format, commit, merge, rebase, push, switch branches, change remotes, alter its GitHub settings, or run commands that write generated files there.
- Do not modify any other nested or sibling repository from a session rooted here.
- The branch shown for `/home/nyaptor/dev/jcode` is the authoritative branch for this session. Do not substitute the branch of a nested repository.
- Read-only inspection of nested repositories is allowed when needed for context.
- If a request appears to require changing the source checkout or another repository, stop before making changes and report that it is outside this repository's ownership boundary.

This boundary overrides any workflow that would otherwise follow implementation code into `source/jcode` or another checkout.
