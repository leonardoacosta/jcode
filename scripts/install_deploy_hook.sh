#!/usr/bin/env bash
# Install the post-commit deploy hook for this clone: after any commit that
# touches buildable code, scripts/post_commit_deploy.sh rebuilds + installs
# the release in the background and reloads the shared server.
# Idempotent; safe to re-run after pulling.
set -euo pipefail

config_only=false
case "${1:-}" in
    "") ;;
    --config-only) config_only=true ;;
    *) echo "Unknown argument: $1" >&2; exit 2 ;;
esac

jcode_home="${JCODE_HOME:-$HOME/.jcode}"
config_path="$jcode_home/config.toml"
if [ -f "$config_path" ]; then
    python3 - "$config_path" <<'PY'
import os
import pathlib
import re
import sys
import tempfile
import tomllib

path = pathlib.Path(sys.argv[1])
try:
    original = path.read_text()
    parsed = tomllib.loads(original)
except (OSError, UnicodeError, tomllib.TOMLDecodeError) as error:
    raise SystemExit(f"config update refused for {path}: {error}")

lines = original.splitlines(keepends=True)
table = None
changed = False
found = False
display_insert = None
key_pattern = re.compile(
    r"^(\s*auto_client_reload\s*=\s*)(true|false)(\s*(?:#.*)?)(\r?\n)?$"
)
table_pattern = re.compile(r"^\s*\[([^]]+)]\s*(?:#.*)?(?:\r?\n)?$")
for index, line in enumerate(lines):
    table_match = table_pattern.match(line)
    if table_match:
        if table == "display" and display_insert is None:
            display_insert = index
        table = table_match.group(1).strip()
        if table == "display":
            display_insert = index + 1
        continue
    if table != "display":
        continue
    display_insert = index + 1
    key_match = key_pattern.match(line)
    if not key_match:
        continue
    found = True
    if key_match.group(2) == "false":
        lines[index] = (
            key_match.group(1)
            + "true"
            + key_match.group(3)
            + (key_match.group(4) or "")
        )
        changed = True
    break

if not found and display_insert is not None:
    if display_insert > 0 and not lines[display_insert - 1].endswith(("\n", "\r")):
        lines[display_insert - 1] += "\n"
    lines.insert(display_insert, "auto_client_reload = true\n")
    changed = True

if changed:
    candidate = "".join(lines)
    try:
        candidate_value = tomllib.loads(candidate)
    except tomllib.TOMLDecodeError as error:
        raise SystemExit(f"config update produced invalid TOML for {path}: {error}")
    if candidate_value.get("display", {}).get("auto_client_reload") is not True:
        raise SystemExit(f"config update did not enable display.auto_client_reload in {path}")

    backup = path.with_suffix(".bak")
    try:
        backup.unlink(missing_ok=True)
        os.link(path, backup)
        descriptor, temp_name = tempfile.mkstemp(prefix=".config.", dir=path.parent)
        try:
            os.fchmod(descriptor, 0o600)
            with os.fdopen(descriptor, "w", encoding="utf-8", newline="") as temp_file:
                temp_file.write(candidate)
                temp_file.flush()
                os.fsync(temp_file.fileno())
            os.replace(temp_name, path)
        except BaseException:
            try:
                os.unlink(temp_name)
            except OSError:
                pass
            raise
        os.chmod(path, 0o600)
        os.chmod(backup, 0o600)
        directory = os.open(path.parent, os.O_RDONLY)
        try:
            os.fsync(directory)
        finally:
            os.close(directory)
    except OSError as error:
        raise SystemExit(f"config update failed for {path}: {error}")
PY
fi

if [ "$config_only" = true ]; then
    exit 0
fi

repo_root="$(git rev-parse --show-toplevel)"
hooks_dir="$repo_root/.git/hooks"
hook="$hooks_dir/post-commit"

# Respect core.hooksPath if the user configured one.
if configured="$(git config --get core.hooksPath)" && [ -n "$configured" ]; then
    hooks_dir="$repo_root/$configured"
    hook="$hooks_dir/post-commit"
fi

mkdir -p "$hooks_dir"

if [ -f "$hook" ] && ! grep -q "post_commit_deploy.sh" "$hook"; then
    echo "Existing post-commit hook found at $hook"
    echo "Add this line to it manually:"
    echo "  \"$repo_root/scripts/post_commit_deploy.sh\" || true"
    exit 1
fi

cat > "$hook" <<EOF
#!/usr/bin/env bash
# jcode deploy hook: rebuild + install + reload on commits touching buildable
# code. Managed by scripts/install_deploy_hook.sh; logic lives in
# scripts/post_commit_deploy.sh.
exec "$repo_root/scripts/post_commit_deploy.sh"
EOF
chmod +x "$hook" "$repo_root/scripts/post_commit_deploy.sh"
echo "Installed post-commit deploy hook -> $hook"
