const vscode = require('vscode');
const fs = require('fs');
const path = require('path');
const cp = require('child_process');
const assert = require('assert');
const root = path.resolve(__dirname, '..');
const bin = path.join(require('os').homedir(), '.local/bin/herdr');
const session = 'ide-spike-20260923';
const evidence = path.join(root, 'evidence');
const env = {...process.env};
for (const key of Object.keys(env)) if (key.startsWith('HERDR_')) delete env[key];
const sleep = ms => new Promise(r => setTimeout(r, ms));
function log(event, data={}) { fs.appendFileSync(path.join(evidence,'events.jsonl'),JSON.stringify({at:new Date().toISOString(),event,...data})+'\n'); }
function api(...args) {
 const raw = cp.execFileSync(bin,['--session',session,...args],{env,encoding:'utf8',timeout:10000});
 const x = JSON.parse(raw); if (x.ok === false || x.error) throw Error(JSON.stringify(x)); return x.result || x;
}
async function until(fn, label, timeout=15000) {
 const end=Date.now()+timeout; let last;
 while(Date.now()<end) { try {const v=await fn();if(v)return v;}catch(e){last=e.message;}await sleep(150); }
 throw Error('Timeout: '+label+' '+(last||''));
}
function state() {return JSON.parse(fs.readFileSync(path.join(root,'state.json'),'utf8'));}
function save(s) {fs.writeFileSync(path.join(root,'state.json'),JSON.stringify(s,null,2));}
function heartbeat() {return fs.readFileSync(path.join(evidence,'heartbeat.txt'),'utf8').trim().split('\n').at(-1).split(' ');}
function attach(id,name) {const t=vscode.window.createTerminal({name,shellPath:bin,shellArgs:['--session',session,'terminal','attach',id,'--takeover'],env:Object.fromEntries(Object.keys(process.env).filter(k=>k.startsWith('HERDR_')).map(k=>[k,null])),isTransient:true});t.show();return t;}
function pidAlive(pid) {try{process.kill(Number(pid),0);return true;}catch{return false;}}
exports.run = async function() {
 const phase=fs.readFileSync(path.join(root,'phase'),'utf8').trim();
 log('test-start',{phase,vscodeVersion:vscode.version,appName:vscode.env.appName,remoteName:vscode.env.remoteName});
 try {
 if(phase==='create') {
  const result=api('workspace','create','--cwd',path.join(root,'project'),'--label','Cursor disposable spike','--no-focus');
  log('workspace-created',{result});
  const pane=result.root_pane;assert(pane&&pane.terminal_id,'workspace root pane with terminal ID');
  const s={paneId:pane.pane_id,terminalId:pane.terminal_id,workspaceId:result.workspace.workspace_id,origin:'ide',hostPid:process.ppid};save(s);
  let t=attach(s.terminalId,'HERDR SPIKE • IDE-owned');
  const attachPid=await t.processId;log('attached',{attachPid});
  t.sendText(`printf '%s\\n' "$$" > '${evidence}/shell.pid'; '${process.env.SHELL || '/bin/zsh'}' -c 'echo $$ > "${evidence}/child.pid"; i=0; while :; do i=$((i+1)); printf "%s %s\\n" "$$" "$i" >> "${evidence}/heartbeat.txt"; sleep 1; done' &`,true);
  await until(()=>fs.existsSync(path.join(evidence,'heartbeat.txt')),'heartbeat creation');
  s.shellPid=Number(fs.readFileSync(path.join(evidence,'shell.pid'),'utf8'));s.childPid=Number(fs.readFileSync(path.join(evidence,'child.pid'),'utf8'));save(s);
  const before=heartbeat();
  t.sendText(`printf 'HERDR_FIDELITY_MARKER unicode: café 日本語 ✓\\n'; printf '%s\\n' 'paste-safe; literal text' > '${evidence}/paste.txt'; stty size > '${evidence}/size.txt'`,true);
  await until(()=>fs.existsSync(path.join(evidence,'paste.txt')),'native terminal sendText');
  assert.equal(fs.readFileSync(path.join(evidence,'paste.txt'),'utf8').trim(),'paste-safe; literal text');
  log('input-verified',{size:fs.readFileSync(path.join(evidence,'size.txt'),'utf8').trim(),shellIntegration:!!t.shellIntegration});
  await sleep(1500);
  t.dispose();await until(()=>!vscode.window.terminals.includes(t),'terminal disposed');
  await until(()=>Number(heartbeat()[1])>Number(before[1])+2,'progress after terminal close');
  assert(pidAlive(s.shellPid)&&pidAlive(s.childPid));
  log('PASS-terminal-close',{shellPid:s.shellPid,childPid:s.childPid,heartbeat:heartbeat()});
  t=attach(s.terminalId,'HERDR SPIKE • reattached');await t.processId;
  t.sendText(`printf '%s\\n' "$$" > '${evidence}/reattach-shell.pid'`,true);
  await until(()=>fs.existsSync(path.join(evidence,'reattach-shell.pid')),'reattached input');
  assert.equal(Number(fs.readFileSync(path.join(evidence,'reattach-shell.pid'),'utf8')),s.shellPid);
  log('PASS-reattach-same-shell',{shellPid:s.shellPid});
  fs.writeFileSync(path.join(evidence,'ready-for-capture'),String(Date.now()));
  await until(()=>fs.existsSync(path.join(evidence,'continue-create')),'capture completion',120000);
  log('ready-for-editor-exit',{heartbeat:heartbeat(),attachmentPid:await t.processId,shellIntegration:!!t.shellIntegration});
 } else if(phase==='reattach') {
  const s=state();assert(pidAlive(s.shellPid)&&pidAlive(s.childPid));
  let t=attach(s.terminalId,'HERDR SPIKE • survived IDE exit');await t.processId;
  t.sendText(`printf '%s\\n' "$$" > '${evidence}/after-exit-shell.pid'; printf 'SAME_SESSION_AFTER_IDE_EXIT\\n'`,true);
  await until(()=>fs.existsSync(path.join(evidence,'after-exit-shell.pid')),'after exit input');
  assert.equal(Number(fs.readFileSync(path.join(evidence,'after-exit-shell.pid'),'utf8')),s.shellPid);
  log('PASS-editor-exit-reattach',{shellPid:s.shellPid,childPid:s.childPid,heartbeat:heartbeat()});
  const oldPid=await t.processId;const replacement=attach(s.terminalId,'HERDR SPIKE • automatic takeover');await replacement.processId;
  await until(()=>!pidAlive(oldPid),'old attachment displaced');
  replacement.sendText(`printf 'takeover-ok\\n' > '${evidence}/takeover.txt'`,true);
  await until(()=>fs.existsSync(path.join(evidence,'takeover.txt')),'takeover input');
  assert(pidAlive(s.shellPid)&&pidAlive(s.childPid));log('PASS-ide-origin-takeover',{oldAttachPid:oldPid,newAttachPid:await replacement.processId});
  const obs=cp.spawn(bin,['--session',session,'terminal','session','observe',s.paneId,'--cols','90','--rows','25'],{env,stdio:['pipe','pipe','pipe']});
  let frames=0;let lines='';obs.stdout.on('data',d=>{lines+=d;let i;while((i=lines.indexOf('\n'))>=0){const line=lines.slice(0,i);lines=lines.slice(i+1);try{if(JSON.parse(line).type==='terminal.frame')frames++;}catch{}}});
  await until(()=>frames>0,'observer frame');obs.stdin.write(JSON.stringify({type:'terminal.input',text:`echo forbidden > '${evidence}/observer-wrote.txt'\r`})+'\n');
  await sleep(1200);assert(!fs.existsSync(path.join(evidence,'observer-wrote.txt')));assert(pidAlive(await replacement.processId));
  log('PASS-observe-readonly-coexists',{frames});obs.kill();
  fs.writeFileSync(path.join(evidence,'phase2-ready'),String(Date.now()));
  await until(()=>fs.existsSync(path.join(evidence,'continue-reattach')),'phase2 completion',120000);
 } else if(phase==='fidelity') {
  const s=state();let t=attach(s.terminalId,'HERDR SPIKE • fidelity');await t.processId;
  async function shell(cmd,file) {t.sendText(cmd,true);await until(()=>fs.existsSync(path.join(evidence,file)),file);return fs.readFileSync(path.join(evidence,file),'utf8');}
  const beforeSize=await shell(`stty size > '${evidence}/resize-before.txt'`,'resize-before.txt');
  await vscode.commands.executeCommand('workbench.action.toggleMaximizedPanel');await sleep(700);
  const afterSize=await shell(`stty size > '${evidence}/resize-after.txt'`,'resize-after.txt');
  log(beforeSize!==afterSize?'PASS-resize-propagation':'UNVERIFIED-resize',{before:beforeSize.trim(),after:afterSize.trim()});
  const alt = await shell(`printf '\\033[?1049h\\033[2JALT_SCREEN_PROBE\\r\\n'; sleep 1; printf '\\033[?1049l'; printf 'alt-returned\\n' > '${evidence}/alt.txt'`,'alt.txt');
  assert(alt.includes('alt-returned'));log('PASS-altscreen-command-completes',{visualInspection:false});
  t.sendText(`sleep 90; printf 'foreground-returned\\n' > '${evidence}/ctrlc.txt'`,true);await sleep(500);t.sendText('\x03',false);
  await until(()=>fs.existsSync(path.join(evidence,'ctrlc.txt')),'Ctrl-C interrupts sleep');assert(pidAlive(s.shellPid)&&pidAlive(s.childPid));log('PASS-Ctrl-C');
  const old=await t.processId;process.kill(old,'SIGKILL');await until(()=>!pidAlive(old),'attachment killed');const beat=Number(heartbeat()[1]);await until(()=>Number(heartbeat()[1])>beat,'child progress after attachment crash');
  t=attach(s.terminalId,'HERDR SPIKE • after attachment crash');await t.processId;
  const again=await shell(`printf '%s\\n' "$$" > '${evidence}/after-crash-shell.pid'`,'after-crash-shell.pid');assert.equal(Number(again),s.shellPid);log('PASS-attachment-SIGKILL',{shellPid:s.shellPid,childPid:s.childPid});
  const tabs=[];for(let i=0;i<2;i++)tabs.push(api('tab','create','--workspace',s.workspaceId,'--cwd',path.join(root,'project'),'--label','spike-'+i,'--no-focus'));
  const listing=api('tab','list','--workspace',s.workspaceId);log('tab-list-result',{listing});
  const list=Array.isArray(listing)?listing:listing.tabs;assert.equal(list.length,3);log('PASS-three-tabs-one-workspace',{count:list.length});
  const observeTarget=tabs[0].root_pane || tabs[0].pane;assert(observeTarget,'new tab root pane');
  let observer;let frameCount=0;const writes=new vscode.EventEmitter();const closes=new vscode.EventEmitter();let rejectedInput=0;
  const pt={onDidWrite:writes.event,onDidClose:closes.event,open(dim){observer=cp.spawn(bin,['--session',session,'terminal','session','observe',observeTarget.pane_id,'--cols',String(dim?.columns||80),'--rows',String(dim?.rows||24)],{env,stdio:['ignore','pipe','pipe']});let pending='';observer.stdout.on('data',d=>{pending+=d;let i;while((i=pending.indexOf('\n'))>=0){const line=pending.slice(0,i);pending=pending.slice(i+1);const msg=JSON.parse(line);if(msg.type==='terminal.frame'){frameCount++;writes.fire(Buffer.from(msg.bytes,'base64').toString('utf8'));}}});observer.on('exit',()=>closes.fire());},handleInput(){rejectedInput++;},close(){observer?.kill();}};
  const view=vscode.window.createTerminal({name:'HERDR SPIKE • external-origin observer fixture',pty:pt,isTransient:true});view.show();
  await until(()=>frameCount>0,'native observer terminal frame');view.sendText('echo should-not-execute',true);await until(()=>rejectedInput>0,'observer input ignored');
  log('PASS-native-Pseudoterminal-observer',{frames:frameCount,rejectedInput,originPolicy:'fixture only; no durable provenance implementation'});view.dispose();
  fs.writeFileSync(path.join(evidence,'fidelity-ready'),String(Date.now()));await until(()=>fs.existsSync(path.join(evidence,'continue-fidelity')),'fidelity release',120000);
 }
 log('PASS-phase',{phase});
 } catch(e) {log('FAIL',{phase,message:e.message,stack:e.stack});throw e;}
};
