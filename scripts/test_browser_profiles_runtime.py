"""Real isolated CLI acceptance. Usage: python3 scripts/test_browser_profiles_runtime.py BINARY PYTHON.
Uses disposable Chromium data only. Never connects to the shared Jcode daemon.
"""
import functools
import http.server
import json
import os
import pathlib
import subprocess
import sys
import tempfile
import threading

binary = str(pathlib.Path(sys.argv[1]).resolve())
python = str(pathlib.Path(sys.argv[2]).absolute())
scratch = pathlib.Path(os.environ.get('JCODE_SCRATCH_DIR', 'scratch')).resolve()
root = pathlib.Path(tempfile.mkdtemp(prefix='profile-acceptance-', dir=scratch))
site = root / 'site'
site.mkdir()
(site / 'index.html').write_text('<title>Profile fixture</title><p id="identity">Disposable signed-in fixture</p>')
server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), functools.partial(http.server.SimpleHTTPRequestHandler, directory=str(site)))
threading.Thread(target=server.serve_forever, daemon=True).start()
url = f'http://127.0.0.1:{server.server_port}'
home = root / 'home'
(home / 'browser').mkdir(parents=True)
legacy = home / 'browser' / 'sentinel'
legacy.write_text('untouched legacy installation')
env = dict(os.environ, JCODE_HOME=str(home), JCODE_BROWSER_PYTHON=python, JCODE_BROWSER_CHROME='/usr/bin/chromium')
evidence = []

def call(action, session='owner', fail=None, confirm=False, **kwargs):
    command = [binary, '--no-update', '--socket', str(root / 'isolated.sock'), 'browser-profiles',
               json.dumps(dict(action=action, **kwargs)), '--session', session]
    if confirm:
        command.append('--confirm-delete')
    process = subprocess.run(command, env=env, capture_output=True, text=True, timeout=65)
    if fail:
        assert process.returncode != 0 and fail in process.stderr, (action, process.stdout, process.stderr)
        result = {'expected_error': fail}
    else:
        assert process.returncode == 0, (action, process.stdout, process.stderr)
        result = json.loads(process.stdout)
    evidence.append({'action': action, 'session': session, 'result': result})
    print(json.dumps(evidence[-1]), flush=True)
    return result

try:
    call('select', fail='no default')
    call('create', profile='../bad', lifetime='persistent', fail='invalid label')
    a = call('create', profile='alpha', lifetime='persistent')['profile']['id']
    call('create', profile='alpha', lifetime='persistent', fail='already exists')
    call('create', profile='beta', lifetime='persistent')
    assert len(call('list')['profiles']) == 2
    call('default', profile='alpha')
    call('select')
    assert call('evaluate', url=url, script="localStorage.getItem('identity')")['value'] is None
    call('evaluate', url=url, script="localStorage.setItem('identity','alpha'); localStorage.getItem('identity')")
    call('default', profile='beta')
    assert call('inspect')['profile']['label'] == 'alpha'
    call('select', profile='beta', fail='already bound')
    call('delete', profile='alpha', confirm=True, fail='active')
    call('close')
    call('select')
    assert call('evaluate', url=url, script="localStorage.getItem('identity')")['value'] is None
    call('evaluate', url=url, script="localStorage.setItem('identity','beta'); localStorage.getItem('identity')")
    call('close')
    for label in ('alpha', 'beta'):
        assert call('evaluate', profile=label, url=url, script="localStorage.getItem('identity')")['value'] == label
        call('close')
    temporary = call('create', profile='temporary', lifetime='temporary')['profile']['id']
    call('default', profile='temporary', fail='cannot become')
    call('evaluate', profile='temporary', url=url, script='document.title')
    call('close')
    assert not (home / 'browser-profiles' / temporary).exists()
    # Build disposable pre-existing signed-in state using the verified upstream harness.
    external = root / 'external'
    (external / 'Default').mkdir(parents=True)
    call('attach', profile='external', path=str(external))
    call('evaluate', profile='external', url=url, script="document.cookie='signed_in=fixture; max-age=3600; path=/'; document.cookie")
    call('close')
    call('detach', profile='external')
    assert (external / 'Default').is_dir()
    call('attach', profile='signed-in', path=str(external))
    alias = root / 'external-alias'
    alias.symlink_to(external, target_is_directory=True)
    call('attach', profile='alias', path=str(alias), fail='already registered')
    call('select', profile='signed-in')
    assert 'signed_in=fixture' in call('evaluate', url=url, script='document.cookie')['value']
    call('close')
    call('delete', profile='signed-in', confirm=True, fail='never be deleted')
    (external / 'Profile 1').mkdir(exist_ok=True)
    call('attach', profile='sibling', path=str(external), directory='Profile 1')
    assert 'signed_in=fixture' not in call('evaluate', profile='sibling', url=url, script='document.cookie')['value']
    call('close')
    call('detach', profile='sibling')
    lock = external / 'SingletonLock'
    assert not lock.exists() and not lock.is_symlink()
    lock.symlink_to('fixture-host-12345')
    call('evaluate', profile='signed-in', url=url, script='document.title', fail='locked')
    assert lock.is_symlink()
    lock.unlink()  # Synthetic lock owned by this test, never removed by application.
    call('detach', profile='signed-in')
    assert (external / 'Default' / 'Cookies').exists()
    call('inspect', profile='unknown', fail='unknown profile')
    call('delete', profile='alpha', fail='confirmation')
    call('delete', profile='alpha', confirm=True)
    assert not (home / 'browser-profiles' / a).exists()
    assert legacy.read_text() == 'untouched legacy installation'
    print('PASS: isolated Jcode CLI profile persistence, isolation, attachment, cleanup and deletion', flush=True)
finally:
    server.shutdown()
    (root / 'evidence.json').write_text(json.dumps(evidence, indent=2))
    print('Evidence root:', root, flush=True)
