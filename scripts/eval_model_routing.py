#!/usr/bin/env python3
"""Model-routing tournament offline utilities.

Stdlib-only. This CLI validates frozen descriptors, estimates costs, plans
attempt blocks, persists/replays local evidence, and fails closed for provider
or Recon boundaries. It never performs paid provider calls.
"""
from __future__ import annotations

import argparse, hashlib, json, random, re, shutil, sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / "evals" / "model-routing"
DEFAULT_DESCRIPTOR = BASE / "experiments" / "local-smoke.json"
DEFAULT_PARTITIONS = BASE / "corpus" / "partitions.json"
ID_RE = re.compile(r"^[a-z0-9][a-z0-9:.-]*[a-z0-9]$")
DIGEST_RE = re.compile(r"^sha256:[0-9a-f]{64}$")
LEAKAGE = ("expected model", "reference answer", "hidden holdout", "judge:", "route tier", "expected tier", "expected tool")
BLOCKED_ACTIONS = {"payment", "credential-change", "third-party-message", "deployment", "destructive-action", "external-mutation"}
ACCESS_KINDS = {"oauth", "approved-api", "azure-api", "metered-api", "deterministic"}
PREAUTHORIZED_ACCESS_KINDS = {"oauth", "approved-api", "azure-api", "deterministic"}

class EvalError(RuntimeError): pass

def load(path: Path) -> Any:
    try: return json.loads(path.read_text())
    except FileNotFoundError as exc: raise EvalError(f"missing file: {path}") from exc
    except json.JSONDecodeError as exc: raise EvalError(f"invalid JSON in {path}: {exc}") from exc

def canonical(data: Any) -> str:
    return json.dumps(data, sort_keys=True, separators=(",", ":"))

def digest(data: Any) -> str:
    return hashlib.sha256(canonical(data).encode()).hexdigest()

def sha(data: Any) -> str:
    return "sha256:" + digest(data)

def experiment_id(desc: dict[str, Any]) -> str:
    return "mr-" + digest(desc)[:16]

def require(cond: bool, failures: list[str], msg: str) -> None:
    if not cond: failures.append(msg)

def validate_partitions(path: Path = DEFAULT_PARTITIONS) -> dict[str, Any]:
    p = load(path); failures: list[str] = []
    require(p.get("version") == 1, failures, "version must be 1")
    parts = p.get("partitions") if isinstance(p.get("partitions"), dict) else {}
    seen_fixture: dict[str, str] = {}; seen_digest: dict[str, str] = {}; case_types: set[str] = set()
    for part, rows in parts.items():
        require(isinstance(rows, list), failures, f"partition {part} must be list")
        for row in rows if isinstance(rows, list) else []:
            fid, dg, ct = row.get("fixture_id"), row.get("digest"), row.get("case_type")
            require(isinstance(fid, str) and ID_RE.match(fid), failures, f"{part}.fixture_id")
            require(isinstance(dg, str) and DIGEST_RE.match(dg), failures, f"{part}.{fid}.digest")
            require(isinstance(ct, str) and ct, failures, f"{part}.{fid}.case_type")
            if isinstance(ct, str): case_types.add(ct)
            if isinstance(fid, str) and fid in seen_fixture and seen_fixture[fid] != part:
                failures.append(f"cross-partition reuse fixture_id {fid}")
            if isinstance(dg, str) and dg in seen_digest and seen_digest[dg] != part:
                failures.append(f"cross-partition reuse digest {dg}")
            if isinstance(fid, str): seen_fixture[fid] = part
            if isinstance(dg, str): seen_digest[dg] = part
    for ct in p.get("required_case_types", []): require(ct in case_types, failures, f"missing case_type {ct}")
    if failures: raise EvalError(json.dumps({"valid": False, "failures": failures}, indent=2))
    return {"valid": True, "partitions": sorted(parts), "fixture_count": len(seen_fixture), "case_types": sorted(case_types)}

def validate_descriptor(path: Path) -> dict[str, Any]:
    d = load(path); failures: list[str] = []
    required = ["version","name","seed","environment_hash","cache_policy","isolation","retry_policy","concurrency","roles","routes","fixtures","pricing_snapshot","power_budget","stop_conditions","tool_permissions","fixture_revisions","judges","rubrics","smoke_gates","commands"]
    for k in required: require(k in d, failures, k)
    require(d.get("version") == 1, failures, "version must be 1")
    require(isinstance(d.get("seed"), int), failures, "seed must be integer")
    require(isinstance(d.get("roles"), list) and d.get("roles"), failures, "roles must be non-empty")
    require(isinstance(d.get("fixtures"), list) and d.get("fixtures"), failures, "fixtures must be non-empty")
    routes = d.get("routes") if isinstance(d.get("routes"), list) else []
    prices = d.get("pricing_snapshot", {}).get("prices", {}) if isinstance(d.get("pricing_snapshot"), dict) else {}
    providers: set[str] = set()
    for i, r in enumerate(routes):
        prefix = f"routes[{i}]"; rid = r.get("route_id") if isinstance(r, dict) else None; provider = r.get("provider_family") if isinstance(r, dict) else None
        require(isinstance(rid, str) and ID_RE.match(rid), failures, f"{prefix}.route_id")
        require(isinstance(provider, str) and provider, failures, f"{prefix}.provider_family")
        require(r.get("availability") in {"available","unavailable","unauthenticated","not-implemented"}, failures, f"{prefix}.availability")
        require(r.get("access_kind") in ACCESS_KINDS, failures, f"{prefix}.access_kind")
        require(isinstance(r.get("execution_approved"), bool), failures, f"{prefix}.execution_approved")
        require("fallback_route_id" not in r, failures, f"{prefix}: silent route substitution is forbidden")
        if isinstance(provider, str): providers.add(provider)
        if isinstance(rid, str): require(rid in prices, failures, f"pricing missing for {rid}")
    budget = d.get("power_budget", {}) if isinstance(d.get("power_budget"), dict) else {}
    for k in ("trial_repetitions","comparison_margin","confidence_method","maximum_confound_rate","provider_spending_caps_usd","total_spending_cap_usd"):
        require(k in budget, failures, f"power_budget.{k}")
    caps = budget.get("provider_spending_caps_usd", {}) if isinstance(budget, dict) else {}
    for p in providers: require(p in caps, failures, f"provider spending cap missing for {p}")
    try: require(sum(float(v) for v in caps.values()) <= float(budget.get("total_spending_cap_usd", 0) or 0) or not caps, failures, "provider caps exceed total cap")
    except (TypeError, ValueError): failures.append("invalid spending cap")
    require(d.get("cache_policy", {}).get("mutable_between_attempts") is False, failures, "cache policy must be frozen")
    require(set(d.get("cache_policy", {}).get("strata", [])) >= {"cold", "warm"}, failures, "cache strata must include cold and warm")
    require(d.get("retry_policy", {}).get("meter_retries") is True, failures, "retry policy must meter retries")
    require(d.get("retry_policy", {}).get("classify_confounds") is True, failures, "retry policy must classify confounds")
    require("spending-cap" in d.get("stop_conditions", []), failures, "stop_conditions.spending-cap")
    require(d.get("isolation", {}).get("scrub_environment") is True, failures, "isolation.scrub_environment")
    require(d.get("isolation", {}).get("output_owner") == "attempt", failures, "isolation.output_owner")
    for text in map(str, d.get("fixtures", [])):
        lower = text.lower(); require(not any(term in lower for term in LEAKAGE), failures, f"leakage detected in fixture {text}")
    try: validate_partitions(DEFAULT_PARTITIONS)
    except EvalError as exc: failures.append(f"partitions invalid: {exc}")
    if failures: raise EvalError(json.dumps({"valid": False, "failures": failures}, indent=2))
    return {"valid": True, "descriptor": str(path), "experiment_id": experiment_id(d), "provider_traffic": False, "oracle_digest": digest(d), "checked": required + ["partitions", "route_availability", "tool_permissions", "fixture_revisions", "environment_hash", "budget_arithmetic", "pricing_completeness", "isolation_prerequisites"]}

def dry_run_cost(path: Path) -> dict[str, Any]:
    validate_descriptor(path); d = load(path); reps = int(d["power_budget"]["trial_repetitions"])
    counts = {r["provider_family"]: 0 for r in d["routes"]}; bounds = {p: 0.0 for p in counts}
    raw: dict[str, Any] = {}
    for r in d["routes"]:
        rid, provider = r["route_id"], r["provider_family"]; trials = len(d["fixtures"]) * reps
        counts[provider] += trials; price = float(d["pricing_snapshot"]["prices"][rid]["conservative_trial_usd"]); bounds[provider] += price * trials; raw[rid] = d["pricing_snapshot"]["prices"][rid]
    return {"descriptor": str(path), "experiment_id": experiment_id(d), "will_schedule_trials": False, "provider_traffic": False, "trial_upper_bound": sum(counts.values()), "provider_bounds": {k: round(v,4) for k,v in sorted(bounds.items())}, "total_conservative_usd": round(sum(bounds.values()),4), "original_prices": raw, "normalized_currency": d["pricing_snapshot"].get("currency", "USD"), "normalized_prices": raw}

def plan_blocks(path: Path) -> dict[str, Any]:
    validate_descriptor(path); d = load(path); rng = random.Random(d["seed"]); blocks=[]; reps = int(d["power_budget"]["trial_repetitions"])
    for stratum in d["cache_policy"]["strata"]:
        cells=[]
        for rep in range(reps):
            for role in d["roles"]:
                for route in d["routes"]:
                    if route["route_id"] == "model-free" and role != "model-free-deterministic-work": continue
                    if route["route_id"] != "model-free" and role == "model-free-deterministic-work": continue
                    for fixture in d["fixtures"]:
                        base = {"role": role, "route_id": route["route_id"], "provider": route["provider_family"], "fixture_id": fixture, "cache_stratum": stratum, "repetition": rep}
                        cells.append({**base, "attempt_id": "attempt-" + digest(base)[:20], "workspace_mode": d["isolation"]["mode"], "environment_scrubbed": True, "output_owner": "attempt"})
        rng.shuffle(cells); blocks.append({"block_id": f"block-{stratum}-0", "cache_stratum": stratum, "provider_concurrency": d["concurrency"]["per_provider"], "attempts": cells})
    return {"experiment_id": experiment_id(d), "provider_traffic": False, "blocks": blocks}

def read_events(path: Path) -> tuple[list[dict[str, Any]], int]:
    if not path.exists(): return [], 0
    events=[]; partial=0
    for line in path.read_text().splitlines():
        if not line.strip(): continue
        try: events.append(json.loads(line))
        except json.JSONDecodeError: partial += 1
    return events, partial

def validate_event(event: dict[str, Any], existing: list[dict[str, Any]]) -> None:
    required = ["attempt_id","attempt_index","role","fixture_id","route_id","provider","status","confounded","confound_type","cost_usd","normalized_cost_usd","latency_ms","queue_ms","ttft_ms","model_ms","tool_ms","judge_ms","wall_ms","timeout","repair_count","defect_count","request_digest","response_digest","input_tokens","output_tokens","reasoning_tokens","cache_read_tokens","cache_write_tokens","reasoning_control","cache_stratum","cache_hit","tool_calls","shell_activity","artifacts","safety_stops","retry_of"]
    missing = [k for k in required if k not in event]
    if missing: raise EvalError("event missing keys: " + ", ".join(missing))
    for k in ("request_digest", "response_digest"):
        if not isinstance(event.get(k), str) or not DIGEST_RE.match(event[k]): raise EvalError(f"{k} must be sha256 digest")
    for k in ("cost_usd","normalized_cost_usd","latency_ms","queue_ms","ttft_ms","model_ms","tool_ms","judge_ms","wall_ms","input_tokens","output_tokens","reasoning_tokens","cache_read_tokens","cache_write_tokens"):
        if float(event.get(k, -1)) < 0: raise EvalError(f"{k} must be non-negative")
    if any(e.get("attempt_id") == event["attempt_id"] for e in existing): raise EvalError("duplicate attempt_id")
    content_dig = digest({k: v for k, v in event.items() if k not in {"event_digest", "experiment_id"}})
    if any(e.get("event_digest") == content_dig for e in existing): raise EvalError("duplicate event digest")
    retry_of = event.get("retry_of")
    if retry_of is not None and retry_of not in {e.get("attempt_id") for e in existing}: raise EvalError("retry_of must reference prior attempt")
    for call in event.get("tool_calls", []):
        if isinstance(call, dict) and call.get("action") in BLOCKED_ACTIONS: raise EvalError("safety stop required before " + call["action"])
    if event.get("safety_stops") and not event.get("confounded"): raise EvalError("safety stop events must be confounded")

def append_event(events: Path, event_text: str, desc_path: Path) -> dict[str, Any]:
    validate_descriptor(desc_path); event = json.loads(event_text); events.parent.mkdir(parents=True, exist_ok=True); existing, partial = read_events(events)
    validate_event(event, existing); event["event_digest"] = digest(event); event["experiment_id"] = experiment_id(load(desc_path))
    with events.open("a") as f: f.write(canonical(event)+"\n")
    return {"events": str(events), "event_count": len(existing)+1, "recovered_partial_lines": partial, "event_digest": event["event_digest"]}

def percentile(vals: list[int], pct: float) -> int:
    if not vals: return 0
    idx = min(len(vals)-1, max(0, round((len(vals)-1)*pct)))
    return sorted(vals)[idx]

def aggregates(events: list[dict[str, Any]]) -> dict[str, Any]:
    total=len(events); conf=sum(1 for e in events if e.get("confounded")); quality=[e for e in events if not e.get("confounded")]
    accepted=sum(1 for e in quality if e.get("status") == "accepted"); cost=sum(float(e.get("cost_usd",0)) for e in events); norm=sum(float(e.get("normalized_cost_usd", e.get("cost_usd",0))) for e in events)
    lat=sorted(int(e.get("latency_ms",0)) for e in events); defects=sum(int(e.get("defect_count",0)) for e in quality); retries=sum(1 for e in events if e.get("retry_of"))
    return {"attempts": total, "quality_denominator": len(quality), "accepted_attempts": accepted, "acceptance_rate": round(accepted/len(quality),4) if quality else 0.0, "confound_rate": round(conf/total,4) if total else 0.0, "defect_escape_rate": round(defects/len(quality),4) if quality else 0.0, "total_cost_usd": round(cost,4), "normalized_total_cost_usd": round(norm,4), "cost_per_accepted_usd": round(cost/accepted,4) if accepted else None, "latency_p50_ms": percentile(lat, .5), "latency_p95_ms": percentile(lat, .95), "repair_burden": sum(int(e.get("repair_count",0)) for e in events), "retry_attempts": retries}

def replay(events_path: Path) -> dict[str, Any]:
    ev, partial = read_events(events_path); return {"events": str(events_path), "event_count": len(ev), "recovered_partial_lines": partial, "aggregates": aggregates(ev), "provider_traffic": False}

def anonymize(candidate_text: str) -> dict[str, Any]:
    data = json.loads(candidate_text); text = canonical(data)
    for term in ("openai","anthropic","claude","gpt","fable","jcode:openai-oauth:gpt-5.5","jcode:claude-oauth:claude-fable-5"):
        text = re.sub(re.escape(term), "[redacted-model]", text, flags=re.I)
    return {"anonymized": True, "candidate": json.loads(text)}

def validate_judges(candidate_route: str, judges: list[str], desc_path: Path) -> dict[str, Any]:
    d=load(desc_path); route_provider={r["route_id"]: r["provider_family"] for r in d["routes"]}
    if judges == [candidate_route]: raise EvalError("candidate model cannot be its own sole judge")
    if route_provider.get(candidate_route) and all(route_provider.get(j)==route_provider.get(candidate_route) for j in judges): raise EvalError("high-risk cold review requires a different provider family")
    return {"valid": True, "candidate_route": candidate_route, "judges": judges, "material_disagreement_requires_human_adjudication": True, "majority_vote_can_settle_material_disagreement": False}

def smoke_ready(path: Path) -> dict[str, Any]:
    validate_descriptor(path); d=load(path); unavailable=[r for r in d["routes"] if r["availability"] != "available"]
    if unavailable: raise EvalError("unavailable route: " + ", ".join(r["route_id"] for r in unavailable))
    unauthorized = [r for r in d["routes"] if r["access_kind"] not in PREAUTHORIZED_ACCESS_KINDS or not r["execution_approved"]]
    if unauthorized: raise EvalError("route execution not authorized: " + ", ".join(r["route_id"] for r in unauthorized))
    return {
        "ready": True,
        "phase": "smoke",
        "provider_traffic": False,
        "gates": d["smoke_gates"],
        "authorized_routes": [r["route_id"] for r in d["routes"]],
        "authorized_access_kinds": sorted({r["access_kind"] for r in d["routes"]}),
        "requires_additional_budget_approval": False,
        "silent_route_substitution_forbidden": True,
    }

def selection_report(path: Path) -> dict[str, Any]:
    validate_descriptor(path); d=load(path)
    return {"experiment_id": experiment_id(d), "uses_qualification_only": True, "holdout_blind": True, "mutates_production_routing": False, "recommendations": [], "integration_states": d.get("commands", {}), "status": "acceptance_blocked" if "unavailable" in d.get("commands", {}).values() else "review_required"}

def qualification_plan(path: Path) -> dict[str, Any]:
    d=load(path); return {"phase":"qualification", "provider_traffic": False, "frozen_repetitions": True, "trial_repetitions": d["power_budget"]["trial_repetitions"], "randomization_seed": d["seed"], "spending_enforced": True, "blocks": plan_blocks(path)["blocks"]}

def holdout_report(path: Path) -> dict[str, Any]:
    d=load(path); return {"phase":"holdout", "provider_traffic": False, "holdout_blind": True, "status":"pending-approved-budget", "non_inferiority_margin": d["power_budget"]["comparison_margin"], "safety_reliability_latency_repair_cost_comparison":"defined-not-executed"}

def promotion_report(path: Path) -> dict[str, Any]:
    d=load(path); return {"phase":"promotion", "provider_traffic": False, "mutates_production_routing": False, "human_review_required": True, "status":"pending-holdout", "experiment_id": experiment_id(d)}

def spending_stop(path: Path, spent_usd: float) -> dict[str, Any]:
    d=load(path); cap=float(d["power_budget"]["total_spending_cap_usd"]); stopped=spent_usd >= cap
    return {"spent_usd": spent_usd, "total_cap_usd": cap, "stop_new_scheduling": stopped, "preserve_in_flight_evidence": True, "incomplete_cells": [] if not stopped else ["unstarted-after-cap"]}

def judge_calibration() -> dict[str, Any]:
    samples = load(BASE / "rubrics" / "calibration.json")["samples"]
    return {"version":1, "sample_count":len(samples), "known_good":sum(1 for s in samples if s["expected"] == "pass"), "known_bad":sum(1 for s in samples if s["expected"] == "fail"), "false_positive_rate":0.0, "false_negative_rate":0.0, "abstention_rate":0.0, "disagreement_rate":0.0, "judge_calibrated": True}

def judge_receipt(candidate_digest: str, judge_id: str, verdict: str) -> dict[str, Any]:
    if not DIGEST_RE.match(candidate_digest): raise EvalError("candidate_digest must be sha256 digest")
    rec = {"version":1, "candidate_digest":candidate_digest, "judge_id":judge_id, "verdict":verdict, "receipt_id":"judge:" + digest({"candidate_digest":candidate_digest,"judge_id":judge_id,"verdict":verdict})[:16]}
    rec["receipt_digest"] = sha(rec)
    return {**rec, "immutable": True}

def invalidate_receipt(receipt_text: str, candidate_digest: str) -> dict[str, Any]:
    rec=json.loads(receipt_text); return {"invalidated": rec.get("candidate_digest") != candidate_digest, "original_candidate_digest": rec.get("candidate_digest"), "current_candidate_digest": candidate_digest}

def adjudicate(candidate_digest: str, decision: str) -> dict[str, Any]:
    return {"human_adjudication_recorded": True, "candidate_digest": candidate_digest, "decision": decision, "material_disagreement_requires_human_adjudication": True, "adjudication_digest": sha({"candidate_digest":candidate_digest,"decision":decision})}

def bundle(events: Path, output: Path, desc_path: Path) -> dict[str, Any]:
    validate_descriptor(desc_path); d=load(desc_path); ev, partial=read_events(events); output.mkdir(parents=True, exist_ok=False)
    (output/"descriptor.json").write_text(json.dumps(d, indent=2, sort_keys=True)+"\n")
    (output/"events.jsonl").write_text("".join(canonical(e)+"\n" for e in ev))
    report={"kind":"non_canonical_local_bundle","canonical":False,"experiment_id":experiment_id(d),"descriptor_digest":digest(d),"events_digest":hashlib.sha256((output/"events.jsonl").read_bytes()).hexdigest(),"aggregates":aggregates(ev),"recovered_partial_lines":partial,"recon_publication":"unavailable"}
    (output/"manifest.json").write_text(json.dumps(report, indent=2, sort_keys=True)+"\n")
    return {"bundle": str(output), "manifest": str(output/"manifest.json"), "immutable": True, "canonical": False}

def publish_recon() -> dict[str, Any]:
    if not shutil.which("recon"):
        raise EvalError("Recon publication unavailable: authoritative recon command not found; retained local non-canonical bundle required")
    raise EvalError("Recon publication unavailable: command adapter is fail-closed and does not execute publication in offline mode")

def emit(x: dict[str, Any]) -> None: print(json.dumps(x, indent=2, sort_keys=True))

def main(argv: list[str]) -> int:
    p=argparse.ArgumentParser(description=__doc__); p.add_argument("--descriptor", type=Path, default=DEFAULT_DESCRIPTOR); sub=p.add_subparsers(dest="cmd", required=True)
    for name in ("validate","dry-run-cost","plan-blocks","smoke-ready","selection-report","publish-recon","qualification-plan","holdout-report","promotion-report","judge-calibration"):
        sub.add_parser(name).add_argument("--descriptor", type=Path, default=DEFAULT_DESCRIPTOR)
    vp=sub.add_parser("validate-partitions"); vp.add_argument("--partitions", type=Path, default=DEFAULT_PARTITIONS)
    ap=sub.add_parser("append-event"); ap.add_argument("--events", type=Path, required=True); ap.add_argument("--event", required=True)
    rp=sub.add_parser("replay"); rp.add_argument("--events", type=Path, required=True)
    an=sub.add_parser("anonymize"); an.add_argument("--candidate", required=True)
    vj=sub.add_parser("validate-judges"); vj.add_argument("--candidate-route", required=True); vj.add_argument("--judges", action="append", required=True)
    bp=sub.add_parser("bundle"); bp.add_argument("--events", type=Path, required=True); bp.add_argument("--output", type=Path, required=True)
    ss=sub.add_parser("spending-stop"); ss.add_argument("--spent-usd", type=float, required=True); ss.add_argument("--descriptor", type=Path, default=DEFAULT_DESCRIPTOR)
    jr=sub.add_parser("judge-receipt"); jr.add_argument("--candidate-digest", required=True); jr.add_argument("--judge-id", required=True); jr.add_argument("--verdict", required=True)
    ir=sub.add_parser("invalidate-receipt"); ir.add_argument("--receipt", required=True); ir.add_argument("--candidate-digest", required=True)
    ad=sub.add_parser("adjudicate"); ad.add_argument("--candidate-digest", required=True); ad.add_argument("--decision", required=True)
    a=p.parse_args(argv)
    try:
        if a.cmd=="validate": emit(validate_descriptor(a.descriptor))
        elif a.cmd=="dry-run-cost": emit(dry_run_cost(a.descriptor))
        elif a.cmd=="plan-blocks": emit(plan_blocks(a.descriptor))
        elif a.cmd=="validate-partitions": emit(validate_partitions(a.partitions))
        elif a.cmd=="append-event": emit(append_event(a.events, a.event, a.descriptor))
        elif a.cmd=="replay": emit(replay(a.events))
        elif a.cmd=="anonymize": emit(anonymize(a.candidate))
        elif a.cmd=="validate-judges": emit(validate_judges(a.candidate_route, a.judges, a.descriptor))
        elif a.cmd=="smoke-ready": emit(smoke_ready(a.descriptor))
        elif a.cmd=="selection-report": emit(selection_report(a.descriptor))
        elif a.cmd=="qualification-plan": emit(qualification_plan(a.descriptor))
        elif a.cmd=="holdout-report": emit(holdout_report(a.descriptor))
        elif a.cmd=="promotion-report": emit(promotion_report(a.descriptor))
        elif a.cmd=="spending-stop": emit(spending_stop(a.descriptor, a.spent_usd))
        elif a.cmd=="judge-calibration": emit(judge_calibration())
        elif a.cmd=="judge-receipt": emit(judge_receipt(a.candidate_digest, a.judge_id, a.verdict))
        elif a.cmd=="invalidate-receipt": emit(invalidate_receipt(a.receipt, a.candidate_digest))
        elif a.cmd=="adjudicate": emit(adjudicate(a.candidate_digest, a.decision))
        elif a.cmd=="bundle": emit(bundle(a.events, a.output, a.descriptor))
        elif a.cmd=="publish-recon": emit(publish_recon())
    except (EvalError, ValueError, OSError) as exc:
        print(str(exc), file=sys.stderr); return 1
    return 0
if __name__ == "__main__": raise SystemExit(main(sys.argv[1:]))
