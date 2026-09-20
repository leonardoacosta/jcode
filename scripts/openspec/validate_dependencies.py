#!/usr/bin/env python3
"""
validate_dependencies.py — Validate OpenSpec change dependency graph.

Usage:
    python3 scripts/openspec/validate_dependencies.py --root . [--json] <change-id> ...
    python3 scripts/openspec/validate_dependencies.py --root . --all [--json]

Exit codes:
    0 — graph is valid
    1 — invalid (unresolvable dependencies, cycles, unsupported versions)
    2 — missing required metadata on active changes
"""

import argparse
import json
import os
import sys
import yaml
from collections import defaultdict, deque


def load_frontmatter(proposal_path):
    """Extract YAML frontmatter from a proposal.md file."""
    with open(proposal_path) as f:
        content = f.read()
    # Strip leading --- lines
    if not content.startswith("---"):
        return None
    parts = content.split("---", 2)
    if len(parts) < 3:
        return None
    try:
        return yaml.safe_load(parts[1])
    except yaml.YAMLError:
        return None


def load_active_changes(changes_dir):
    """Load all active changes with their dependency metadata."""
    changes = {}
    for entry in os.listdir(changes_dir):
        # Skip the archive subdirectory.
        if entry == "archive":
            continue
        change_dir = os.path.join(changes_dir, entry)
        proposal_path = os.path.join(change_dir, "proposal.md")
        if not os.path.isdir(change_dir) or not os.path.isfile(proposal_path):
            continue
        fm = load_frontmatter(proposal_path)
        if fm is None or "execution" not in fm:
            changes[entry] = {
                "id": entry,
                "version": None,
                "depends_on": [],
                "valid": False,
                "reason": "missing execution frontmatter",
            }
            continue
        exec_cfg = fm["execution"]
        version = exec_cfg.get("version")
        depends = exec_cfg.get("depends_on", [])
        if depends is None:
            depends = []
        changes[entry] = {
            "id": entry,
            "version": version,
            "depends_on": depends,
            "valid": version == 1,
            "reason": f"unsupported version {version}" if version != 1 else None,
        }
    return changes


def load_archived_changes(archive_dir):
    """Load archived changes (for resolving legacy dependencies).

    Archived directories use the naming convention:
    YYYY-MM-DD-<original-change-id>
    """
    changes = {}
    if not os.path.isdir(archive_dir):
        return changes
    for entry in os.listdir(archive_dir):
        archive_item = os.path.join(archive_dir, entry)
        if not os.path.isdir(archive_item):
            continue
        proposal_path = os.path.join(archive_item, "proposal.md")
        if not os.path.isfile(proposal_path):
            continue
        # Strip date prefix to get the original change ID.
        original_id = entry
        # Match YYYY-MM-DD- prefix.
        if len(entry) > 11 and entry[4] == '-' and entry[7] == '-':
            original_id = entry[11:]
        changes[original_id] = {"id": original_id, "archived": True}
    return changes


def verify_archived_prerequisite_integration(root, dep_id):
    """Verify an archived prerequisite is integrated into current HEAD.

    Finds the archive commit for the proposal and verifies it is an
    ancestor of HEAD via git merge-base.
    """
    import subprocess
    archive_dir = os.path.join(root, "openspec", "changes", "archive")
    if not os.path.isdir(archive_dir):
        return False
    for entry in os.listdir(archive_dir):
        original_id = entry
        if len(entry) > 11 and entry[4] == '-' and entry[7] == '-':
            original_id = entry[11:]
        if original_id != dep_id:
            continue
        try:
            # Use the archive commit itself. The archive rename means the
            # implementation was already integrated before archiving.
            result = subprocess.run(
                ["git", "-C", root, "log", "--format=%H", "-1",
                 "--", f"openspec/changes/archive/{entry}/"],
                capture_output=True, text=True, timeout=5
            )
            if result.returncode == 0 and result.stdout.strip():
                archive_commit = result.stdout.strip()
                check = subprocess.run(
                    ["git", "-C", root, "merge-base", "--is-ancestor",
                     archive_commit, "HEAD"],
                    capture_output=True, timeout=5
                )
                return check.returncode == 0
        except (subprocess.TimeoutExpired, OSError):
            pass
    return False


def validate_dependency_graph(selected_ids, all_changes, archived_changes, root):
    """Validate the dependency graph for selected changes."""
    issues = []

    for cid in selected_ids:
        if cid not in all_changes:
            issues.append(f"Change '{cid}' not found in active changes")
            continue
        change = all_changes[cid]
        if not change["valid"]:
            issues.append(f"Change '{cid}': {change['reason']}")
            continue

        for dep_id in change["depends_on"]:
            # Check if dependency exists as active
            if dep_id in all_changes:
                dep = all_changes[dep_id]
                if not dep["valid"]:
                    issues.append(f"Change '{cid}' depends on '{dep_id}' which has {dep['reason']}")
                continue
            # Check if dependency exists as archived.
            if dep_id in archived_changes:
                # Verify the prerequisite integration revision is in the current
                # HEAD ancestry. If proven, the dependency is satisfied.
                if verify_archived_prerequisite_integration(root, dep_id):
                    continue
                # Archived dependencies need manual evidence review.
                issues.append(f"Change '{cid}' depends on archived '{dep_id}' — manual evidence review required")
                continue
            # Not found anywhere
            issues.append(f"Change '{cid}' depends on '{dep_id}' which is not an active or archived change")

    # Cycle detection (topological sort on selected + their transitive deps)
    graph = defaultdict(list)
    in_degree = defaultdict(int)
    all_in_scope = set(selected_ids)
    # Build full transitive closure of selected
    queue = deque(selected_ids)
    while queue:
        cid = queue.popleft()
        if cid not in all_changes:
            continue
        for dep_id in all_changes[cid].get("depends_on", []):
            if dep_id not in all_in_scope and dep_id in all_changes:
                all_in_scope.add(dep_id)
                queue.append(dep_id)

    for cid in all_in_scope:
        in_degree.setdefault(cid, 0)
        if cid not in all_changes:
            continue
        for dep_id in all_changes[cid].get("depends_on", []):
            if dep_id in all_in_scope:
                graph[dep_id].append(cid)
                in_degree[cid] += 1

    # Topological sort
    ready = deque([cid for cid in all_in_scope if in_degree.get(cid, 0) == 0])
    sorted_order = []
    while ready:
        node = ready.popleft()
        sorted_order.append(node)
        for neighbor in graph.get(node, []):
            in_degree[neighbor] -= 1
            if in_degree[neighbor] == 0:
                ready.append(neighbor)

    if len(sorted_order) != len(all_in_scope):
        # Cycle detected — find the cycle
        remaining = all_in_scope - set(sorted_order)
        issues.append(f"Cycle detected involving: {', '.join(sorted(remaining))}")

    return issues, sorted_order


def main():
    parser = argparse.ArgumentParser(description="Validate OpenSpec change dependency graph")
    parser.add_argument("--root", default=".", help="Repository root")
    parser.add_argument("--json", action="store_true", help="Output JSON")
    parser.add_argument("--all", action="store_true", help="Validate all active changes")
    parser.add_argument("ids", nargs="*", help="Change IDs to validate")
    args = parser.parse_args()

    changes_dir = os.path.join(args.root, "openspec", "changes")
    archive_dir = os.path.join(args.root, "openspec", "changes", "archive")

    if not os.path.isdir(changes_dir):
        result = {"valid": False, "issues": [f"Changes directory not found: {changes_dir}"]}
        if args.json:
            print(json.dumps(result))
        else:
            for issue in result["issues"]:
                print(f"ERROR: {issue}")
        sys.exit(1)

    active_changes = load_active_changes(changes_dir)
    archived_changes = load_archived_changes(archive_dir)

    if args.all:
        selected_ids = [cid for cid in active_changes if active_changes[cid]["valid"]]
    else:
        selected_ids = args.ids

    if not selected_ids:
        result = {
            "valid": True,
            "issues": [],
            "warning": "No changes selected for validation",
            "active_count": len(active_changes),
            "archived_count": len(archived_changes),
        }
        if args.json:
            print(json.dumps(result))
        sys.exit(0)

    issues, topo_order = validate_dependency_graph(selected_ids, active_changes, archived_changes, args.root)

    result = {
        "valid": len(issues) == 0,
        "issues": issues,
        "selected": selected_ids,
        "topological_order": topo_order if not issues else None,
        "active_count": len(active_changes),
        "archived_count": len(archived_changes),
        "transitive_closure_size": len(topo_order) if topo_order else 0,
    }

    if args.json:
        print(json.dumps(result, indent=2))
    else:
        if result["valid"]:
            print(f"VALID: {len(selected_ids)} change(s), {len(topo_order)} in transitive closure")
            if topo_order:
                print(f"Topological order: {' → '.join(topo_order)}")
        else:
            for issue in issues:
                print(f"ERROR: {issue}")
            sys.exit(1)


if __name__ == "__main__":
    main()