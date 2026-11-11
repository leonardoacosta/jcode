import hashlib, json, os, pathlib, select, socket, subprocess, tempfile, time
root = pathlib.Path(__file__).resolve().parents[1]
scratch = pathlib.Path(os.environ.get('JCODE_SCRATCH_DIR', root/'scratch'))
scratch.mkdir(parents=True, exist_ok=True)
run = scratch/'reload-identity-acceptance'
run.mkdir(exist_ok=True)
binary = root/'target/selfdev/jcode'
real_home = pathlib.Path.home()
protected = [real_home/'.local/bin/jcode', root/'builds/current/jcode', root/'builds/shared-server/jcode']
baseline = {str(p): os.readlink(p) if p.is_symlink() else None for p in protected}
with tempfile.TemporaryDirectory(prefix='reload-success-', dir=scratch) as temp:
    state = pathlib.Path(temp); home = state/'home'; home.mkdir(); runtime = state/'runtime'; runtime.mkdir()
    native = runtime/'native.sock'; debug = runtime/'native-debug.sock'
    (home/'config.toml').write_text('[features]\nmemory=false\nauto_poke=false\ncheck_updates=false\n[ambient]\nenabled=false\n[autojudge]\nmax_continuations_per_turn=0\n[agents]\nmemory_sidecar_enabled=false\n[display]\ndebug_socket=true\n')
    env = {k:v for k,v in os.environ.items() if k in ['PATH', 'LANG', 'LC_ALL', 'LD_LIBRARY_PATH']}
    env.update(HOME=str(state), JCODE_HOME=str(home), JCODE_RUNTIME_DIR=str(runtime), XDG_RUNTIME_DIR=str(runtime), JCODE_REPO_DIR=str(root), JCODE_DEBUG_CONTROL='1', JCODE_NO_TELEMETRY='1', JCODE_MEMORY_ENABLED='0', JCODE_NO_UPDATE='1', JCODE_REMOTE_CARGO='0', RUSTUP_HOME=str(real_home/'.rustup'), CARGO_HOME=str(real_home/'.cargo'), OPENAI_API_KEY='synthetic-local-only', JCODE_OPENAI_API_BASE='http://127.0.0.1:1/v1', JCODE_DISABLE_WEBSOCKET='1', JCODE_OPENAI_TRANSPORT='https', NO_COLOR='1', TERM='xterm-256color')
    log = open(state/'daemon.log', 'wb'); processes = []; receipts = []
    proc = subprocess.Popen([str(binary), '--no-update', '--no-selfdev', '--provider', 'openai-api', '--model', 'gpt-5.4', '--socket', str(native), 'serve', '--temporary-server', '--owner-pid', str(os.getpid()), '--temp-idle-timeout-secs', '300'], cwd=state, env=env, stdout=log, stderr=log)
    processes.append(proc)
    def request(path, value, timeout=10):
        with socket.socket(socket.AF_UNIX) as client:
            client.settimeout(timeout); client.connect(str(path)); client.sendall((json.dumps(value)+'\n').encode())
            reader = client.makefile('rb')
            while True:
                raw = reader.readline()
                if not raw: raise EOFError('private daemon closed connection during reload')
                frame = json.loads(raw)
                if frame.get('id') == value['id']:
                    receipts.append(frame); return frame
    def wait_ready(timeout=20):
        deadline = time.monotonic()+timeout
        while time.monotonic() < deadline:
            assert proc.poll() is None, (state/'daemon.log').read_text(errors='replace')
            try: return request(native, {'type':'ping', 'id':77}, 1)
            except (OSError, EOFError): select.select([], [], [], .1)
        raise TimeoutError('private native socket not ready')
    try:
        before = wait_ready()
        created = request(debug, {'type':'debug_command', 'id':1, 'command':'server:create_session:selfdev:'+str(root)})
        assert created['ok'], created
        sid = json.loads(created['output'])['session_id']; print('private_session='+sid, flush=True)
        # Supported self-dev build writes exact source metadata, then publishes only under isolated JCODE_HOME.
        builder = subprocess.Popen([str(binary), '--no-update', '--socket', str(native), '--resume', sid, 'self-dev', '--build'], cwd=root, env=env, stdout=open(state/'build.log','wb'), stderr=subprocess.STDOUT)
        processes.append(builder)
        try: builder.wait(timeout=180)
        except subprocess.TimeoutExpired: builder.terminate(); builder.wait(timeout=10)
        text = (state/'build.log').read_text(errors='replace')
        assert 'Build complete' in text, text[-10000:]
        assert (home/'builds/current/jcode').exists(), text[-10000:]
        print('PASS supported isolated self-dev build/publication, source sidecar refreshed', flush=True)
        old_exe = pathlib.Path(f'/proc/{proc.pid}/exe').resolve(); old_socket_inode = native.stat().st_ino
        try:
            response = request(debug, {'type':'debug_command', 'id':2, 'session_id':sid, 'command':'server:tool:selfdev '+json.dumps({'action':'reload', 'context':'Private isolated reload identity acceptance. No model continuation.'})}, 30)
            print('reload_response='+json.dumps(response), flush=True)
            assert response.get('ok'), response
        except (OSError, EOFError) as error:
            print('reload_connection_closed='+type(error).__name__, flush=True)
        deadline = time.monotonic()+35
        while time.monotonic()<deadline:
            assert proc.poll() is None, 'private daemon exited during replacement'
            try:
                actual_exe = pathlib.Path(f'/proc/{proc.pid}/exe').resolve()
                if actual_exe != old_exe and native.stat().st_ino != old_socket_inode:
                    after = wait_ready(); break
            except OSError: pass
            select.select([], [], [], .1)
        else: raise AssertionError('No actual private replacement process/socket observed')
        expected = subprocess.check_output(['git','rev-parse','--short','HEAD'],cwd=root,text=True).strip()
        assert after['server_runtime']['git_hash']==expected, after
        assert pathlib.Path(after['server_runtime']['binary_path']).resolve()==actual_exe, after
        assert actual_exe.is_relative_to(home), actual_exe
        manifest_path=home/'builds/manifest.json'; pending=json.loads(manifest_path.read_text())['pending_activation']
        assert pending['session_id']==sid and pending['new_version'].startswith(expected), pending
        resumed = subprocess.run([str(binary), '--no-update', '--socket', str(native), '--resume', sid, 'self-dev'], cwd=root, env=env, capture_output=True, text=True, timeout=20)
        (state/'resume.log').write_text(resumed.stdout+resumed.stderr)
        manifest=json.loads(manifest_path.read_text())
        assert manifest.get('pending_activation') is None, manifest
        assert manifest['canary_status']=='passed' and manifest['canary_session']==sid, manifest
        assert manifest['canary']==pending['new_version'], manifest
        logs='\n'.join(p.read_text(errors='replace') for p in (home/'logs').glob('*.log'))
        assert 'Verified pending self-dev activation for build '+pending['new_version']+', serving revision '+expected in logs, logs[-10000:]
        assert {str(p):os.readlink(p) if p.is_symlink() else None for p in protected}==baseline
        print('PASS actual private process exec + replaced socket + runtime identity + matching resume activation marks passed and logs verified revision', flush=True)
        print('Shared daemon/channels/launcher unchanged; synthetic credentials only, no provider completion request.', flush=True)
        print('binary_sha256='+hashlib.sha256(binary.read_bytes()).hexdigest(), flush=True)
        print('runtime='+json.dumps(after['server_runtime']), flush=True)
    finally:
        for child in reversed(processes):
            if child.poll() is None:
                child.terminate()
                try: child.wait(timeout=8)
                except subprocess.TimeoutExpired: child.kill(); child.wait(timeout=5)
        log.close()
        for path in state.glob('*.log'): (run/('task-003-'+path.name)).write_bytes(path.read_bytes())
        for path in (home/'logs').glob('*.log'): (run/('task-003-runtime-'+path.name)).write_bytes(path.read_bytes())
        (run/'frames.json').write_text(json.dumps(receipts, indent=2))
        assert {str(p):os.readlink(p) if p.is_symlink() else None for p in protected}==baseline
