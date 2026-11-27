"""Read-only probe: per-source readers against a profile home with no ledgers."""
import os
import sys
from pathlib import Path

sys.path.insert(0, "/home/nyaptor/worktrees/github.com-NousResearch-hermes-agent/jcode/async-supervision")

PROBE_HOME = Path("/home/nyaptor/.jcode/review-scratch/empty-home")
os.environ["HERMES_HOME"] = str(PROBE_HOME)

from hermes_cli.web_routers import work  # noqa: E402

home = PROBE_HOME
print("cron/executions.db exists:", (home / "cron" / "executions.db").exists())
print("state.db exists:", (home / "state.db").exists())
print("processes.json exists:", (home / "processes.json").exists())

for name, reader in (
    ("cron", work._cron_rows),
    ("delegation", work._delegation_rows),
    ("process", work._process_rows),
):
    try:
        print(name, "OK rows=", len(list(reader(home))))
    except Exception as exc:  # probe only: report the exception class
        print(name, "RAISES", type(exc).__name__, str(exc)[:160])

print("cron/executions.db exists after:", (home / "cron" / "executions.db").exists())
