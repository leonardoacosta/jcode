#!/usr/bin/env python3
"""Code-search eval harness. Usage: harness.py [--agentgrep BIN] > results.json
Per query+strategy: tool_calls, tokens (output chars/4 estimate), recall@1/@5 (file-level), wall_ms."""
import json, os, re, shutil, subprocess, sys, time
ROOT = subprocess.check_output(["git", "rev-parse", "--show-toplevel"], text=True).strip()
HERE = os.path.dirname(os.path.abspath(__file__))
AG = sys.argv[sys.argv.index("--agentgrep") + 1] if "--agentgrep" in sys.argv else shutil.which("agentgrep")
STOP = {"who","calls","callers","of","the","struct","definition","type","tool","with","and"}

def run(cmd):
    t = time.monotonic(); p = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True)
    return p.stdout, (time.monotonic() - t) * 1000

def paths(out):
    seen = []
    for m in re.finditer(r"((?:crates|src)/[\w./-]+\.rs)", out):
        if m.group(1) not in seen: seen.append(m.group(1))
    return seen

def agentgrep(q):
    if q.get("sym"):
        out, ms = run([AG, "grep", "--paths-only", q["sym"]]) if q["kind"] != "caller" else run([AG, "grep", q["sym"] + "("])
        # symbol: rank definition files first
        defs = [p for p in paths(out) if re.search(r"(struct|fn|enum|trait) " + q["sym"] + r"\b", open(os.path.join(ROOT, p), errors="ignore").read())] if q["kind"] != "caller" else []
        ranked = defs + [p for p in paths(out) if p not in defs]
        return ranked, out, ms, 1
    out, ms = run([AG, "find", "--paths-only", *q["q"].split()])
    return paths(out), out, ms, 1

def graft(q):
    out, ms = run(["graft", "ask", "--no-refresh", "-n", "8", q["q"]])
    return paths(out), out, ms, 1

def astgrep(q):
    if not q.get("sg"): return None
    files, outs, ms = [], "", 0
    for pat in q["sg"]:  # one call per shape tried (unit/braced struct, plain/method call)
        out, t = run(["ast-grep", "run", "--lang", "rust", "-p", pat, "--json=compact", "crates", "src"])
        outs += out; ms += t
        for h in json.loads(out or "[]"):
            if h["file"] not in files: files.append(h["file"])
    return files, outs, ms, len(q["sg"])

DEF = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:fn|struct|enum|trait)\s+(\w+)", re.M)
MAP = None
def repomap(q):
    """Empryo/aider-style map: file -> defined symbols; score query tokens vs path+symbol tokens."""
    global MAP
    t = time.monotonic()
    if MAP is None:
        MAP = {}
        for f in subprocess.check_output(["git", "ls-files", "*.rs"], cwd=ROOT, text=True).split():
            try: MAP[f] = DEF.findall(open(os.path.join(ROOT, f), errors="ignore").read())
            except OSError: pass
    toks = [w.lower() for w in re.findall(r"[A-Za-z]+", q["q"]) if w.lower() not in STOP]
    split = lambda s: [w.lower() for w in re.findall(r"[A-Z]?[a-z]+|[A-Z]+(?![a-z])", s)]
    scores = []
    for f, syms in MAP.items():
        if "/tests" in f or f.endswith("_tests.rs"): pen = 0.5
        else: pen = 1.0
        pt = set(split(f.replace("/", " ").replace("_", " ")))
        st = set(w for s in syms for w in split(s.replace("_", " ")))
        exact = q.get("sym") in syms if q.get("sym") else False
        sc = (sum(2 for w in toks if w in pt) + sum(1 for w in toks if w in st) + (10 if exact else 0)) * pen
        if sc: scores.append((sc, f))
    scores.sort(key=lambda x: -x[0]); ranked = [f for _, f in scores[:10]]
    return ranked, "\n".join(ranked), (time.monotonic() - t) * 1000, 1

STRATS = {"agentgrep": agentgrep if AG else None, "graft": graft if shutil.which("graft") else None,
          "ast-grep": astgrep if shutil.which("ast-grep") else None, "repo-map": repomap}
NOT_RUN = {"search-subagent": "not run: needs ~20 paid LLM agent sessions with one pinned model; no model budget authorized in this change"}

qs = json.load(open(os.path.join(HERE, "queries.json")))
for q in qs:
    for a in q["answers"]: assert os.path.exists(os.path.join(ROOT, a)), a
rows = []
for name, fn in STRATS.items():
    if fn is None: rows.append({"strategy": name, "status": "not run: binary missing"}); continue
    for q in qs:
        r = fn(q)
        if r is None: rows.append({"strategy": name, "query": q["id"], "status": "n/a: no structural pattern"}); continue
        ranked, out, ms, calls = r
        hit = lambda k: int(any(a in ranked[:k] for a in q["answers"]))
        rows.append({"strategy": name, "query": q["id"], "kind": q["kind"], "tool_calls": calls,
                     "tokens": len(out) // 4, "r1": hit(1), "r5": hit(5), "wall_ms": round(ms), "top": ranked[:3]})
for k, v in NOT_RUN.items(): rows.append({"strategy": k, "status": v})
json.dump({"commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(), "rows": rows}, sys.stdout, indent=1)
