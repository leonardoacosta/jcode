"""Public native daemon/bridge contract acceptance with synthetic loopback inference.

Build local binaries first, then run: python3 tests/test_harness_contracts.py.
Never connects to the installed shared daemon or a live provider.
"""
import hashlib
import http.server
import json
import os
from pathlib import Path
import select
import socket
import subprocess
import tempfile
import threading
import time

root = Path(__file__).resolve().parents[1]
scratch = Path(os.environ.get('JCODE_SCRATCH_DIR', root / 'scratch'))
scratch.mkdir(parents=True, exist_ok=True)
binary = root / 'target/selfdev/jcode'
bridge_binary = root / 'target/selfdev/jcode-harness-api-bridge'
first_started = threading.Event()
release_old = threading.Event()
old_attempted = threading.Event()
requests = []
class Fixture(http.server.BaseHTTPRequestHandler):
    protocol_version = 'HTTP/1.1'
    def log_message(self, *args): pass
    def do_GET(self):
        body = json.dumps({'data': [{'id': 'gpt-5.4', 'object': 'model'}]}).encode()
        self.send_response(200); self.send_header('Content-Type', 'application/json'); self.send_header('Content-Length', str(len(body))); self.end_headers(); self.wfile.write(body)
    def do_POST(self):
        body = self.rfile.read(int(self.headers.get('Content-Length', '0')))
        requests.append(self.path)
        turn = len(requests)
        self.send_response(200); self.send_header('Content-Type', 'text/event-stream'); self.send_header('Connection', 'close'); self.end_headers()
        def event(value):
            self.wfile.write(('data: ' + json.dumps(value) + '\n\n').encode()); self.wfile.flush()
        try:
            event({'type': 'response.created', 'response': {'id': f'resp_fixture_{turn}'}})
            event({'type': 'response.output_text.delta', 'item_id': f'msg_fixture_{turn}', 'output_index': 0, 'content_index': 0, 'delta': 'first-partial' if turn == 1 else 'second-complete'})
            if turn == 1:
                first_started.set()
                release_old.wait(15)
                event({'type': 'response.output_text.delta', 'delta': 'LATE-OLD-GENERATION'})
            event({'type': 'response.completed', 'response': {'id': f'resp_fixture_{turn}', 'status': 'completed', 'output': [], 'usage': {'input_tokens': 1, 'output_tokens': 1}}})
        except (BrokenPipeError, ConnectionResetError): pass
        finally:
            if turn == 1:
                old_attempted.set()
        self.close_connection = True
fixture = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Fixture)
fixture.daemon_threads = True
threading.Thread(target=fixture.serve_forever, daemon=True).start()
with tempfile.TemporaryDirectory(prefix='sdk-cancel-', dir=scratch) as state:
    state = Path(state)
    home = state/'home'; home.mkdir()
    runtime = state/'runtime'; runtime.mkdir()
    native = runtime/'native.sock'; api = runtime/'api.sock'
    (home/'config.toml').write_text('[features]\nmemory=false\nauto_poke=false\ncheck_updates=false\n[autojudge]\nmax_continuations_per_turn=0\n[agents]\nmemory_sidecar_enabled=false\n')
    env = {k:v for k,v in os.environ.items() if k in ['PATH','LANG','LC_ALL','LD_LIBRARY_PATH']}
    env.update(HOME=str(state), JCODE_HOME=str(home), XDG_RUNTIME_DIR=str(runtime), JCODE_RUNTIME_DIR=str(runtime), JCODE_NO_TELEMETRY='1', JCODE_MEMORY_ENABLED='0', OPENAI_API_KEY='synthetic-local-only', JCODE_OPENAI_API_BASE=f'http://127.0.0.1:{fixture.server_port}/v1', JCODE_OPENAI_TRANSPORT='https', JCODE_DISABLE_WEBSOCKET='1', NO_COLOR='1')
    processes = []
    logs = []
    frames = []
    try:
        for name, argv in [('native', [str(binary), '--no-update', '--no-selfdev', '--provider','openai-api','--model','gpt-5.4','--socket',str(native),'serve','--temporary-server','--owner-pid',str(os.getpid()),'--temp-idle-timeout-secs','60']), ('bridge',[str(bridge_binary),str(api),str(native)])]:
            log = open(state/f'{name}.log','wb'); logs.append(log)
            processes.append(subprocess.Popen(argv, cwd=state, env=env, stdout=log, stderr=log))
        deadline=time.monotonic()+15
        while not api.exists() or not native.exists():
            assert all(p.poll() is None for p in processes), '\n'.join(p.read_text() for p in state.glob('*.log'))
            assert time.monotonic()<deadline, 'isolated sockets not ready'
            select.select([],[],[],.01)
        client=socket.socket(socket.AF_UNIX); client.connect(str(api)); client.settimeout(10)
        reader=client.makefile('rb')
        def send(value): client.sendall((json.dumps(value)+'\n').encode())
        def read_until(predicate):
            deadline=time.monotonic()+15
            while True:
                assert time.monotonic()<deadline
                raw=reader.readline()
                assert raw, 'unexpected API EOF'
                frame=json.loads(raw); frames.append(frame)
                if predicate(frame): return frame
        send({'v':1,'id':1,'req':'hello','min_version':1,'max_version':1,'client':'isolated-cancel-acceptance'})
        assert read_until(lambda f:f.get('reply_to')==1)['ev']=='hello_ok'
        send({'v':1,'id':2,'req':'create_session','working_dir':str(state),'system_prompt':'Synthetic public cancellation acceptance. No tools.'})
        attached=read_until(lambda f:f.get('reply_to')==2)
        assert attached['ev']=='attached', attached
        sid=attached['session']['session_id']
        send({'v':1,'id':3,'req':'send_message','session_id':sid,'content':'synthetic first'})
        read_until(lambda f:f.get('ev')=='text_delta' and 'first-partial' in f.get('text',''))
        first_message_id=frames[-1].get('message_id')
        cancel_start=len(frames)
        send({'v':1,'id':4,'req':'cancel','session_id':sid})
        read_until(lambda f:f.get('ev')=='turn_done')
        first_boundary=len(frames)-1
        assert sum(f.get('ev')=='turn_done' for f in frames)==1
        assert any(f.get('ev') in ['turn_stopped','session_status'] for f in frames[cancel_start:first_boundary])
        release_old.set()
        assert old_attempted.wait(5), 'old upstream did not attempt late output'
        send({'v':1,'id':5,'req':'ping'})
        read_until(lambda f:f.get('reply_to')==5)
        assert not any(f.get('ev')=='text_delta' for f in frames[first_boundary+1:]), frames
        send({'v':1,'id':6,'req':'send_message','session_id':sid,'content':'synthetic second'})
        read_until(lambda f:f.get('ev')=='text_delta' and 'second-complete' in f.get('text',''))
        assert frames[-1].get('message_id') != first_message_id
        read_until(lambda f:f.get('ev')=='turn_done')
        assert sum(f.get('ev')=='turn_done' for f in frames)==2, frames
        assert not any('LATE-OLD-GENERATION' in f.get('text','') for f in frames), frames
        assert len(requests)==2, requests
        assert Path(f'/proc/{processes[0].pid}/exe').resolve()==binary.resolve()
        assert Path(f'/proc/{processes[1].pid}/exe').resolve()==bridge_binary.resolve()
        with socket.socket(socket.AF_UNIX) as probe:
            probe.connect(str(native));probe.settimeout(2)
            probe.sendall(b'{"type":"ping","id":99}\n')
            runtime=json.loads(probe.makefile('rb').readline())['server_runtime']
            expected=subprocess.check_output(['git','rev-parse','--short','HEAD'],cwd=root,text=True).strip()
            assert runtime['git_hash']==expected
            assert Path(runtime['binary_path']).resolve()==binary.resolve()
        print('PASS actual native daemon Ping reports serving revision/path matching source and /proc executable')
        print('PASS actual native daemon + actual API bridge: Cancel mid-stream => one TurnDone, old stream suppressed, next execution distinct and completes')
        print('provider=synthetic loopback only; real public socket path exercised; not live provider/durable-stop acceptance')
        print('native_sha256='+hashlib.sha256(binary.read_bytes()).hexdigest())
        print('bridge_sha256='+hashlib.sha256(bridge_binary.read_bytes()).hexdigest())
        reader.close();client.close()
    finally:
        release_old.set()
        (scratch/'sdk-cancel-frames.json').write_text(json.dumps(frames,indent=2))
        for p in reversed(processes):
            p.terminate()
            try:p.wait(timeout=5)
            except subprocess.TimeoutExpired:p.kill();p.wait(timeout=5)
        for log in logs:log.close()
        for log in state.glob('*.log'):
            (scratch/f'sdk-cancel-{log.name}').write_bytes(log.read_bytes())
fixture.shutdown()
