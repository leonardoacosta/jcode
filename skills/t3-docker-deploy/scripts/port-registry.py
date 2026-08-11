#!/usr/bin/env python3
"""Fleet service-port registry for t3-docker homelab apps.

The registry (references/port-registry.json next to this script) is the single
source of truth for which app owns which host-port block. Every t3-docker app
gets a contiguous 10-port DEV block in the 5000s and a mirrored PROD block in
the 6000s (prod = dev + 1000). Within a block, services get fixed offsets:
web = base+0, postgres = base+1, then +2, +3 ... for extra services (redis,
workers, etc.). Container-internal ports never change (web 3000, postgres 5432)
— only the HOST-published port follows the scheme.

Why a script instead of hand-editing JSON: allocation must be collision-free
and deterministic. The script picks the lowest free block index, so two people
(or two agents) registering apps can't accidentally hand out the same ports.

Commands:
  list                       Human table of every registered app + its ports.
  show <code>                One app's block + per-service dev/prod/container ports.
  next                       The block index + dev/prod ranges that allocate would use.
  domains                    Full subdomain log: every service (apps + infra) ->
                             subdomain, backend, ingress mechanism, status.
  allocate <code> [opts]     Reserve the next free block for <code> and write the registry.
      --name "<display>"     App display name.
      --domain <host>        Traefik subdomain (e.g. tc.leonardoacosta.dev).
      --services a,b,c       Service names in offset order (default: web,postgres).
      --dry-run              Print what would be allocated; do not write.

Ingress preference (see SKILL.md § Ingress & DNS): Traefik owns :443 with the
Cloudflare DNS-01 wildcard cert and Host-routes *.leonardoacosta.dev; tailscale
serve is the .ts.net fallback. The `ingress` + `infra_services` keys in the
registry are the source of truth for the subdomain map; hand-edit infra_services.

All commands accept --json for machine-readable output and exit 0 on expected
errors (emitting an {"error": ...} object) so callers never abort mid-pipeline.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

REGISTRY = Path(__file__).resolve().parent.parent / "references" / "port-registry.json"


def load() -> dict:
    return json.loads(REGISTRY.read_text(encoding="utf-8"))


def save(data: dict) -> None:
    # Atomic-ish write: tmp then replace, so a crash can't truncate the registry.
    tmp = REGISTRY.with_suffix(".json.tmp")
    tmp.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    tmp.replace(REGISTRY)


def block_for(scheme: dict, index: int) -> tuple[list[int], list[int]]:
    size = scheme["block_size"]
    dev_lo = scheme["dev_base"] + size * index
    prod_lo = scheme["prod_base"] + size * index
    return [dev_lo, dev_lo + size - 1], [prod_lo, prod_lo + size - 1]


def next_free_index(data: dict) -> int:
    used = {a["index"] for a in data["apps"]}
    i = 0
    while i in used:
        i += 1
    return i


def emit(obj, as_json: bool, human) -> int:
    if as_json:
        print(json.dumps(obj, indent=2))
    else:
        human(obj)
    return 0


def cmd_list(data, as_json):
    rows = []
    for a in sorted(data["apps"], key=lambda x: x["index"]):
        for svc, p in a["services"].items():
            rows.append((a["code"], a["index"], svc, p["dev"], p["prod"], p["container"]))

    def human(_):
        if not rows:
            print("registry empty — no apps allocated yet")
            return
        print(f"{'app':6} {'idx':3} {'service':10} {'dev':6} {'prod':6} {'container':9}")
        print("-" * 48)
        for code, idx, svc, dev, prod, cont in rows:
            print(f"{code:6} {idx:<3} {svc:10} {dev:<6} {prod:<6} {cont:<9}")

    return emit({"apps": data["apps"]}, as_json, human)


def cmd_show(data, code, as_json):
    app = next((a for a in data["apps"] if a["code"] == code), None)
    if app is None:
        return emit({"error": f"app '{code}' not registered"}, as_json,
                    lambda o: print(o["error"], file=sys.stderr) or None)

    def human(_):
        print(f"{app['code']} ({app.get('name', '')}) — index {app['index']}")
        print(f"  domain:    {app.get('domain', '-')}")
        print(f"  dev block: {app['dev_block'][0]}-{app['dev_block'][1]}")
        print(f"  prod block:{app['prod_block'][0]}-{app['prod_block'][1]}")
        for svc, p in app["services"].items():
            print(f"  {svc:10} dev {p['dev']} -> prod {p['prod']} (container {p['container']})")

    return emit(app, as_json, human)


def cmd_domains(data, as_json):
    # Unified subdomain log: T3 apps (subdomain = their `domain`, backed by the
    # prod web port) plus the hand-maintained infra_services entries.
    rows = []
    for a in sorted(data.get("apps", []), key=lambda x: x.get("index", 0)):
        web = a.get("services", {}).get("web")
        backend = f"http://172.20.0.1:{web['prod']}" if web else "-"
        rows.append({
            "code": a["code"], "name": a.get("name", a["code"]),
            "subdomain": a.get("domain", "-"), "backend": backend,
            "ingress": "traefik", "status": "active", "kind": "app",
        })
    for s in data.get("infra_services", []):
        rows.append({
            "code": s["code"], "name": s.get("name", s["code"]),
            "subdomain": s.get("subdomain", "-"), "backend": s.get("backend", "-"),
            "ingress": s.get("ingress", "-"), "status": s.get("status", "-"),
            "kind": "infra", "notes": s.get("notes", ""),
        })
    rows.sort(key=lambda r: r["subdomain"])

    def human(_):
        print(f"{'subdomain':34} {'service':18} {'ingress':14} {'status':11} backend")
        print("-" * 100)
        for r in rows:
            print(f"{r['subdomain']:34} {r['name']:18} {r['ingress']:14} "
                  f"{r['status']:11} {r['backend']}")

    return emit({"domain": data.get("ingress", {}).get("domain"),
                 "services": rows}, as_json, human)


def cmd_next(data, as_json):
    idx = next_free_index(data)
    dev, prod = block_for(data["scheme"], idx)
    obj = {"index": idx, "dev_block": dev, "prod_block": prod}
    return emit(obj, as_json,
                lambda o: print(f"next free block: index {o['index']} — "
                                f"dev {o['dev_block'][0]}-{o['dev_block'][1]}, "
                                f"prod {o['prod_block'][0]}-{o['prod_block'][1]}"))


def cmd_allocate(data, args, as_json):
    code = args.code
    if any(a["code"] == code for a in data["apps"]):
        return emit({"error": f"app '{code}' already registered — use show/edit, not allocate"},
                    as_json, lambda o: print(o["error"], file=sys.stderr) or None)

    scheme = data["scheme"]
    idx = next_free_index(data)
    dev_block, prod_block = block_for(scheme, idx)
    services = [s.strip() for s in args.services.split(",") if s.strip()]
    if len(services) > scheme["block_size"]:
        return emit({"error": f"{len(services)} services exceeds block size {scheme['block_size']}"},
                    as_json, lambda o: print(o["error"], file=sys.stderr) or None)

    svc_map = {}
    for offset, svc in enumerate(services):
        dev_port = dev_block[0] + offset
        prod_port = dev_port + scheme["prod_offset"]
        # Container-internal port: by default the service binds its prod host port
        # (1:1 host:container, e.g. 6000:6000). `container_ports` lists the only
        # fixed overrides — postgres always binds 5432. We never use 3000 (Grafana).
        svc_map[svc] = {
            "dev": dev_port,
            "prod": prod_port,
            "container": scheme["container_ports"].get(svc, prod_port),
        }

    entry = {
        "code": code,
        "name": args.name or code,
        "index": idx,
        "domain": args.domain or f"{code}.leonardoacosta.dev",
        "dev_block": dev_block,
        "prod_block": prod_block,
        "services": svc_map,
    }

    if args.dry_run:
        return emit({"would_allocate": entry}, as_json,
                    lambda o: cmd_show({"apps": [entry]}, code, False))

    data["apps"].append(entry)
    save(data)
    return emit(entry, as_json, lambda o: cmd_show({"apps": [entry]}, code, False))


def main() -> int:
    p = argparse.ArgumentParser(description="t3-docker fleet service-port registry")
    p.add_argument("--json", action="store_true", help="machine-readable output")
    sub = p.add_subparsers(dest="cmd", required=True)
    sub.add_parser("list")
    s_show = sub.add_parser("show")
    s_show.add_argument("code")
    sub.add_parser("next")
    sub.add_parser("domains")
    s_alloc = sub.add_parser("allocate")
    s_alloc.add_argument("code")
    s_alloc.add_argument("--name", default="")
    s_alloc.add_argument("--domain", default="")
    s_alloc.add_argument("--services", default="web,postgres")
    s_alloc.add_argument("--dry-run", action="store_true")
    args = p.parse_args()

    data = load()
    if args.cmd == "list":
        return cmd_list(data, args.json)
    if args.cmd == "show":
        return cmd_show(data, args.code, args.json)
    if args.cmd == "next":
        return cmd_next(data, args.json)
    if args.cmd == "domains":
        return cmd_domains(data, args.json)
    if args.cmd == "allocate":
        return cmd_allocate(data, args, args.json)
    return 1


if __name__ == "__main__":
    sys.exit(main())
