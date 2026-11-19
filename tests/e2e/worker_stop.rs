//! Actual daemon/public Comm/native write acceptance. Only inference is synthetic.
//! Run explicitly with --ignored. Never attaches to an existing daemon.
#![cfg(target_os = "linux")]

use std::process::Command;

const ACCEPTANCE: &str = r#"
import hashlib, http.server, json, os, select, socket, subprocess, sys, tempfile, threading, time
from pathlib import Path

root, binary, parent_net = Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3]
scenario = sys.argv[4] if len(sys.argv) > 4 else 'write'
BUSY = ('bash','handoff','stalewait','reload_drain','reload_settled','restart','alias','resume','churn','restart_settled','restart_drain','stopwake')
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
tool_started = tool_marker = None
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
        if self.path != '/v1/responses' or turn > (6 if scenario in ('handoff','reload_settled') else 4 if scenario in ('resume','restart_settled','churn','stalewait') else 2):
            faults.append('unexpected POST '+self.path)
            self.send_error(400); return
        self.send_response(200); self.send_header('Content-Type','text/event-stream')
        self.send_header('Connection','close'); self.end_headers()
        def event(value):
            self.wfile.write(('data: '+json.dumps(value)+'\n\n').encode()); self.wfile.flush()
        try:
            event({'type':'response.created','response':{'id':f'resp_{turn}'}})
            output = []
            if turn == 1 and scenario in BUSY:
                started.set()
                pause = 60 if scenario in ('restart','restart_drain','reload_drain') else 40 if scenario == 'handoff' else 6
                command = f"echo running > '{tool_started}'; sleep {pause}; echo finished > '{tool_marker}'"
                arguments = json.dumps({'command':command,'intent':'Isolated native tool busy stop regression'})
                item = {'type':'function_call','id':'bash_item','call_id':'bash_call','name':'bash','arguments':''}
                event({'type':'response.output_item.added','output_index':1,'item':item})
                event({'type':'response.function_call_arguments.done','item_id':'bash_item','arguments':arguments})
                item = dict(item, arguments=arguments)
                event({'type':'response.output_item.done','output_index':1,'item':item})
                output = [item]
            elif turn == 1:
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
    tool_started, tool_marker = state/'tool-started.txt', state/'tool-finished.txt'
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
        def read_until(reader, predicate, limit=10):
            deadline = time.monotonic()+limit
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
        def stop_req(requester, target, rid, force=None, limit=40):
            with connect() as sock:
                sock.settimeout(limit)
                msg = {'type':'comm_stop','id':rid,'session_id':requester,'target_session':target}
                if force is not None: msg['force'] = force
                send(sock,msg)
                with sock.makefile('rb') as sr:
                    return read_until(sr,lambda f:f.get('id')==rid and f.get('type') in ['done','error'],limit=limit)
        def status_of(target, rid, requester=None):
            return request({'type':'comm_status','id':rid,'session_id':requester or session,'target_session':target},['comm_status_response'])['snapshot']
        def restart_daemon(logname):
            global process, client, reader
            process.kill(); process.wait(timeout=5)
            reader.close(); client.close(); reader = client = None
            time.sleep(0.5)
            owned(native)
            if native.exists(): native.unlink()
            with open(state/logname,'wb') as log2:
                process = subprocess.Popen(argv,cwd=state,env=env,stdin=subprocess.DEVNULL,stdout=log2,stderr=log2)
            deadline = time.monotonic()+15
            while not native.exists():
                assert process.poll() is None, 'restarted daemon exited before socket readiness'
                assert time.monotonic()<deadline, 'restarted socket readiness deadline'
                select.select([],[],[],.01)
            client = connect(); reader = client.makefile('rb')
            send(client,{'type':'subscribe','id':40,'working_dir':str(state),'target_session_id':session,'allow_session_takeover':True})
            return read_until(reader,lambda f:f.get('type')=='session')['session_id']
        def graceful_reload():
            global client, reader
            send(client,{'type':'reload','id':900,'force':True})
            try:
                read_until(reader,lambda f:f.get('type') in ('reloading','done','error') and f.get('id',900)==900,limit=15)
            except AssertionError: pass
            reader.close(); client.close(); reader = client = None
            deadline = time.monotonic()+40
            time.sleep(1)
            while True:
                assert process.poll() is None, 'daemon exited during graceful reload'
                assert time.monotonic()<deadline, 'daemon did not return after graceful reload'
                try:
                    client = connect(); reader = client.makefile('rb')
                    send(client,{'type':'subscribe','id':40,'working_dir':str(state),'target_session_id':session,'allow_session_takeover':True})
                    return read_until(reader,lambda f:f.get('type')=='session')['session_id']
                except (OSError, AssertionError, ValueError):
                    try: client.close()
                    except Exception: pass
                    time.sleep(0.5)
        def dm_stopped(requester, target, base):
            replies = []
            for n, wake in enumerate((True, False)):
                with connect() as sk:
                    send(sk,{'type':'comm_message','id':base+n,'from_session':requester,'message':'dm to stopped worker','to_session':target,'wake':wake})
                    with sk.makefile('rb') as sr:
                        replies.append(read_until(sr,lambda f,n=n:f.get('id')==base+n and f.get('type') not in ('ack',),limit=15))
            return replies
        def other_session(rid):
            global intruder
            intruder = connect(); ireader = intruder.makefile('rb')
            send(intruder,{'type':'subscribe','id':rid,'working_dir':str(state),'system_prompt':'Unrelated idle session.'})
            return read_until(ireader,lambda f:f.get('type')=='session')['session_id']
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
                           'label':'isolated-stop-marker'},['comm_spawn_response'])
        worker = spawned['new_session_id']
        assert started.wait(10), 'worker never reached deterministic busy provider gate'
        if scenario in BUSY:
            deadline = time.monotonic()+10
            while not tool_started.exists():
                assert time.monotonic()<deadline, 'native tool never started'
                select.select([],[],[],.01)
        before = request({'type':'comm_status','id':4,'session_id':session,'target_session':worker},['comm_status_response'])
        assert before['snapshot']['activity']['is_processing'], 'worker was not busy at stop boundary'
        assert not marker.exists(), 'marker predated stop'
        if scenario == 'churn':
            s2 = request({'type':'comm_spawn','id':70,'session_id':session,'working_dir':str(state),
                          'initial_message':'Second idle worker.','spawn_mode':'headless','label':'second'},['comm_spawn_response'])
            worker2 = s2['new_session_id']
            deadline = time.monotonic()+10
            while True:
                snap2 = status_of(worker2,69)
                if len(requests) >= 2 and not (snap2.get('activity') or {}).get('is_processing'): break
                assert time.monotonic()<deadline, ('second worker never went idle', requests)
                select.select([],[],[],.1)
            ambiguous = stop_req(session,'session_',71)
            st1, st2 = status_of(worker,72), status_of(worker2,73)
            other = other_session(20)
            denied = stop_req(other,worker2,74)
            st2_denied = status_of(worker2,75)
            forced = stop_req(other,worker2,76,True)
            st2_forced = status_of(worker2,77)
            reader.close(); client.close()
            client = connect(); reader = client.makefile('rb')
            send(client,{'type':'subscribe','id':80,'working_dir':str(state),'target_session_id':session,'allow_session_takeover':True})
            reconnected = read_until(reader,lambda f:f.get('type')=='session')['session_id']
            n_pre = len(requests)
            owner_stop = stop_req(reconnected,worker,81)
            time.sleep(3)
            st1_final = status_of(worker,82)
            print(json.dumps({'scenario':'churn','ambiguous_reply':ambiguous,'unrelated_denied':denied,'forced_reply':forced,
                              'worker1_control_after_ambiguous':st1.get('control'),'worker2_control_after_ambiguous':st2.get('control'),
                              'worker2_after_denied':st2_denied.get('control'),'worker2_after_force':st2_forced.get('control'),
                              'worker2_member_status_after_force':st2_forced.get('status'),'reconnected_is_original':reconnected==session,
                              'owner_stop_reply_after_reconnect':owner_stop,'worker1_final':st1_final.get('control'),
                              'provider_requests':requests,'faults':faults},sort_keys=True),flush=True)
            assert ambiguous['type']=='error' and 'mbiguous' in ambiguous['message'], ambiguous
            assert st1['control']['state']=='runnable' and st2['control']['state']=='runnable', (st1,st2)
            assert denied['type']=='error' and 'Refusing to stop' in denied['message'], denied
            assert st2_denied['control']['state']=='runnable', st2_denied
            assert forced['type']=='done', forced
            assert st2_forced['control']['state']=='stopped' and st2_forced['control']['revoked_through']>=1, st2_forced
            assert reconnected == session
            assert owner_stop['type']=='done', owner_stop
            assert st1_final['control']['state']=='stopped', st1_final
            # Requests so far: worker1 bash turn, worker2 final text, and at most the
            # coordinator's own report-back turn. Nothing may run after worker1 is stopped.
            assert len(requests) == n_pre and len(requests) <= 3, requests
            assert not faults, faults
            sys.exit(0)
        if scenario == 'handoff':
            def raw(value, limit=45):
                with connect() as sk:
                    sk.settimeout(limit)
                    send(sk,value)
                    with sk.makefile('rb') as sr:
                        return read_until(sr,lambda f:f.get('id')==value['id'] and f.get('type') not in ('ack',),limit=limit)
            s2 = request({'type':'comm_spawn','id':70,'session_id':session,'working_dir':str(state),
                          'initial_message':'Second idle worker.','spawn_mode':'headless','label':'successor'},['comm_spawn_response'])
            successor = s2['new_session_id']
            deadline = time.monotonic()+10
            while len(requests) < 2:
                assert time.monotonic()<deadline, ('successor never ran its initial turn', requests)
                select.select([],[],[],.1)
            proposed = raw({'type':'comm_propose_plan','id':71,'session_id':session,
                            'items':[{'id':'t1','content':'handoff task','status':'queued','priority':'high'}]})
            assigned = raw({'type':'comm_assign_task','id':72,'session_id':session,'target_session':worker,'task_id':'t1'})
            # Stop cannot settle while the 40s shell child lives: the reply is unresolved.
            stop = stop_req(session,worker,73,limit=45)
            st_unsettled = status_of(worker,74)
            n_pre_refusal = len(requests)
            refused = raw({'type':'comm_task_control','id':75,'session_id':session,'action':'reassign','task_id':'t1','target_session':successor})
            st_successor_refused = status_of(successor,76)
            n_refused = len(requests)
            time.sleep(2)
            assert len(requests) == n_refused == n_pre_refusal, ('refused reassign started work', n_pre_refusal, n_refused, requests)
            # Child exits at ~40s; the retry then settles and the explicit reassign lands.
            deadline = time.monotonic()+60
            while not tool_marker.exists():
                assert time.monotonic()<deadline, 'shell child never exited'
                select.select([],[],[],.2)
            time.sleep(1)
            settled = stop_req(session,worker,77,limit=45)
            st_settled = status_of(worker,78)
            handed = raw({'type':'comm_task_control','id':79,'session_id':session,'action':'reassign','task_id':'t1','target_session':successor})
            time.sleep(2)
            print(json.dumps({'scenario':'claims_handoff','proposed':proposed,'assigned':assigned,'first_stop':stop,
                              'unsettled':st_unsettled.get('control'),'reassign_while_unsettled':refused,
                              'successor_after_refusal':st_successor_refused.get('status'),'second_stop':settled,
                              'settled':st_settled.get('control'),'reassign_after_settle':handed,
                              'provider_requests':requests,'faults':faults},sort_keys=True),flush=True)
            assert proposed['type'] != 'error', proposed
            assert assigned['type']=='comm_assign_task_response', assigned
            assert st_unsettled['control']['state'] != 'runnable', st_unsettled
            if stop['type']=='done':
                # The drain settled early; the refusal below only applies to an unsettled stop.
                assert refused['type'] in ('error','comm_assign_task_response'), refused
            else:
                assert refused['type']=='error' and 'not settled' in refused['message'], refused
            assert settled['type']=='done', settled
            assert st_settled['control']['state']=='stopped', st_settled
            assert handed['type']=='comm_assign_task_response' and handed['target_session']==successor, handed
            assert not faults, faults
            sys.exit(0)
        if scenario == 'reload_drain':
            with connect() as sock:
                sock.settimeout(20)
                send(sock,{'type':'comm_stop','id':30,'session_id':session,'target_session':worker})
                with sock.makefile('rb') as sr:
                    read_until(sr,lambda f:f.get('type')=='draining' and f.get('request_id')==30,limit=15)
            n_before = len(requests)
            pid_before = process.pid
            resumed = graceful_reload()
            snap = status_of(worker,41,resumed)
            time.sleep(2)
            snap_late = status_of(worker,42,resumed)
            dms = dm_stopped(resumed,worker,47)
            time.sleep(2)
            print(json.dumps({'scenario':'graceful_reload_mid_drain','same_pid':process.pid==pid_before,'resumed_is_original':resumed==session,
                              'after_reload':snap,'later':snap_late.get('control'),'dms':dms,'new_requests':len(requests)-n_before,'faults':faults},sort_keys=True),flush=True)
            assert resumed == session
            assert snap['control']['state'] != 'runnable' and snap['control']['revoked_through'] >= 1, snap
            assert snap_late['control']['state'] != 'runnable', snap_late
            assert snap.get('status') not in ('ready','running','completed'), snap
            assert len(requests) == n_before == 1, requests
            assert not faults, faults
            sys.exit(0)
        if scenario == 'reload_settled':
            stop = stop_req(session,worker,30,limit=40)
            assert stop['type']=='done', stop
            gen_stopped = status_of(worker,31)['control']['generation']
            n_before = len(requests)
            resumed = graceful_reload()
            snap = status_of(worker,41,resumed)
            n_dm = len(requests)
            dms = dm_stopped(resumed,worker,47)
            time.sleep(2)
            snap_dm = status_of(worker,48,resumed)
            n_after_dm = len(requests)
            with connect() as sk:
                send(sk,{'type':'comm_task_control','id':50,'session_id':resumed,'action':'wake','task_id':'','target_session':worker,'message':'resume after reload'})
                with sk.makefile('rb') as sr:
                    wake = read_until(sr,lambda f:f.get('id')==50 and f.get('type')!='ack')
            time.sleep(3)
            snap_resumed = status_of(worker,46,resumed)
            print(json.dumps({'scenario':'graceful_reload_settled','stopped_generation':gen_stopped,'after_reload':snap,'dms':dms,
                              'after_dm':snap_dm.get('control'),'wake_reply':wake,'after_wake':snap_resumed.get('control'),
                              'provider_requests':requests,'faults':faults},sort_keys=True),flush=True)
            assert resumed == session
            assert snap['control']['state']=='stopped' and snap['control']['revoked_through']>=1, snap
            assert snap.get('status') not in ('ready','running'), snap
            assert n_after_dm == n_dm, ('DM to stopped worker after reload started a turn', requests)
            assert snap_dm['control']['state']=='stopped', snap_dm
            assert wake['type']=='comm_task_control_response', wake
            assert snap_resumed['control']['state']=='runnable' and snap_resumed['control']['generation']>=gen_stopped+1, snap_resumed
            assert len(requests) > n_after_dm, ('explicit wake after reload did not start a turn', requests)
            assert not faults, faults
            sys.exit(0)
        if scenario == 'stalewait':
            import queue
            out = queue.Queue()
            def do_wait():
                with connect() as sk:
                    sk.settimeout(40)
                    send(sk,{'type':'comm_await_members','id':70,'session_id':session,'target_status':['completed','ready'],
                             'session_ids':[worker],'timeout_secs':20,'background':False,'notify':False})
                    with sk.makefile('rb') as sr:
                        out.put(read_until(sr,lambda f:f.get('id')==70 and f.get('type')!='ack',limit=35))
            waiter = threading.Thread(target=do_wait); waiter.start()
            time.sleep(1.5)
            stop = stop_req(session,worker,30,limit=40)
            assert stop['type']=='done', stop
            gen_stopped = status_of(worker,31)['control']['generation']
            with connect() as sk:
                send(sk,{'type':'comm_task_control','id':50,'session_id':session,'action':'wake','task_id':'','target_session':worker,'message':'new work after stop'})
                with sk.makefile('rb') as sr:
                    wake = read_until(sr,lambda f:f.get('id')==50 and f.get('type')!='ack')
            time.sleep(4)
            resumed = status_of(worker,32)
            waiter.join()
            reply = out.get()
            print(json.dumps({'scenario':'stale_wait_replay','stopped_generation':gen_stopped,'wake_reply':wake,'worker_after_new_work':resumed,
                              'wait_reply':reply,'provider_requests':requests,'faults':faults},sort_keys=True),flush=True)
            assert wake['type']=='comm_task_control_response', wake
            assert len(requests) >= 2, ('wake did not start new work', requests)
            assert resumed['control']['generation'] >= gen_stopped+1, resumed
            assert reply['type']=='comm_await_members_response', reply
            assert reply['completed'] is False, ('wait bound to the stopped generation completed from newer work', reply)
            assert all(not m['done'] for m in reply['members']), reply
            assert not faults, faults
            sys.exit(0)
        if scenario == 'restart_drain':
            with connect() as sock:
                sock.settimeout(20)
                send(sock,{'type':'comm_stop','id':30,'session_id':session,'target_session':worker})
                with sock.makefile('rb') as sr:
                    read_until(sr,lambda f:f.get('type')=='draining' and f.get('request_id')==30,limit=15)
            n_before = len(requests)
            resumed = restart_daemon('daemon2.log')
            snap = status_of(worker,41,resumed)
            time.sleep(2)
            snap_late = status_of(worker,42,resumed)
            print(json.dumps({'scenario':'restart_mid_drain','resumed_is_original':resumed==session,'after_restart':snap,
                              'later':snap_late.get('control'),'new_requests':len(requests)-n_before,'faults':faults},sort_keys=True),flush=True)
            assert resumed == session
            assert snap['control']['state'] != 'runnable' and snap['control']['revoked_through'] >= 1, snap
            assert snap_late['control']['state'] != 'runnable', snap_late
            assert snap.get('status') not in ('ready','running','completed'), snap
            assert len(requests) == n_before == 1, requests
            assert not faults, faults
            sys.exit(0)
        if scenario == 'stopwake':
            import queue
            out = queue.Queue()
            def do_stop(): out.put(('stop',stop_req(session,worker,30)))
            def do_wake():
                with connect() as sk:
                    sk.settimeout(40)
                    send(sk,{'type':'comm_task_control','id':50,'session_id':session,'action':'wake','task_id':'','target_session':worker,'message':'wake during stop'})
                    with sk.makefile('rb') as sr:
                        out.put(('wake',read_until(sr,lambda f:f.get('id')==50 and f.get('type')!='ack',limit=40)))
            ts = [threading.Thread(target=do_stop),threading.Thread(target=do_wake)]
            for t in ts: t.start()
            for t in ts: t.join()
            res = dict(out.get() for _ in range(2))
            time.sleep(3)
            final = status_of(worker,60)
            print(json.dumps({'scenario':'concurrent_stop_wake','stop_reply':res['stop'],'wake_reply':res['wake'],
                              'final':final,'provider_requests':requests,'faults':faults},sort_keys=True),flush=True)
            c = final['control']
            # Either ordering is legal: the wake is admitted first (generation G, then revoked by the
            # stop) or it is refused/admitted after the stop. What must never happen is a runnable
            # generation at or below the revoked one, or a model turn after the stop barrier.
            if c['state']=='runnable':
                assert c['generation'] > c['revoked_through'] >= 1, final
            else:
                assert c['state'] in ('stopped','unresolved') and c['revoked_through']>=1, final
                assert len(requests) == 1, ('turn ran after the stop barrier', requests)
            assert res['stop']['type'] in ('done','error'), res
            assert c.get('admitted_generation') is None or c['admitted_generation'] > c['revoked_through'], final
            assert not faults, faults
            sys.exit(0)
        if scenario == 'alias':
            alias = before['snapshot']['friendly_name']
            intruder = connect(); ireader = intruder.makefile('rb')
            send(intruder,{'type':'subscribe','id':20,'working_dir':str(state),'system_prompt':'Unrelated idle session.'})
            other = read_until(ireader,lambda f:f.get('type')=='session')['session_id']
            send(intruder,{'type':'comm_stop','id':21,'session_id':other,'target_session':alias})
            denied = read_until(ireader,lambda f:f.get('id')==21 and f.get('type') in ['done','error'])
            still = request({'type':'comm_status','id':22,'session_id':session,'target_session':worker},['comm_status_response'])
            with connect() as sock:
                send(sock,{'type':'comm_stop','id':30,'session_id':session,'target_session':alias})
                with sock.makefile('rb') as sr:
                    stop = read_until(sr,lambda f:f.get('id')==30 and f.get('type') in ['done','error'])
            after = request({'type':'comm_status','id':31,'session_id':session,'target_session':worker},['comm_status_response'])
            print(json.dumps({'scenario':'alias','alias':alias,'unrelated_alias_reply':denied,
                              'worker_after_denied':still['snapshot'].get('control'),'owner_alias_stop_reply':stop,
                              'worker_after_owner_stop':after['snapshot'].get('control'),'faults':faults},sort_keys=True),flush=True)
            assert denied['type']=='error', denied
            assert still['snapshot']['activity']['is_processing'] and still['snapshot']['control']['state']=='runnable', still
            assert after['snapshot']['control']['revoked_through'] >= 1, after
            assert after['snapshot']['control']['state'] != 'runnable', after
            assert len(requests) == 1, requests
            assert not faults, faults
            sys.exit(0)
        if scenario in BUSY:
            with connect() as sock:
                sock.settimeout(40)
                send(sock,{'type':'comm_stop','id':30,'session_id':session,'target_session':worker})
                with sock.makefile('rb') as sr:
                    stop = read_until(sr,lambda f:f.get('id')==30 and f.get('type') in ['done','error'],limit=40)
            stop_at = time.monotonic()
            finished_at_stop = tool_marker.exists()
            stop_frames = [f['type'] for f in frames if (f.get('id')==30 or f.get('request_id')==30) and f['type'] != 'ack']
            after = request({'type':'comm_status','id':31,'session_id':session,'target_session':worker},['comm_status_response'])
            deadline = time.monotonic()+(9 if scenario == 'bash' else 1)
            while not tool_marker.exists() and time.monotonic()<deadline:
                select.select([],[],[],.05)
            finished = tool_marker.exists()
            result = {'scenario':'native_tool_busy','binary':str(binary),'binary_sha256':sha(binary.read_bytes()),
                      'binary_version':subprocess.check_output([str(binary),'--version'],env=env,cwd=state,text=True).strip(),
                      'busy_snapshot':before,'stop_reply':stop,'status_after_stop':after,
                      'tool_finished_after_stop':finished,'tool_finished_at_stop_reply':finished_at_stop,
                      'stop_frames':stop_frames,'provider_requests':requests,
                      'observed_after_stop_secs':round(time.monotonic()-stop_at,3),'faults':faults}
            print(json.dumps(result,sort_keys=True),flush=True)
            if scenario == 'restart':
                assert stop['type'] == 'error' and 'unresolved' in stop['message'], stop
                control_before = after['snapshot'].get('control')
                assert control_before and control_before['state'] != 'runnable', after
                # Kill the daemon outright (no graceful settle), then restart it on the same state.
                process.kill(); process.wait(timeout=5)
                reader.close(); client.close(); reader = client = None
                time.sleep(0.5)
                owned(native)
                if native.exists(): native.unlink()
                n_before = len(requests)
                with open(state/'daemon2.log','wb') as log2:
                    process = subprocess.Popen(argv,cwd=state,env=env,stdin=subprocess.DEVNULL,stdout=log2,stderr=log2)
                deadline = time.monotonic()+15
                while not native.exists():
                    assert process.poll() is None, 'restarted daemon exited before socket readiness'
                    assert time.monotonic()<deadline, 'restarted socket readiness deadline'
                    select.select([],[],[],.01)
                client = connect(); reader = client.makefile('rb')
                send(client,{'type':'subscribe','id':40,'working_dir':str(state),'target_session_id':session,'allow_session_takeover':True})
                resumed = read_until(reader,lambda f:f.get('type')=='session')['session_id']
                restored = request({'type':'comm_status','id':41,'session_id':resumed,'target_session':worker},['comm_status_response'])
                snap = restored['snapshot']
                control_after = snap.get('control')
                time.sleep(3)
                result = {'scenario':'restart_mid_stop','binary':str(binary),'binary_sha256':sha(binary.read_bytes()),
                          'resumed_requester_is_original':resumed == session,'stop_reply':stop,
                          'control_before_restart':control_before,'status_after_restart':restored,
                          'provider_requests_total':requests,'new_requests_after_restart':len(requests)-n_before,
                          'faults':faults}
                print(json.dumps(result,sort_keys=True),flush=True)
                assert resumed == session, 'could not resume the original requester session'
                assert control_after and control_after['state'] != 'runnable', restored
                assert control_after['revoked_through'] >= control_before['revoked_through'], restored
                assert snap.get('status') not in ('ready','running','completed'), restored
                assert len(requests) == n_before, ('restart re-ran stopped worker', requests)
                assert not faults, faults
                sys.exit(0)
            # stop_requested and draining are nonterminal and precede the terminal reply.
            assert stop_frames[:2] == ['stop_requested','draining'] and stop_frames[-1] == stop['type'], stop_frames
            # The shell child is accounted per process: Done is only allowed once it is
            # proven gone, so the tool outcome can no longer change after the reply.
            assert stop['type'] == 'done', stop
            assert finished == finished_at_stop, 'native tool changed state after stop was confirmed'
            assert after['snapshot'].get('control',{}).get('state') == 'stopped', after
            if scenario == 'restart_settled':
                gen_stopped = after['snapshot']['control']['generation']
                resumed = restart_daemon('daemon2.log')
                n_before = len(requests)
                snap = status_of(worker,41,resumed)
                other = other_session(42)
                denied = stop_req(other,worker,43)
                owner_again = stop_req(resumed,worker,44)
                snap_after_stop = status_of(worker,45,resumed)
                n_dm = len(requests)
                dms = dm_stopped(resumed,worker,47)
                time.sleep(2)
                snap_after_dm = status_of(worker,48,resumed)
                assert len(requests) == n_dm, ('DM to stopped revived worker started a turn', requests)
                assert snap_after_dm['control']['state']=='stopped', snap_after_dm
                with connect() as sk:
                    send(sk,{'type':'comm_task_control','id':50,'session_id':resumed,'action':'wake','task_id':'','target_session':worker,'message':'resume after restart'})
                    with sk.makefile('rb') as sr:
                        wake = read_until(sr,lambda f:f.get('id')==50 and f.get('type')!='ack')
                time.sleep(3)
                snap_resumed = status_of(worker,46,resumed)
                print(json.dumps({'scenario':'restart_after_settled_stop','stopped_generation':gen_stopped,'after_restart':snap,
                                  'unrelated_denied':denied,'owner_repeat_stop':owner_again,'dm_replies_to_stopped_after_restart':dms,'after_dm':snap_after_dm.get('control'),'after_repeat':snap_after_stop.get('control'),
                                  'wake_reply':wake,'after_wake':snap_resumed.get('control'),'provider_requests':requests,'faults':faults},sort_keys=True),flush=True)
                assert resumed == session
                assert snap['control']['state']=='stopped' and snap['control']['revoked_through']>=1, snap
                assert snap.get('status') not in ('ready','running'), snap
                assert denied['type']=='error', denied
                assert owner_again['type']=='done', owner_again
                assert snap_after_stop['control']['state']=='stopped', snap_after_stop
                assert wake['type']=='comm_task_control_response', wake
                assert snap_resumed['control']['state']=='runnable' and snap_resumed['control']['generation']>=gen_stopped+1, snap_resumed
                assert len(requests) > n_before, ('wake did not start a turn', requests)
                assert not faults, faults
                sys.exit(0)
            if scenario == 'resume':
                assert after['snapshot']['control']['state'] == 'stopped', after
                gen_stopped = after['snapshot']['control']['generation']
                assert len(requests) == 1, requests
                dms = dm_stopped(session,worker,90)
                time.sleep(2)
                snap_dm = status_of(worker,92)
                assert len(requests) == 1, ('DM to stopped live worker started a turn', requests)
                assert snap_dm['control']['state']=='stopped', snap_dm
                # Two concurrent explicit wakes: exactly one new generation may be admitted.
                socks = [connect(), connect()]
                for n, sk in enumerate(socks):
                    send(sk,{'type':'comm_task_control','id':50+n,'session_id':session,'action':'wake',
                             'task_id':'','target_session':worker,'message':'resume after stop'})
                replies = []
                for n, sk in enumerate(socks):
                    with sk.makefile('rb') as sr:
                        replies.append(read_until(sr,lambda f,n=n:f.get('id')==50+n and f.get('type') not in ('ack',)))
                    sk.close()
                time.sleep(3)
                resumed = request({'type':'comm_status','id':60,'session_id':session,'target_session':worker},['comm_status_response'])
                control = resumed['snapshot']['control']
                print(json.dumps({'scenario':'resume_race','dm_replies_to_stopped_live_worker':dms,'generation_when_stopped':gen_stopped,'wake_replies':replies,
                                  'control_after':control,'provider_requests':requests,'faults':faults},sort_keys=True),flush=True)
                assert all(r['type']=='comm_task_control_response' for r in replies), replies
                # One winner resumes (G+1). Its turn then admits its own generation (G+2).
                # A second resume would have produced G+3 or more.
                assert control['generation'] == gen_stopped + 2, ('exactly one resume and one turn', control)
                assert control['state'] == 'runnable' and control['admitted_generation'] == gen_stopped + 2, control
                assert control['revoked_through'] == gen_stopped, control
                assert sorted(r['status'] for r in replies) == ['queued','running'], replies
                assert not faults, faults
                sys.exit(0)
            assert len(requests) == 1, ('stopped worker made another model request', requests)
            assert not faults, faults
            sys.exit(0)
        intruder = connect(); ireader = intruder.makefile('rb')
        send(intruder,{'type':'subscribe','id':20,'working_dir':str(state),'system_prompt':'Unrelated idle session.'})
        other = read_until(ireader,lambda f:f.get('type')=='session')['session_id']
        send(intruder,{'type':'comm_stop','id':21,'session_id':other,'target_session':worker})
        denied = read_until(ireader,lambda f:f.get('id')==21 and f.get('type') in ['done','error'])
        assert denied['type']=='error' and 'Refusing to stop' in denied['message'], denied
        assert not marker.exists(), 'denied stop changed worker state'
        still = request({'type':'comm_status','id':22,'session_id':session,'target_session':worker},['comm_status_response'])
        assert still['snapshot']['activity']['is_processing'], 'denied stop interrupted the worker'
        stop = request({'type':'comm_stop','id':5,'session_id':session,'target_session':worker},['done'])
        stop_at = time.monotonic()
        repeat = request({'type':'comm_stop','id':7,'session_id':session,'target_session':worker},['done'])
        assert repeat['type']=='done', repeat
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
                  'worker':worker,'busy_snapshot':before,'stop_reply':stop,'repeat_stop_reply':repeat,'unrelated_denial':denied['message'],
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
        try: intruder.close()
        except NameError: pass
        if client is not None: client.close()
        if process is not None:
            process.terminate()
            try: process.wait(timeout=5)
            except subprocess.TimeoutExpired: process.kill(); process.wait(timeout=5)
        fixture.shutdown(); fixture.server_close(); thread.join(timeout=2)
        if sys.exc_info()[0] not in (None, SystemExit):
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

#[test]
#[ignore = "actual private binary + Linux user/network namespaces; see change evidence.md"]
fn public_stop_during_native_tool_never_reports_stopped_while_tool_runs() {
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
        .arg("bash")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("TMPDIR", root.join("scratch"))
        .output()
        .expect("launch isolated native-tool-busy acceptance");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    assert!(
        output.status.success(),
        "private native-tool-busy acceptance failed ({}):\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "actual private binary + Linux user/network namespaces; see change evidence.md"]
fn restart_after_unresolved_stop_keeps_worker_revoked() {
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
        .arg("restart")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("TMPDIR", root.join("scratch"))
        .output()
        .expect("launch isolated restart-mid-stop acceptance");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    assert!(
        output.status.success(),
        "private restart-mid-stop acceptance failed ({}):\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "actual private binary + Linux user/network namespaces; see change evidence.md"]
fn alias_stop_is_swarm_local_over_the_public_socket() {
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
        .arg("alias")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("TMPDIR", root.join("scratch"))
        .output()
        .expect("launch isolated alias acceptance");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    assert!(
        output.status.success(),
        "private alias acceptance failed ({}):\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "actual private binary + Linux user/network namespaces; see change evidence.md"]
fn concurrent_wakes_after_settled_stop_admit_exactly_one_generation() {
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
        .arg("resume")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("TMPDIR", root.join("scratch"))
        .output()
        .expect("launch isolated resume race acceptance");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    assert!(
        output.status.success(),
        "private resume race acceptance failed ({}):\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "actual private binary + Linux user/network namespaces; see change evidence.md"]
fn owner_control_survives_churn_ambiguity_denial_and_force_over_the_public_socket() {
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
        .arg("churn")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("TMPDIR", root.join("scratch"))
        .output()
        .expect("launch isolated churn control acceptance");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    assert!(
        output.status.success(),
        "private churn control acceptance failed ({}):\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "actual private binary + Linux user/network namespaces; see change evidence.md"]
fn restart_after_settled_stop_stays_stopped_until_explicit_wake() {
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
        .arg("restart_settled")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("TMPDIR", root.join("scratch"))
        .output()
        .expect("launch isolated settled restart acceptance");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    assert!(
        output.status.success(),
        "private settled restart acceptance failed ({}):\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "actual private binary + Linux user/network namespaces; see change evidence.md"]
fn restart_while_draining_keeps_worker_revoked() {
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
        .arg("restart_drain")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("TMPDIR", root.join("scratch"))
        .output()
        .expect("launch isolated drain restart acceptance");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    assert!(
        output.status.success(),
        "private drain restart acceptance failed ({}):\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "actual private binary + Linux user/network namespaces; see change evidence.md"]
fn concurrent_stop_and_wake_never_leave_a_runnable_revoked_generation() {
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
        .arg("stopwake")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("TMPDIR", root.join("scratch"))
        .output()
        .expect("launch isolated stop wake race acceptance");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    assert!(
        output.status.success(),
        "private stop wake race acceptance failed ({}):\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "actual private binary + Linux user/network namespaces; see change evidence.md"]
fn reassign_waits_for_stop_settlement_then_hands_the_task_off() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let binary = std::env::var_os("JCODE_WORKER_STOP_BINARY")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join("target/selfdev/jcode"));
    let namespace = std::fs::read_link("/proc/self/ns/net").expect("Linux network namespace");
    let output = Command::new("/usr/bin/timeout")
        .args(["--kill-after=5", "150", "/usr/bin/unshare"])
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
        .arg("handoff")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("TMPDIR", root.join("scratch"))
        .output()
        .expect("launch isolated claims handoff acceptance");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    assert!(
        output.status.success(),
        "private claims handoff acceptance failed ({}):\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "actual private binary + Linux user/network namespaces; see change evidence.md"]
fn graceful_reload_after_settled_stop_stays_stopped_until_explicit_wake() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let binary = std::env::var_os("JCODE_WORKER_STOP_BINARY")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join("target/selfdev/jcode"));
    let namespace = std::fs::read_link("/proc/self/ns/net").expect("Linux network namespace");
    let output = Command::new("/usr/bin/timeout")
        .args(["--kill-after=5", "120", "/usr/bin/unshare"])
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
        .arg("reload_settled")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("TMPDIR", root.join("scratch"))
        .output()
        .expect("launch isolated graceful reload acceptance");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    assert!(
        output.status.success(),
        "private graceful reload acceptance failed ({}):\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "actual private binary + Linux user/network namespaces; see change evidence.md"]
fn graceful_reload_while_draining_keeps_worker_revoked() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let binary = std::env::var_os("JCODE_WORKER_STOP_BINARY")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join("target/selfdev/jcode"));
    let namespace = std::fs::read_link("/proc/self/ns/net").expect("Linux network namespace");
    let output = Command::new("/usr/bin/timeout")
        .args(["--kill-after=5", "120", "/usr/bin/unshare"])
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
        .arg("reload_drain")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("TMPDIR", root.join("scratch"))
        .output()
        .expect("launch isolated graceful reload acceptance");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    assert!(
        output.status.success(),
        "private graceful reload acceptance failed ({}):\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "actual private binary + Linux user/network namespaces; see change evidence.md"]
fn wait_bound_to_stopped_generation_never_completes_from_resumed_work() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let binary = std::env::var_os("JCODE_WORKER_STOP_BINARY")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join("target/selfdev/jcode"));
    let namespace = std::fs::read_link("/proc/self/ns/net").expect("Linux network namespace");
    let output = Command::new("/usr/bin/timeout")
        .args(["--kill-after=5", "120", "/usr/bin/unshare"])
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
        .arg("stalewait")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("TMPDIR", root.join("scratch"))
        .output()
        .expect("launch isolated graceful reload acceptance");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    assert!(
        output.status.success(),
        "private graceful reload acceptance failed ({}):\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}
