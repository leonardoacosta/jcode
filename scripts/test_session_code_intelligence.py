import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import session_code_intelligence as action


class StartupTest(unittest.TestCase):
    def test_mcp_and_append_only_setup(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / '.mcp.json').write_text(json.dumps({'mcpServers': {
                'agent-lsp': {'command': 'agent-lsp'},
                'ast-grep': {'url': 'https://example.com/mcp', 'disabled': True}}}))
            self.assertEqual(action.mcp_status(root), {'agent-lsp': True, 'ast-grep': False})
            agents = root / 'AGENTS.md'
            agents.write_text('Keep my rules.\n')
            with patch.object(action.subprocess, 'run') as run:
                action.setup(root, '/usr/bin/graft')
                action.setup(root, '/usr/bin/graft')
                self.assertEqual(run.call_args.args[0], ['/usr/bin/graft', 'build',
                    '--no-gitignore', '--no-ignore', str(root)])
            self.assertTrue(agents.read_text().startswith('Keep my rules.\n'))
            self.assertEqual(agents.read_text().count('<!-- jcode:graft-startup -->'), 1)
            self.assertEqual(sorted(p.name for p in root.iterdir()), ['.mcp.json', 'AGENTS.md'])

    def test_no_instructions_after_failed_build(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with patch.object(action.subprocess, 'run', side_effect=OSError('missing')):
                with self.assertRaises(OSError):
                    action.setup(root, 'graft')
            self.assertFalse((root / 'AGENTS.md').exists())

    def test_symlink_instructions_refused(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = root / 'original'
            target.write_text('untouched')
            (root / 'AGENTS.md').symlink_to(target)
            with patch.object(action.subprocess, 'run'):
                with self.assertRaises(OSError):
                    action.setup(root, 'graft')
            self.assertEqual(target.read_text(), 'untouched')


if __name__ == '__main__':
    unittest.main()
