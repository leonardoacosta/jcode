//! Actual daemon/public Comm/native write acceptance. Only inference is synthetic.
//! Run explicitly with --ignored. Never attaches to an existing daemon.
#![cfg(target_os = "linux")]

use std::process::Command;

const ACCEPTANCE: &str = r#"
import hashlib, http.server, json, os, select, socket, subprocess, sys, tempfile, threading, time
from pathlib import Path

root, binary, parent_net = Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3]
assert os.readlink('/proc/self/ns/net') != parent_net, 'refusing shared network namespace'
subprocess.run(['/usr/sbin/ip', 'link', 'set', 'lo', 'up'], check=True)
links = json.loads(subprocess.check_output(['/usr/sbin/ip', '-json', 'link', 'show']))
assert {link['ifname'] for link in links} == {'lo'}, 'refusing external interfaces'
assert not subprocess.check_output(['/usr/sbin/ip', 'route', 'show', 'default']), 'refusing external route'
with socket.socket() as probe:
    probe.settimeout(1)
    try: probe.connect(('192.0.2.1',443))
    except OSError: pass
    else: raise AssertionError('external network unexpectedly reachable')
assert binary.is_file(), 'build a private selfdev binary first'
binary = binary.resolve()
sha = lambda data: hashlib.sha256(data).hexdigest()
source = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip()
source_paths = ['crates/jcode-app-core/src/server/comm_session.rs',
                'crates/jcode-app-core/src/server/client_lightweight_control.rs',
                'crates/jcode-app-core/src/server/live_turn.rs',
                'crates/jcode-app-core/src/server/swarm/claims.rs']
binding = {p: sha((root/p).read_bytes()) for p in source_paths}
started, release = threading.Event(), threading.Event()
requests, faults, frames = [], [], []

class Fixture(http.server.BaseHTTPRequestHandler):
    protocol_version = 'HTTP/1.1'
    def log_message(self, *args): pass
    def do_GET(self):
        if self.path != '/v1/models':
            faults.append('unexpected GET '+self.path)
            self.send_error(400); return
        body = json.dumps({'data':[{'id':'gpt-5.4','object':'model'}]}).encode()
        self.send_response(200); self.send_header('Content-Type','application/json')
        self.send_header('Content-Length',str(len(body))); self.end_headers(); self.wfile.write(body)
    def do_POST(self):
        self.rfile.read(int(self.headers.get('Content-Length','0')))
        requests.append(self.path)
        turn = len(requests)
        if self.path != '/v1/responses' or turn > 2:
            faults.append('unexpected POST '+self.path)
            self.send_error(400); return
        self.send_response(200); self.send_header('Content-Type','text/event-stream')
        self.send_header('Connection','close'); self.end_headers()
        def event(value):
            self.wfile.write(('data: '+json.dumps(value)+'\n\n').encode()); self.wfile.flush()
        try:
            event({'type':'response.created','response':{'id':f'resp_{turn}'}})
            output = []
            if turn == 1:
                event({'type':'response.output_text.delta','item_id':'busy','output_index':0,'content_index':0,'delta':'fixture-provider-busy'})
                started.set()
                if not release.wait(20):
                    faults.append('provider release deadline'); return
                arguments = json.dumps({'file_path':str(marker),'content':'POST-STOP-NATIVE-WRITE\n','intent':'Isolated stop regression marker'})
                item = {'type':'function_call','id':'write_item','call_id':'write_call','name':'write','arguments':''}
                event({'type':'response.output_item.added','output_index':1,'item':item})
                event({'type':'response.function_call_arguments.done','item_id':'write_item','arguments':arguments})
                item = dict(item, arguments=arguments)
                event({'type':'response.output_item.done','output_index':1,'item':item})
                output = [item]
            else:
                event({'type':'response.output_text.delta','item_id':'final','output_index':0,'content_index':0,'delta':'fixture-complete'})
            event({'type':'response.completed','response':{'id':f'resp_{turn}','status':'completed','output':output,'usage':{'input_tokens':1,'output_tokens':1}}})
        except (BrokenPipeError, ConnectionResetError): pass
        finally: self.close_connection = True

# No socket/state arguments: all control targets are minted under this owned root.
with tempfile.TemporaryDirectory(prefix='jc-stop-') as directory:
    state = Path(directory).resolve()
    home, runtime, install = state/'home', state/'runtime', state/'install'
    for path in [home, runtime, install]: path.mkdir(mode=0o700)
    native, marker = runtime/'native.sock', state/'post-stop.txt'
    def owned(path):
        assert path.parent.resolve() == runtime and not path.is_symlink(), 'refusing shared socket'
    owned(native)
    assert not native.exists(), 'refusing existing socket'
    (home/'config.toml').write_text('[features]\nmemory=false\nauto_poke=false\ncheck_updates=false\n[autojudge]\nmax_continuations_per_turn=0\n[agents]\nmemory_sidecar_enabled=false\n')
    fixture = http.server.ThreadingHTTPServer(('127.0.0.1',0),Fixture)
    fixture.daemon_threads = True
    thread = threading.Thread(target=fixture.serve_forever,daemon=True); thread.start()
    endpoint = f'http://127.0.0.1:{fixture.server_port}/v1'
    assert endpoint.startswith('http://127.0.0.1:') and fixture.server_address[0] == '127.0.0.1'
    # No inherited credentials, proxies, MCP config, session or shared-runtime env.
    env = {'PATH':'/usr/bin:/bin','LANG':'C.UTF-8','HOME':str(state),
           'XDG_CONFIG_HOME':str(state/'config'),'XDG_DATA_HOME':str(state/'data'),
           'XDG_CACHE_HOME':str(state/'cache'),'XDG_RUNTIME_DIR':str(runtime),
           'JCODE_HOME':str(home),'JCODE_RUNTIME_DIR':str(runtime),'JCODE_INSTALL_DIR':str(install),
           'JCODE_NO_TELEMETRY':'1','JCODE_MEMORY_ENABLED':'0','JCODE_MEMORY_SIDECAR_ENABLED':'0',
           'OPENAI_API_KEY':'synthetic-local-only','JCODE_OPENAI_API_BASE':endpoint,
           'JCODE_OPENAI_TRANSPORT':'https','JCODE_DISABLE_WEBSOCKET':'1','NO_COLOR':'1'}
    process, client, reader = None, None, None
    try:
        with open(state/'daemon.log','wb') as log:
            argv = [str(binary),'--no-update','--no-selfdev','--provider','openai-api','--model','gpt-5.4',
                    '--socket',str(native),'serve','--temporary-server','--owner-pid',str(os.getpid()),
                    '--temp-idle-timeout-secs','60']
            process = subprocess.Popen(argv,cwd=state,env=env,stdin=subprocess.DEVNULL,stdout=log,stderr=log)
        deadline = time.monotonic()+15
        while not native.exists():
            assert process.poll() is None, 'daemon exited before socket readiness'
            assert time.monotonic()<deadline, 'private socket readiness deadline'
            select.select([],[],[],.01)
        assert Path(f'/proc/{process.pid}/exe').resolve() == binary, 'wrong serving executable'
        def connect():
            owned(native)
            sock = socket.socket(socket.AF_UNIX); sock.settimeout(10); sock.connect(str(native))
            return sock
        def send(sock, value): sock.sendall((json.dumps(value)+'\n').encode())
        def read_until(reader, predicate):
            deadline = time.monotonic()+10
            while True:
                assert time.monotonic()<deadline, 'public response deadline'
                raw = reader.readline(); assert raw, 'public socket EOF'
                frame = json.loads(raw); frames.append(frame)
                if predicate(frame): return frame
        def request(value, terminal):
            with connect() as sock:
                send(sock,value)
                with sock.makefile('rb') as reader:
                    frame = read_until(reader,lambda f:f.get('id')==value['id'] and f.get('type') in terminal+['error'])
                assert frame['type'] != 'error', frame
                return frame
        pong = request({'type':'ping','id':1},['pong'])
        identity = pong['server_runtime']
        assert Path(identity['binary_path']).resolve() == binary, identity
        version = subprocess.check_output([str(binary),'--version'],env=env,cwd=state,text=True).strip()
        assert identity['git_hash'] in version, (identity,version)
        assert source.startswith(identity['git_hash']), 'refusing binary/source revision mismatch'
        assert identity['git_hash'], 'refusing missing serving revision'
        client = connect(); reader = client.makefile('rb')
        send(client,{'type':'subscribe','id':2,'working_dir':str(state),'system_prompt':'Isolated synthetic workload. Execute only the supplied native write.'})
        session = read_until(reader,lambda f:f.get('type')=='session')['session_id']
        spawned = request({'type':'comm_spawn','id':3,'session_id':session,'working_dir':str(state),
                           'initial_message':'Run the isolated marker workload.','spawn_mode':'headless',
                           'model':'inherit','label':'isolated-stop-marker'},['comm_spawn_response'])
        worker = spawned['new_session_id']
        assert started.wait(10), 'worker never reached deterministic busy provider gate'
        before = request({'type':'comm_status','id':4,'session_id':session,'target_session':worker},['comm_status_response'])
        assert before['snapshot']['activity']['is_processing'], 'worker was not busy at stop boundary'
        assert not marker.exists(), 'marker predated stop'
        stop = request({'type':'comm_stop','id':5,'session_id':session,'target_session':worker},['done'])
        stop_at = time.monotonic()
        assert not marker.exists(), 'marker predated stop acknowledgement'
        release.set()
        deadline = time.monotonic()+5
        while not marker.exists() and time.monotonic()<deadline:
            select.select([],[],[],.01)
        exists = marker.exists()
        elapsed = time.monotonic()-stop_at
        health = request({'type':'ping','id':6},['pong'])
        assert health['server_runtime'] == identity, 'serving daemon changed during acceptance'
        assert not faults, faults
        result = {'source_head':source,'source_sha256':binding,'binary':str(binary),
                  'binary_version':version,'binary_sha256':sha(binary.read_bytes()),'server_runtime':identity,
                  'worker':worker,'busy_snapshot':before,'stop_reply':stop,
                  'marker_before_stop':False,'marker_after_stop':exists,
                  'marker_sha256':sha(marker.read_bytes()) if exists else None,
                  'observed_after_stop_secs':round(elapsed,3),'provider_requests':requests,
                  'generation':stop.get('generation'),'inference':'synthetic loopback Responses fixture',
                  'deadlines_secs':{'startup':15,'public_frame':10,'busy_gate':10,'provider_release':20,'post_stop':5},
                  'isolation':{'network':'new user/network namespace, loopback interface only','home_runtime_install':'owned temporary directory','debug_control':False}}
        print(json.dumps(result,sort_keys=True),flush=True)
        assert not exists, 'BASELINE FAILURE: public CommStop returned Done while busy worker later executed native write'
    finally:
        release.set()
        if reader is not None: reader.close()
        if client is not None: client.close()
        if process is not None:
            process.terminate()
            try: process.wait(timeout=5)
            except subprocess.TimeoutExpired: process.kill(); process.wait(timeout=5)
        fixture.shutdown(); fixture.server_close(); thread.join(timeout=2)
        if sys.exc_info()[0] is not None:
            log_path = state/'daemon.log'
            print('owned daemon log:\n'+(log_path.read_text(errors='replace') if log_path.exists() else '(daemon not started)'),file=sys.stderr)
        print('cleanup: owned daemon reaped, fixture server closed, temporary state removed on context exit',flush=True)
"#;

#[test]
fn refuses_shared_network_before_starting_daemon() {
    let namespace = std::fs::read_link("/proc/self/ns/net").expect("Linux network namespace");
    let output = Command::new("/usr/bin/python3")
        .args([
            "-c",
            ACCEPTANCE,
            "/nonexistent-source",
            "/nonexistent-binary",
        ])
        .arg(namespace)
        .env_clear()
        .output()
        .expect("run shared-network refusal check");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("refusing shared network namespace"));
}

#[test]
#[ignore = "actual private binary + Linux user/network namespaces; see change evidence.md"]
fn public_stop_prevents_post_stop_native_write() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let binary = std::env::var_os("JCODE_WORKER_STOP_BINARY")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join("target/selfdev/jcode"));
    let namespace = std::fs::read_link("/proc/self/ns/net").expect("Linux network namespace");
    let output = Command::new("/usr/bin/timeout")
        .args(["--kill-after=5", "90", "/usr/bin/unshare"])
        .args([
            "--user",
            "--map-root-user",
            "--net",
            "/usr/bin/python3",
            "-c",
            ACCEPTANCE,
        ])
        .arg(root)
        .arg(binary)
        .arg(namespace)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("TMPDIR", root.join("scratch"))
        .output()
        .expect("launch isolated actual-binary acceptance");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    assert!(
        output.status.success(),
        "private worker-stop acceptance failed ({}):\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}
