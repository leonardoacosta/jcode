#!/usr/bin/env python3
"""Static contract validator for the Jcode command-system documentation microsite.

This script intentionally encodes the approved OpenSpec contract. It does not
create or mutate microsite HTML/CSS/source data.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from html.parser import HTMLParser
from pathlib import Path
from typing import Iterable

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SITE = ROOT / "docs/diagrams/jcode-command-system"
DEFAULT_ATLAS_SOURCE = ROOT / "docs/diagrams/agent-stack-recreation.html"

COMMAND_PAGES = [
    "command-lifecycle.html",
    "lane-protocol.html",
    "apply-orchestration.html",
    "model-routing.html",
    "evaluation-tournament.html",
    "telemetry-results.html",
]
ATLAS_PAGE = "agent-stack.html"
LAYER_PAGES = [
    "stack-surface.html",
    "stack-orchestration.html",
    "stack-context.html",
    "stack-model.html",
    "stack-tools.html",
    "stack-runtime.html",
    "stack-memory.html",
]
ECOSYSTEM_PAGE = "daily-driven-ecosystem.html"
PAGES = ["index.html", *COMMAND_PAGES, ATLAS_PAGE, *LAYER_PAGES, ECOSYSTEM_PAGE]
CHAPTERS = [*COMMAND_PAGES, ATLAS_PAGE, *LAYER_PAGES, ECOSYSTEM_PAGE]
SHARED_ASSETS = [
    "styles.css",
    "sources.json",
    "ecosystem-evidence.json",
    "atlas-history-evidence.json",
]

STABLE_IDS = {
    "DOCS-INDEX",
    "DOCS-ATLAS",
    "DOCS-LAYER",
    "DOCS-ECOSYSTEM",
    "DOCS-EVIDENCE",
    "DOCS-DIAGRAM",
    "DOCS-TELEMETRY",
    "DOCS-OFFLINE",
    "DOCS-A11Y",
    "DOCS-TRUTH",
}

REQUIRED_ARTIFACTS = {
    "DOCS-INDEX": ["index.html"],
    "DOCS-ATLAS": [ATLAS_PAGE],
    "DOCS-LAYER": LAYER_PAGES,
    "DOCS-ECOSYSTEM": [ECOSYSTEM_PAGE, "ecosystem-evidence.json"],
    "DOCS-EVIDENCE": ["sources.json"],
    "DOCS-OFFLINE": [*PAGES, "styles.css"],
    "DOCS-A11Y": PAGES,
    "DOCS-TRUTH": PAGES,
}

ATLAS_LAYERS = ["surface", "orchestration", "context", "model", "tools", "runtime", "memory"]
LAYER_TO_PAGE = dict(zip(ATLAS_LAYERS, LAYER_PAGES, strict=True))
HARNESS_CARDS = ["claude-code", "codex", "pi", "jcode", "cross-provider-agents"]
REMOTE_RE = re.compile(r"(?:src|href)=[\"'](?:https?:)?//", re.I)
CSS_REMOTE_RE = re.compile(r"(?:@import|url\()\s*[\"']?(?:https?:)?//", re.I)
ANIME_RE = re.compile(r"\banime(?:\.min)?\.js\b|\banime\.", re.I)
CONTRAST_TOKEN_RE = re.compile(r"contrast", re.I)
GIT_REV_RE = re.compile(r"^[0-9a-f]{40}$")
SHA256_RE = re.compile(r"^sha256:[0-9a-f]{64}$")
EXPECTED_ECOSYSTEM_CLASSES = {
    "claude-code": ("documented", "high"),
    "codex": ("documented", "medium"),
    "pi": ("inferred", "low"),
    "jcode": ("documented", "high"),
    "cross-provider-agents": ("documented", "medium"),
}
DIAGRAM_MANIFESTS = {
    "index.html": {"intent", "feature", "apply", "evidence"},
    "command-lifecycle.html": {"feature", "apply", "human"},
    "lane-protocol.html": {"lanes", "decision"},
    "apply-orchestration.html": {"risk", "topology", "direct", "swarm", "dag", "jcode", "orca", "verification"},
    "model-routing.html": {"deterministic", "model", "frontier", "cold", "review"},
    "evaluation-tournament.html": {"descriptor", "deterministic", "checks", "blind", "judges", "evidence", "human"},
    "telemetry-results.html": {"claude", "openai"},
    "agent-stack.html": {"surface", "orchestration", "context", "model", "tools", "runtime", "memory"},
    "stack-surface.html": {"intent", "command", "surface", "orchestration", "human"},
    "stack-orchestration.html": {"request", "plan", "slots", "merge", "evidence", "owner"},
    "stack-context.html": {"repo", "rules", "retrieved", "files", "evidence", "context"},
    "stack-model.html": {"role", "capability", "provider", "route", "fail", "closed"},
    "stack-tools.html": {"tool", "schema", "args", "effect", "observation"},
    "stack-runtime.html": {"start", "observe", "timeout", "receipt"},
    "stack-memory.html": {"fact", "receipt", "recall", "restore", "context"},
}


@dataclass
class Diagnostic:
    req: str
    page: str
    element: str
    message: str

    def render(self) -> str:
        return f"[{self.req}] {self.page}#{self.element}: {self.message}"


class Parser(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.tags: list[str] = []
        self.links: list[str] = []
        self.ids: set[str] = set()
        self.classes: set[str] = set()
        self.attrs_by_tag: list[tuple[str, dict[str, str]]] = []
        self.title = ""
        self.h1_count = 0
        self.svg_titles = 0
        self._title = False
        self._svg_depth = 0

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        attr_map = {key: value or "" for key, value in attrs}
        self.tags.append(tag)
        self.attrs_by_tag.append((tag, attr_map))
        if "id" in attr_map:
            self.ids.add(attr_map["id"])
        self.classes.update(attr_map.get("class", "").split())
        if tag == "a" and attr_map.get("href"):
            self.links.append(attr_map["href"])
        if tag == "title":
            self._title = True
            if self._svg_depth:
                self.svg_titles += 1
        if tag == "h1":
            self.h1_count += 1
        if tag == "svg":
            self._svg_depth += 1

    def handle_endtag(self, tag: str) -> None:
        if tag == "title":
            self._title = False
        if tag == "svg":
            self._svg_depth -= 1

    def handle_data(self, data: str) -> None:
        if self._title and not self._svg_depth:
            self.title += data


def diagnostic(req: str, page: str, element: str, message: str) -> Diagnostic:
    if req not in STABLE_IDS:
        raise AssertionError(f"validator emitted unsupported stable ID: {req}")
    return Diagnostic(req, page, element, message)


def local_target(site: Path, page_name: str, href: str) -> Path:
    path = href.split("#", 1)[0]
    return (site / path).resolve() if path else (site / page_name).resolve()


def load_sources(site: Path) -> tuple[dict, list[Diagnostic]]:
    try:
        return json.loads((site / "sources.json").read_text()), []
    except (json.JSONDecodeError, OSError) as exc:
        return {}, [diagnostic("DOCS-EVIDENCE", "sources.json", "file", f"unreadable source inventory: {exc}")]


def source_page_records(sources: dict) -> dict:
    return sources.get("pages", {}) if isinstance(sources.get("pages", {}), dict) else {}


def validate_inventory(site: Path) -> list[Diagnostic]:
    errors: list[Diagnostic] = []
    for req, files in REQUIRED_ARTIFACTS.items():
        for name in files:
            if not (site / name).exists():
                errors.append(diagnostic(req, name, "artifact", "missing required artifact"))
    return errors


def git_blob_digest(revision: str, path: str) -> str | None:
    if not GIT_REV_RE.fullmatch(revision):
        return None
    try:
        data = subprocess.check_output(["git", "show", f"{revision}:{path}"], cwd=ROOT, stderr=subprocess.DEVNULL)
    except (OSError, subprocess.CalledProcessError):
        return None
    return "sha256:" + hashlib.sha256(data).hexdigest()


def validate_source_pin(page_name: str, element: str, revision: str, digests: object, refs: list[str]) -> list[Diagnostic]:
    errors: list[Diagnostic] = []
    if not GIT_REV_RE.fullmatch(str(revision)):
        errors.append(diagnostic("DOCS-EVIDENCE", page_name, element, "source_revision must be a full 40-character git commit"))
        return errors
    if SHA256_RE.fullmatch(str(revision)):
        errors.append(diagnostic("DOCS-EVIDENCE", page_name, element, "source_revision must not contain a sha256 digest"))
    if not isinstance(digests, dict) or not digests:
        errors.append(diagnostic("DOCS-EVIDENCE", page_name, element, "missing source_digest map"))
        return errors
    for ref in refs:
        digest = digests.get(ref)
        if not isinstance(digest, str) or not SHA256_RE.fullmatch(digest):
            errors.append(diagnostic("DOCS-EVIDENCE", page_name, element, f"source_digest for {ref} must be sha256:<64 hex>"))
            continue
        actual = git_blob_digest(str(revision), ref)
        if actual is None:
            errors.append(diagnostic("DOCS-EVIDENCE", page_name, element, f"source_revision does not contain {ref}"))
        elif actual != digest:
            errors.append(diagnostic("DOCS-EVIDENCE", page_name, element, f"source_digest mismatch for {ref}"))
    return errors


def validate_sources(site: Path, sources: dict) -> list[Diagnostic]:
    errors: list[Diagnostic] = []
    page_records = source_page_records(sources)
    if set(page_records) != set(PAGES):
        missing = sorted(set(PAGES) - set(page_records))
        extra = sorted(set(page_records) - set(PAGES))
        errors.append(diagnostic("DOCS-EVIDENCE", "sources.json", "pages", f"page inventory mismatch; missing={missing}; extra={extra}"))

    for page_name, mapping in page_records.items():
        page_sources = mapping.get("sources", [])
        content = mapping.get("content", {})
        if not content:
            errors.append(diagnostic("DOCS-EVIDENCE", page_name, "content", "missing element-level content matrix"))
        for source in page_sources:
            if not (ROOT / source).exists():
                errors.append(diagnostic("DOCS-EVIDENCE", page_name, source, f"missing source artifact {source}"))
        for element, trace in content.items():
            if not isinstance(trace, dict):
                errors.append(diagnostic("DOCS-EVIDENCE", page_name, element, "must have structured traceability"))
                continue
            for required_key in ("claim", "evidence_class", "source_refs", "source_revision", "confidence", "implementation_status"):
                if not trace.get(required_key):
                    errors.append(diagnostic("DOCS-EVIDENCE", page_name, element, f"missing {required_key}"))
            evidence_class = trace.get("evidence_class")
            if evidence_class not in {"measured", "documented", "inferred"}:
                errors.append(diagnostic("DOCS-EVIDENCE", page_name, element, f"unsupported evidence_class {evidence_class!r}"))
            refs = trace.get("source_refs", [])
            if refs and not set(refs).issubset(set(page_sources)):
                errors.append(diagnostic("DOCS-EVIDENCE", page_name, element, "has undeclared source_refs"))
            if refs:
                errors.extend(validate_source_pin(page_name, element, str(trace.get("source_revision", "")), trace.get("source_digest"), refs))
            if evidence_class == "inferred" and trace.get("confidence") not in {"low", "medium"}:
                errors.append(diagnostic("DOCS-TRUTH", page_name, element, "inferred claim must carry low or medium confidence"))
            if page_name == ECOSYSTEM_PAGE and element in EXPECTED_ECOSYSTEM_CLASSES:
                expected_class, expected_confidence = EXPECTED_ECOSYSTEM_CLASSES[element]
                if (trace.get("evidence_class"), trace.get("confidence")) != (expected_class, expected_confidence):
                    errors.append(diagnostic("DOCS-EVIDENCE", page_name, element, f"ecosystem class/confidence must be {expected_class}/{expected_confidence}"))

    snapshot = sources.get("ecosystem_evidence") or sources.get("ecosystemEvidence")
    if not isinstance(snapshot, dict) or not (snapshot.get("snapshot_id") or snapshot.get("snapshot_digest")):
        errors.append(diagnostic("DOCS-ECOSYSTEM", "sources.json", "ecosystem_evidence", "missing frozen ecosystem snapshot id"))
    elif snapshot.get("snapshot_digest") and not SHA256_RE.fullmatch(str(snapshot.get("snapshot_digest"))):
        errors.append(diagnostic("DOCS-ECOSYSTEM", "sources.json", "ecosystem_evidence", "snapshot_digest must be sha256:<64 hex>; use snapshot_id for semantic ids"))
    return errors


def validate_page(site: Path, name: str, parser: Parser, text: str) -> list[Diagnostic]:
    errors: list[Diagnostic] = []
    if not parser.title.strip():
        errors.append(diagnostic("DOCS-A11Y", name, "title", "missing document title"))
    if parser.h1_count != 1:
        errors.append(diagnostic("DOCS-A11Y", name, "h1", f"expected one h1, found {parser.h1_count}"))
    for tag in ("header", "main", "footer", "nav"):
        if tag not in parser.tags:
            errors.append(diagnostic("DOCS-A11Y", name, tag, f"missing <{tag}>"))
    if "main" not in parser.ids:
        errors.append(diagnostic("DOCS-A11Y", name, "main", "missing #main landmark target"))
    for marker in ("breadcrumb", "chapter-menu", "diagram-panel", "diagram-fallback", "evidence-map"):
        if marker not in parser.classes:
            errors.append(diagnostic("DOCS-INDEX" if name == "index.html" else "DOCS-LAYER" if name in LAYER_PAGES else "DOCS-ATLAS" if name == ATLAS_PAGE else "DOCS-ECOSYSTEM" if name == ECOSYSTEM_PAGE else "DOCS-DIAGRAM", name, marker, f"missing .{marker}"))
    if parser.svg_titles < 2:
        errors.append(diagnostic("DOCS-A11Y", name, "svg-title", "illustration and primary diagram require SVG titles"))
    if "mermaid-source" not in parser.classes:
        errors.append(diagnostic("DOCS-DIAGRAM", name, "mermaid-source", "missing Mermaid source"))
    if name != "index.html" and "<pre" not in text:
        errors.append(diagnostic("DOCS-DIAGRAM", name, "snippet", "missing code/data snippet"))
    manifest = DIAGRAM_MANIFESTS.get(name)
    if manifest:
        svgs = re.findall(r"<svg\b[\s\S]*?</svg>", text, flags=re.I)
        mermaid = re.search(r'<pre class="mermaid-source">([\s\S]*?)</pre>', text, flags=re.I)
        fallback = re.search(r'class="diagram-fallback"[^>]*>([\s\S]*?)</(?:p|div)>', text, flags=re.I)
        representations = {
            "svg": svgs[1] if len(svgs) > 1 else "",
            "mermaid": mermaid.group(1) if mermaid else "",
            "fallback": fallback.group(1) if fallback else "",
        }
        for representation, body in representations.items():
            normalized = re.sub(r"<[^>]+>", " ", body).casefold()
            missing_terms = sorted(term for term in manifest if term not in normalized)
            if missing_terms:
                errors.append(
                    diagnostic(
                        "DOCS-DIAGRAM",
                        name,
                        representation,
                        f"semantic manifest mismatch; missing {missing_terms}",
                    )
                )
    current_links = [
        attrs
        for tag, attrs in parser.attrs_by_tag
        if tag == "a" and attrs.get("aria-current") == "page"
    ]
    if len(current_links) != 1:
        errors.append(
            diagnostic(
                "DOCS-A11Y",
                name,
                "aria-current",
                f"expected exactly one current-page link, found {len(current_links)}",
            )
        )
    if REMOTE_RE.search(text):
        errors.append(diagnostic("DOCS-OFFLINE", name, "asset", "remote asset reference"))
    if name == ATLAS_PAGE and ANIME_RE.search(text):
        errors.append(diagnostic("DOCS-ATLAS", name, "runtime", "Atlas must not import anime.js or anime runtime calls"))

    expected_links = set(CHAPTERS)
    linked_pages = {href.split("#", 1)[0] for href in parser.links if href and not href.startswith("#")}
    if name == "index.html":
        missing = sorted(expected_links - linked_pages)
        if missing:
            errors.append(diagnostic("DOCS-INDEX", name, "chapter-links", f"missing required links: {', '.join(missing)}"))
    elif name in PAGES and "index.html" not in linked_pages:
        errors.append(diagnostic("DOCS-INDEX", name, "index-link", "page cannot return to index"))

    for href in parser.links:
        if href.startswith(("#", "mailto:", "javascript:")):
            continue
        target = local_target(site, name, href)
        if site.resolve() not in target.parents and target != site.resolve():
            errors.append(diagnostic("DOCS-OFFLINE", name, href, f"link escapes microsite: {href}"))
        elif not target.exists():
            errors.append(diagnostic("DOCS-INDEX", name, href, f"broken link {href}"))
    return errors


def validate_atlas_contract(site: Path, parser: Parser, sources: dict) -> list[Diagnostic]:
    errors: list[Diagnostic] = []
    linked_pages = {href.split("#", 1)[0] for href in parser.links if href and not href.startswith("#")}
    for layer, page in LAYER_TO_PAGE.items():
        if page not in linked_pages:
            errors.append(diagnostic("DOCS-ATLAS", ATLAS_PAGE, layer, f"missing atlas card link to {page}"))
    if ECOSYSTEM_PAGE not in linked_pages:
        errors.append(diagnostic("DOCS-ATLAS", ATLAS_PAGE, "ecosystem-link", f"missing link to {ECOSYSTEM_PAGE}"))
    if "atlas-history-evidence.json" not in linked_pages:
        errors.append(diagnostic("DOCS-EVIDENCE", ATLAS_PAGE, "history-evidence", "missing persisted Atlas history evidence link"))

    atlas_record = source_page_records(sources).get(ATLAS_PAGE, {})
    content_keys = set((atlas_record.get("content") or {}).keys())
    for layer in ATLAS_LAYERS:
        if layer not in content_keys and f"layer-{layer}" not in content_keys:
            errors.append(diagnostic("DOCS-ATLAS", ATLAS_PAGE, layer, "missing source record for Atlas layer card"))
    return errors


def validate_layer_contract(name: str, parser: Parser, text: str) -> list[Diagnostic]:
    errors: list[Diagnostic] = []
    linked_pages = {href.split("#", 1)[0] for href in parser.links if href and not href.startswith("#")}
    for target in (ATLAS_PAGE, "index.html"):
        if target not in linked_pages:
            errors.append(diagnostic("DOCS-LAYER", name, target, f"missing navigation link to {target}"))
    for phrase in ("evolution", "daily", "interfaces", "ownership", "failure", "related"):
        if phrase not in text.casefold():
            errors.append(diagnostic("DOCS-LAYER", name, phrase, f"missing layer contract section: {phrase}"))
    return errors


def validate_ecosystem(site: Path, parser: Parser) -> list[Diagnostic]:
    errors: list[Diagnostic] = []
    linked_targets = "\n".join(parser.links).casefold()
    try:
        evidence = json.loads((site / "ecosystem-evidence.json").read_text())
    except (json.JSONDecodeError, OSError) as exc:
        return [diagnostic("DOCS-ECOSYSTEM", "ecosystem-evidence.json", "file", f"unreadable frozen ecosystem evidence: {exc}")]
    claims = evidence.get("claims", []) if isinstance(evidence, dict) else []
    claim_by_id = {claim.get("id"): claim for claim in claims if isinstance(claim, dict)}
    for card in HARNESS_CARDS:
        if card not in linked_targets and not any(card in str(claim).casefold() for claim in claims):
            errors.append(diagnostic("DOCS-ECOSYSTEM", ECOSYSTEM_PAGE, card, "missing linked or frozen evidence-backed harness card"))
        if card in EXPECTED_ECOSYSTEM_CLASSES:
            claim = claim_by_id.get(card, {})
            expected_label, expected_confidence = EXPECTED_ECOSYSTEM_CLASSES[card]
            if (claim.get("label"), claim.get("confidence")) != (expected_label, expected_confidence):
                errors.append(diagnostic("DOCS-EVIDENCE", "ecosystem-evidence.json", card, f"claim label/confidence must be {expected_label}/{expected_confidence}"))
    for ref in evidence.get("references", []):
        if not isinstance(ref, dict):
            continue
        revision = ref.get("source_revision")
        digest = ref.get("digest")
        path = ref.get("path")
        if not GIT_REV_RE.fullmatch(str(revision)):
            errors.append(diagnostic("DOCS-EVIDENCE", "ecosystem-evidence.json", str(ref.get("ref_id", "reference")), "reference source_revision must be a full git commit"))
        elif isinstance(path, str) and isinstance(digest, str):
            actual = git_blob_digest(str(revision), path)
            if not SHA256_RE.fullmatch(digest):
                errors.append(diagnostic("DOCS-EVIDENCE", "ecosystem-evidence.json", path, "reference digest must be sha256:<64 hex>"))
            elif actual and actual != digest:
                errors.append(diagnostic("DOCS-EVIDENCE", "ecosystem-evidence.json", path, "reference digest mismatch"))
    return errors


def validate_css(site: Path) -> list[Diagnostic]:
    errors: list[Diagnostic] = []
    css_path = site / "styles.css"
    try:
        css = css_path.read_text()
    except OSError as exc:
        return [diagnostic("DOCS-OFFLINE", "styles.css", "file", f"unreadable CSS: {exc}")]
    for token in ("--parchment", "--walnut", "--umber", "--espresso", "--copper", ":focus-visible", "prefers-reduced-motion"):
        if token not in css:
            errors.append(diagnostic("DOCS-A11Y" if token in {":focus-visible", "prefers-reduced-motion"} else "DOCS-TRUTH", "styles.css", token, f"missing {token}"))
    if CSS_REMOTE_RE.search(css):
        errors.append(diagnostic("DOCS-OFFLINE", "styles.css", "asset", "remote CSS asset reference"))
    mobile = css.split("@media(max-width:900px)", 1)[-1].split("@media(max-width:560px)", 1)[0]
    if ".chapter-menu{display:none}" in mobile or "overflow-x:auto" not in mobile:
        errors.append(diagnostic("DOCS-A11Y", "styles.css", "mobile-nav", "mobile chapter navigation must remain visible and scrollable"))
    if not CONTRAST_TOKEN_RE.search(css):
        errors.append(diagnostic("DOCS-A11Y", "styles.css", "contrast", "contrast pair computation metadata is missing"))
    if "#f3bd52" in css:
        errors.append(diagnostic("DOCS-A11Y", "styles.css", "focus-visible", "focus color fails on light surfaces"))
    if ".chapter-menu a[aria-current=page]{background:var(--copper);color:var(--espresso)}" in css:
        errors.append(diagnostic("DOCS-A11Y", "styles.css", "active-tab", "active chapter tab contrast is below 4.5:1"))
    if "pre.mermaid-source" not in css:
        errors.append(diagnostic("DOCS-A11Y", "styles.css", "mermaid-source", "pre.mermaid-source needs an explicit readable foreground"))
    if ".diagram-panel,.snippet-panel,.layer article,.layer>section" in css:
        errors.append(diagnostic("DOCS-TRUTH", "styles.css", "atlas-scope", "Atlas additive selectors must be scoped under .atlas-shell"))
    if ".card .num{font:700 54px" in css:
        errors.append(diagnostic("DOCS-A11Y", "styles.css", "evidence-label", "evidence labels must not use the low-opacity watermark component"))
    return errors


def validate_telemetry(site: Path, sources: dict) -> list[Diagnostic]:
    errors: list[Diagnostic] = []
    telemetry_source = sources.get("telemetry_source") or "evals/model-routing/runs/oauth-smoke-2026-08-12.json"
    telemetry_path = ROOT / telemetry_source
    try:
        telemetry_data = json.loads(telemetry_path.read_text())
    except (json.JSONDecodeError, OSError) as exc:
        return [diagnostic("DOCS-TELEMETRY", "telemetry-results.html", telemetry_source, f"unreadable telemetry evidence: {exc}")]
    try:
        page_text = (site / "telemetry-results.html").read_text().casefold()
    except OSError as exc:
        return [diagnostic("DOCS-TELEMETRY", "telemetry-results.html", "file", f"unreadable telemetry page: {exc}")]
    evidence_text = json.dumps(telemetry_data, sort_keys=True)
    for token_value in re.findall(r"\b\d{1,3}(?:,\d{3})+\b|\b\d+\.\d+s\b", evidence_text):
        if token_value.casefold() not in page_text:
            errors.append(diagnostic("DOCS-TELEMETRY", "telemetry-results.html", token_value, "telemetry value from committed JSON is absent from page"))
    for phrase in ("separate human approval", "production routing was not mutated", "one frozen fixture", "split judge"):
        if phrase not in page_text:
            errors.append(diagnostic("DOCS-TRUTH", "telemetry-results.html", phrase, f"missing decision boundary: {phrase}"))
    return errors


def validate_site(site: Path = DEFAULT_SITE, stop_after_inventory: bool = False) -> list[Diagnostic]:
    errors = validate_inventory(site)
    if errors and stop_after_inventory:
        return errors

    sources, source_errors = load_sources(site)
    errors.extend(source_errors)
    if not source_errors:
        errors.extend(validate_sources(site, sources))

    page_parsers: dict[str, Parser] = {}
    for name in PAGES:
        try:
            text = (site / name).read_text()
        except OSError:
            continue
        parser = Parser()
        parser.feed(text)
        page_parsers[name] = parser
        errors.extend(validate_page(site, name, parser, text))

    if ATLAS_PAGE in page_parsers and not source_errors:
        errors.extend(validate_atlas_contract(site, page_parsers[ATLAS_PAGE], sources))
    for name in LAYER_PAGES:
        if name in page_parsers:
            errors.extend(validate_layer_contract(name, page_parsers[name], (site / name).read_text()))
    if ECOSYSTEM_PAGE in page_parsers:
        errors.extend(validate_ecosystem(site, page_parsers[ECOSYSTEM_PAGE]))

    if (site / "styles.css").exists():
        errors.extend(validate_css(site))
    if not source_errors and (site / "telemetry-results.html").exists():
        errors.extend(validate_telemetry(site, sources))
    return errors


class LayerHeadingParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.headings: list[str] = []
        self._in_layer_h2 = False
        self._current = ""
        self._article_depth = 0
        self._in_layer_article = False

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        attr_map = {key: value or "" for key, value in attrs}
        if tag == "article" and "layer" in attr_map.get("class", "").split():
            self._in_layer_article = True
            self._article_depth = 1
        elif self._in_layer_article:
            self._article_depth += 1
        if self._in_layer_article and tag == "h2":
            self._in_layer_h2 = True
            self._current = ""

    def handle_endtag(self, tag: str) -> None:
        if self._in_layer_h2 and tag == "h2":
            value = re.sub(r"^\s*\d+\s+", "", self._current.strip().casefold())
            self.headings.append(value)
            self._in_layer_h2 = False
        if self._in_layer_article:
            self._article_depth -= 1
            if self._article_depth <= 0:
                self._in_layer_article = False

    def handle_data(self, data: str) -> None:
        if self._in_layer_h2:
            self._current += data


def extract_atlas_source_layers(source: Path) -> list[str]:
    parser = LayerHeadingParser()
    parser.feed(source.read_text())
    return parser.headings


def validate_atlas_source(source: Path, site: Path = DEFAULT_SITE) -> list[Diagnostic]:
    errors: list[Diagnostic] = []
    try:
        source_text = source.read_text()
        extracted = extract_atlas_source_layers(source)
    except OSError as exc:
        return [diagnostic("DOCS-ATLAS", str(source), "source", f"unreadable Atlas source: {exc}")]
    if extracted != ATLAS_LAYERS:
        errors.append(diagnostic("DOCS-ATLAS", str(source), "layer-order", f"source layers {extracted} do not match approved order {ATLAS_LAYERS}"))
    if not (site / ATLAS_PAGE).exists():
        errors.append(diagnostic("DOCS-ATLAS", ATLAS_PAGE, "artifact", "missing Atlas page for source comparison"))
        return errors
    atlas_text = (site / ATLAS_PAGE).read_text().casefold()
    if ANIME_RE.search(atlas_text) or REMOTE_RE.search(atlas_text):
        errors.append(diagnostic("DOCS-ATLAS", ATLAS_PAGE, "runtime", "new Atlas imports anime.js or remote runtime dependency"))
    for layer, page in LAYER_TO_PAGE.items():
        if layer not in atlas_text:
            errors.append(diagnostic("DOCS-ATLAS", ATLAS_PAGE, layer, "Atlas card is missing authoritative layer name"))
        if page not in atlas_text:
            errors.append(diagnostic("DOCS-ATLAS", ATLAS_PAGE, layer, f"Atlas card is missing dedicated page link {page}"))
    if ANIME_RE.search(source_text) and not source.exists():
        errors.append(diagnostic("DOCS-ATLAS", str(source), "source", "unreachable source artifact"))
    return errors


def run_self_test() -> int:
    """Exercise representative negative fixtures for the stable diagnostic API."""
    expected_ids = {
        "DOCS-EVIDENCE",
        "DOCS-TELEMETRY",
        "DOCS-ATLAS",
        "DOCS-DIAGRAM",
        "DOCS-OFFLINE",
        "DOCS-TRUTH",
        "DOCS-A11Y",
    }
    with tempfile.TemporaryDirectory() as tmp:
        site = Path(tmp)
        (site / "index.html").write_text('<!doctype html><title>x</title><main id="main"><a href="agent-stack.html">atlas</a><img src="https://example.invalid/x.png"></main>')
        (site / "telemetry-results.html").write_text('<!doctype html><title>telemetry</title><main id="main"><h1>Telemetry</h1><p>drifted values only</p></main>')
        (site / "styles.css").write_text(":root{--parchment:#fff;--walnut:#000;--umber:#321;--espresso:#111;--copper:#b76}:focus-visible{outline:1px solid} @media(max-width:900px){.chapter-menu{display:none}}")
        (site / "site.js").write_text("")
        (site / "sources.json").write_text(json.dumps({"telemetry_source": "missing-telemetry-fixture.json", "pages": {"index.html": {"sources": ["missing-source.md"], "content": {"claim": {"claim": "unsupported", "evidence_class": "unsupported", "source_refs": ["missing-source.md"], "source_revision": "stale", "source_digest": {"missing-source.md": "not-a-digest"}, "confidence": "low", "implementation_status": "draft"}}}}}))
        errors = validate_site(site)
    observed_ids = {error.req for error in errors}
    missing = sorted(expected_ids - observed_ids)
    if missing:
        for item in missing:
            print(f"[DOCS-TRUTH] self-test#{item}: negative fixture did not emit expected diagnostic family", file=sys.stderr)
        return 1
    rendered = "\n".join(error.render() for error in errors).casefold()
    promised_defects = {
        "stale revision": "source_revision",
        "semantic diagram disagreement": "semantic manifest mismatch",
        "telemetry drift": "telemetry evidence",
        "remote asset": "remote asset",
        "inaccessible contrast": "contrast",
        "current-page ambiguity": "aria-current",
    }
    absent = [label for label, marker in promised_defects.items() if marker not in rendered]
    if absent:
        print(
            f"[DOCS-TRUTH] self-test#defect-classes: missing focused diagnostics for {absent}",
            file=sys.stderr,
        )
        return 1
    print(f"command-system-docs self-test: PASS ({len(errors)} expected negative diagnostics observed)")
    return 0


def emit(errors: Iterable[Diagnostic]) -> int:
    errors = list(errors)
    for error in errors:
        print(error.render(), file=sys.stderr)
    print(f"command-system-docs: {'PASS' if not errors else 'FAIL'} ({len(PAGES)} pages)")
    return len(errors)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--site", type=Path, default=DEFAULT_SITE, help="Microsite directory to validate")
    parser.add_argument("--self-test", action="store_true", help="Run negative validator fixtures and require stable diagnostics")
    parser.add_argument("--check-atlas-source", type=Path, metavar="HTML", help="Compare System Atlas with the authoritative source diagram")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv or sys.argv[1:])
    if args.self_test:
        return run_self_test()
    if args.check_atlas_source:
        return emit(validate_atlas_source(args.check_atlas_source, args.site))
    return emit(validate_site(args.site, stop_after_inventory=True))


if __name__ == "__main__":
    raise SystemExit(main())
