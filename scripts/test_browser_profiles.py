"""Run with python3 scripts/test_browser_profiles.py. No browser or network required."""
import importlib.util
import pathlib
import tempfile
import unittest
import subprocess
import os
import json
import fcntl

spec = importlib.util.spec_from_file_location('profiles', pathlib.Path(__file__).with_name('browser_profiles.py'))
profiles = importlib.util.module_from_spec(spec)

class ProfilesTest(unittest.TestCase):
    def test_create_is_explicit_and_persistent(self):
        spec.loader.exec_module(profiles)
        with tempfile.TemporaryDirectory() as root:
            store = profiles.Store(pathlib.Path(root) / 'profiles', 'owner')
            with self.assertRaisesRegex(ValueError, 'default'):
                store.dispatch({'action': 'select'})
            created = store.dispatch({'action': 'create', 'profile': 'alpha', 'lifetime': 'persistent'})
            self.assertEqual(created['profile']['label'], 'alpha')
            self.assertEqual(store.dispatch({'action': 'list'})['profiles'][0]['id'], created['profile']['id'])
            with self.assertRaisesRegex(ValueError, 'exists'):
                store.dispatch({'action': 'create', 'profile': 'alpha', 'lifetime': 'persistent'})
            with self.assertRaisesRegex(ValueError, 'label'):
                store.dispatch({'action': 'create', 'profile': '../escape', 'lifetime': 'persistent'})

    def test_attachment_deletion_and_temporary_cleanup(self):
        spec.loader.exec_module(profiles)
        with tempfile.TemporaryDirectory() as root:
            root = pathlib.Path(root)
            store = profiles.Store(root / 'profiles', 'owner')
            external = root / 'external'
            (external / 'Default').mkdir(parents=True)
            sentinel = external / 'Default' / 'Cookies'
            sentinel.write_text('fixture')
            store.dispatch({'action': 'attach', 'profile': 'external', 'path': str(external)})
            with self.assertRaisesRegex(ValueError, 'registered'):
                store.dispatch({'action': 'attach', 'profile': 'alias', 'path': str(external)})
            (external / 'Profile 1').mkdir()
            store.dispatch({'action': 'attach', 'profile': 'second', 'path': str(external), 'directory': 'Profile 1'})
            with store.locked():
                first_profile = store.find('external')
                second_profile = store.find('second')
                self.assertEqual(store.lease_path(first_profile), store.lease_path(second_profile))
                with open(store.lease_path(first_profile), 'w') as lease:
                    fcntl.flock(lease, fcntl.LOCK_EX)
                    with self.assertRaisesRegex(ValueError, 'active'):
                        store.require_inactive(second_profile)
            store.dispatch({'action': 'detach', 'profile': 'second'})
            store.dispatch({'action': 'detach', 'profile': 'external'})
            self.assertEqual(sentinel.read_text(), 'fixture')
            temporary = store.dispatch({'action': 'create', 'profile': 'temp', 'lifetime': 'temporary'})
            directory = root / 'profiles' / temporary['profile']['id']
            store.dispatch({'action': 'select', 'profile': 'temp'})
            store.dispatch({'action': 'close'})
            self.assertFalse(directory.exists())
            store.dispatch({'action': 'create', 'profile': 'saved', 'lifetime': 'persistent'})
            with self.assertRaisesRegex(ValueError, 'confirmation'):
                store.dispatch({'action': 'delete', 'profile': 'saved'})
            store.dispatch({'action': 'select', 'profile': 'saved'})
            with self.assertRaisesRegex(ValueError, 'active'):
                store.dispatch({'action': 'delete', 'profile': 'saved', 'confirmed': True}, user=True)
            store.dispatch({'action': 'close'})
            store.dispatch({'action': 'delete', 'profile': 'saved', 'confirmed': True}, user=True)
            self.assertEqual(store.dispatch({'action': 'list'})['profiles'], [])

    def test_binding_corruption_and_symlink_safety(self):
        spec.loader.exec_module(profiles)
        with tempfile.TemporaryDirectory() as root:
            root = pathlib.Path(root)
            store = profiles.Store(root / 'profiles', 'one')
            a = store.dispatch({'action': 'create', 'profile': 'a', 'lifetime': 'persistent'})['profile']
            store.dispatch({'action': 'create', 'profile': 'b', 'lifetime': 'persistent'})
            store.dispatch({'action': 'default', 'profile': 'a'})
            store.dispatch({'action': 'select'})
            store.dispatch({'action': 'default', 'profile': 'b'})
            self.assertEqual(store.dispatch({'action': 'inspect'})['profile']['label'], 'a')
            other = profiles.Store(root / 'profiles', 'two')
            self.assertEqual(other.dispatch({'action': 'select'})['profile']['label'], 'b')
            store.dispatch({'action': 'close'})
            path = root / 'profiles' / a['id']
            path.rmdir()
            sentinel = root / 'sentinel'
            sentinel.mkdir()
            (sentinel / 'keep').write_text('safe')
            path.symlink_to(sentinel, target_is_directory=True)
            with self.assertRaisesRegex(ValueError, 'unsafe'):
                store.dispatch({'action': 'delete', 'profile': 'a', 'confirmed': True}, user=True)
            self.assertEqual((sentinel / 'keep').read_text(), 'safe')
            metadata = root / 'profiles' / 'profiles.json'
            metadata.write_text('{broken')
            with self.assertRaisesRegex(ValueError, 'corrupt'):
                store.dispatch({'action': 'list'})
            self.assertEqual(metadata.read_text(), '{broken')

    def test_runtime_missing_is_actionable(self):
        spec.loader.exec_module(profiles)
        status = profiles.runtime_status()
        self.assertFalse(status['ready'])
        self.assertIn('setup', status['error'])

    def test_partial_cleanup_persists_completed_removals(self):
        spec.loader.exec_module(profiles)
        with tempfile.TemporaryDirectory() as root:
            root = pathlib.Path(root) / 'profiles'
            store = profiles.Store(root, 'owner')
            first = store.dispatch({'action': 'create', 'profile': 'first', 'lifetime': 'temporary'})['profile']
            second = store.dispatch({'action': 'create', 'profile': 'second', 'lifetime': 'temporary'})['profile']
            (root / second['id'] / 'SingletonLock').symlink_to('fixture-123')
            with self.assertRaisesRegex(ValueError, 'cleanup pending'):
                store.dispatch({'action': 'close'})
            self.assertFalse((root / first['id']).exists())
            self.assertEqual([p['label'] for p in store.dispatch({'action': 'list'})['profiles']], ['second'])

    def test_crash_recovery_retains_locked_data(self):
        spec.loader.exec_module(profiles)
        with tempfile.TemporaryDirectory() as root:
            root = pathlib.Path(root) / 'profiles'
            child = subprocess.Popen(['sleep', '30'])
            previous = os.environ.get('JCODE_BROWSER_OWNER_PID')
            os.environ['JCODE_BROWSER_OWNER_PID'] = str(child.pid)
            try:
                store = profiles.Store(root, 'crash-owner')
                profile = store.dispatch({'action': 'create', 'profile': 'temp', 'lifetime': 'temporary'})['profile']
                self.assertEqual(root.stat().st_mode & 0o777, 0o700)
                self.assertEqual((root / 'profiles.json').stat().st_mode & 0o777, 0o600)
                self.assertEqual(store.dispatch({'action': 'recover'})['recovered'], [])
                child.terminate()
                child.wait()
                lock = root / profile['id'] / 'SingletonLock'
                lock.symlink_to('fixture-host-12345')
                self.assertEqual(store.dispatch({'action': 'recover'})['pending_cleanup'], ['temp'])
                self.assertTrue(lock.is_symlink())
                lock.unlink()  # Test owns the synthetic lock, runtime never removes it.
                self.assertEqual(store.dispatch({'action': 'recover'})['recovered'], ['temp'])
                self.assertFalse((root / profile['id']).exists())
            finally:
                if child.poll() is None:
                    child.terminate()
                    child.wait()
                if previous is None:
                    os.environ.pop('JCODE_BROWSER_OWNER_PID', None)
                else:
                    os.environ['JCODE_BROWSER_OWNER_PID'] = previous

if __name__ == '__main__':
    unittest.main()
