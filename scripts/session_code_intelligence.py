#!/usr/bin/env python3
"""Opt-in session-start action: inspect local MCPs, build Graft, append guidance.

Configure [hooks].session_start to include `python3 /absolute/path/to/this.py`.
No packages, MCP entries, agent hooks, or global Graft wiring are installed.
"""
import fcntl
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

GUIDANCE = '''\n\n<!-- jcode:graft-startup -->
## Graft code navigation

Use `graft map`, `graft ask`, and `graft callers` for indexed code navigation.
Verify results against current source. Use ordinary search for unindexed files.
Refresh with `graft build --no-gitignore --no-ignore`. No Graft hooks are installed
by Jcode's startup action.
<!-- /jcode:graft-startup -->
'''


def mcp_status(root):
    servers = {}
    for relative in ('.jcode/mcp.json', '.mcp.json', '.claude/mcp.json'):
        path = root / relative
        if path.exists():
            data = json.loads(path.read_text())
            entries = data.get('mcpServers', {})
            if not isinstance(entries, dict):
                raise ValueError(f'{relative}: mcpServers must be an object')
            servers.update(entries)
    return {name: any(
        name in key.lower().replace('_', '-') and isinstance(value, dict)
        and not value.get('disabled', False)
        and bool(value.get('command') or value.get('url'))
        for key, value in servers.items()) for name in ('agent-lsp', 'ast-grep')}


def setup(root, graft):
    env = dict(os.environ, DO_NOT_TRACK='1')
    subprocess.run([graft, 'build', '--no-gitignore', '--no-ignore', str(root)],
                   cwd=root, env=env, check=True, timeout=120,
                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    # ponytail: append once, never regenerate user instructions or Graft wiring.
    path = root / 'AGENTS.md'
    flags = os.O_RDWR | os.O_CREAT | os.O_APPEND | os.O_NOFOLLOW
    with os.fdopen(os.open(path, flags, 0o644), 'r+') as handle:
        fcntl.flock(handle, fcntl.LOCK_EX)
        text = handle.read()
        if '<!-- jcode:graft-startup -->' not in text and '<!-- graft' not in text:
            handle.write(GUIDANCE)


def main():
    cwd = os.environ.get('JCODE_HOOK_CWD')
    if not cwd:
        return  # Never infer the daemon's working directory.
    result = subprocess.run(['git', '-C', cwd, 'rev-parse', '--show-toplevel'],
                            capture_output=True, text=True, timeout=5)
    if result.returncode:
        return
    root = Path(result.stdout.strip()).resolve()
    state = Path.home() / '.jcode' / 'cache' / 'code-intelligence'
    state.mkdir(parents=True, exist_ok=True, mode=0o700)
    key = hashlib.sha256(os.fsencode(root)).hexdigest()
    with (state / (key + '.lock')).open('w') as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            return
        status = {'repository': str(root)}
        try:
            status['local_mcp'] = mcp_status(root)
            graft = shutil.which('graft')
            if graft:
                setup(root, graft)
                status['graft'] = 'built; AGENTS.md guidance present'
            else:
                status['graft'] = 'not installed; skipped'
        except (OSError, ValueError, subprocess.SubprocessError) as error:
            status['error'] = str(error)
        with tempfile.NamedTemporaryFile(mode='w', dir=state, delete=False) as out:
            json.dump(status, out, indent=2)
            temporary = out.name
        os.replace(temporary, state / (key + '.json'))


if __name__ == '__main__':
    main()
