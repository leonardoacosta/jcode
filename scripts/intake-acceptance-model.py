#!/usr/bin/env python3
"""Executable acceptance model for the factory-intake spec.

Every test maps to a named scenario in
openspec/changes/add-factory-intake-capability/specs/factory-intake/spec.md.

This is a reference model, not the implementation. Its purpose is to find
scenarios that are contradictory, unimplementable, or silently under-specified
before anyone writes production code. A failure here is a spec defect.

Usage: intake-acceptance-model.py    (exit 0 all pass, 1 any fail)
"""
import hashlib
import re
import sys

CREDENTIAL_PATTERNS = [
    re.compile(r"\b[0-9]{8,10}:[A-Za-z0-9_-]{35}\b"),        # telegram bot token
    re.compile(r"\bxox[baprs]-[A-Za-z0-9-]{10,}\b"),          # slack token
    re.compile(r"\b(?:sk|pk)-[A-Za-z0-9]{20,}\b"),            # api key
    re.compile(r"\bghp_[A-Za-z0-9]{36}\b"),                   # github pat
]
REDACTION_MARKER = "[REDACTED]"


def scrub(text):
    """Return (scrubbed_text, redaction_count)."""
    if text is None:
        return None, 0
    count = 0
    for pat in CREDENTIAL_PATTERNS:
        text, n = pat.subn(REDACTION_MARKER, text)
        count += n
    return text, count


def scrub_deep(obj):
    """Scrub every string in a nested structure. Returns (obj, count)."""
    total = 0
    if isinstance(obj, str):
        return scrub(obj)
    if isinstance(obj, dict):
        out = {}
        for k, v in obj.items():
            out[k], n = scrub_deep(v)
            total += n
        return out, total
    if isinstance(obj, list):
        out = []
        for v in obj:
            sv, n = scrub_deep(v)
            out.append(sv)
            total += n
        return out, total
    return obj, 0


class Intake:
    """Reference model of the factory-intake capability."""

    def __init__(self, execution_budget=None):
        self.records = []        # every inbound message, permanently
        self.proposals = []
        self.tracked_work = []
        self.events = []         # redaction, deferral, failure events
        self._seen = {}          # dedupe key -> record id
        self._next_id = 1
        # Budget is per message class, so read-only traffic cannot starve
        # work proposals. An int applies the same budget to every class.
        self.execution_budget = execution_budget
        self._executed = 0
        self._executed_by_class = {}

    # -- envelope ---------------------------------------------------------
    def dedupe_key(self, sender_identity, conversation, content):
        """Derived from content and identity only. No transport sequence."""
        h = hashlib.sha256()
        for part in (sender_identity, conversation, content or ""):
            h.update(part.encode())
            h.update(b"\x1f")
        return h.hexdigest()

    def receive(self, adapter, sender_identity, conversation, content,
                raw_payload, operator=None, classify=None):
        """Record first, then interpret. Returns the record."""
        # 1. scrub at ingress, before ANY write
        clean_content, n1 = scrub(content)
        clean_raw, n2 = scrub_deep(raw_payload)
        redactions = n1 + n2

        # Dedupe on PRE-scrub content: two different credentials both redact to
        # the same marker, and keying on the scrubbed text would collapse them.
        key = self.dedupe_key(sender_identity, conversation, content)
        rid = self._next_id
        self._next_id += 1

        record = {
            "id": rid,
            "adapter": adapter,
            "sender_identity": sender_identity,
            "conversation": conversation,
            "content": clean_content,
            "raw_payload": clean_raw,
            "operator": operator,
            "dedupe_key": key,
            "duplicate_of": self._seen.get(key),
            "retry_of": None,
            "classification": None,
            "classification_error": None,
            "executed": False,
            "deferred": False,
        }
        # durable record happens here, before interpretation
        self.records.append(record)
        if record["duplicate_of"] is None:
            self._seen[key] = rid

        if redactions:
            self.events.append({"type": "redaction", "record": rid,
                                "count": redactions})

        prior = None
        if record["duplicate_of"] is not None:
            prior = self.records[record["duplicate_of"] - 1]
            # A resend of a message that never executed is a retry, not a
            # duplicate to swallow. Otherwise throttling is permanent.
            if prior["executed"] or prior["classification_error"] is not None:
                return record
            record["retry_of"] = record["duplicate_of"]
            record["duplicate_of"] = None

        # 2. interpret (classification is needed to apply a per-class budget)
        if classify is not None:
            try:
                record["classification"] = classify(clean_content)
            except Exception as exc:
                record["classification_error"] = str(exc)
                self.events.append({"type": "classification_failure",
                                    "record": rid, "error": str(exc)})
                return record

        # 3. admission control bounds execution, never recording
        cls = record["classification"] or "unclassified"
        if self.execution_budget is not None:
            used = self._executed_by_class.get(cls, 0)
            if used >= self.execution_budget:
                record["deferred"] = True
                self.events.append({"type": "deferral", "record": rid,
                                    "class": cls})
                return record
            self._executed_by_class[cls] = used + 1

        self._executed += 1
        record["executed"] = True

        if record["classification"] == "work_request":
            self.proposals.append({
                "id": len(self.proposals) + 1,
                "record": rid,
                "state": "awaiting_approval",
                "approved_by": None,
                "approved_at": None,
                "approved_channel": None,
            })
        return record

    def approve(self, proposal_id, approver_identity, at, channel):
        p = self.proposals[proposal_id - 1]
        p["state"] = "approved"
        p["approved_by"] = approver_identity
        p["approved_at"] = at
        p["approved_channel"] = channel
        self.tracked_work.append({
            "id": len(self.tracked_work) + 1,
            "from_record": p["record"],
            "from_proposal": proposal_id,
        })
        return self.tracked_work[-1]


# -- test harness ---------------------------------------------------------
RESULTS = []


def check(requirement, scenario, condition, detail=""):
    RESULTS.append((requirement, scenario, bool(condition), detail))


def telegram_raw(update_id, chat_id, text):
    return {"update_id": update_id, "message": {"chat": {"id": chat_id},
            "from": {"id": 42}, "text": text}}


def run():
    # Requirement: Provider-neutral intake envelope
    ix = Intake()
    r = ix.receive("telegram", "op:leo", "conv:1", "hello",
                   telegram_raw(1000, 555, "hello"), operator="leo")
    envelope_fields = {"id", "adapter", "sender_identity", "conversation",
                       "content", "operator", "dedupe_key"}
    leaked = [f for f in r if f in ("update_id", "chat_id", "thread_ts")]
    check("Provider-neutral intake envelope", "A message arrives from any transport",
          envelope_fields <= set(r) and not leaked,
          f"leaked={leaked}")
    check("Provider-neutral intake envelope", "A message arrives from any transport",
          r["raw_payload"] is not None and "update_id" in r["raw_payload"],
          "raw payload retained separately")

    # second transport, identical core path
    r2 = ix.receive("slack", "op:leo", "conv:2", "hello from slack",
                    {"event": {"user": "U1", "channel": "C1",
                               "thread_ts": "1.2", "text": "hello from slack"}},
                    operator="leo")
    check("Provider-neutral intake envelope", "A second transport is added",
          set(r2) == set(r), "same envelope shape, no schema change")

    # Requirement: Content-derived deduplication
    ix = Intake()
    a = ix.receive("telegram", "op:leo", "conv:1", "deploy the thing",
                   telegram_raw(1, 555, "deploy the thing"))
    b = ix.receive("telegram", "op:leo", "conv:1", "deploy the thing",
                   telegram_raw(2, 555, "deploy the thing"))
    check("Content-derived deduplication", "A transport replays a delivery",
          b["duplicate_of"] == a["id"] and len(ix.records) == 2,
          "duplicate marked, both recorded")

    # sequence randomized after inactivity (the Telegram behavior)
    c = ix.receive("telegram", "op:leo", "conv:1", "deploy the thing",
                   telegram_raw(987654321, 555, "deploy the thing"))
    check("Content-derived deduplication", "A transport resets its sequence numbering",
          c["duplicate_of"] == a["id"],
          "dedupe survives randomized update_id")

    # distinct content is not a duplicate
    d = ix.receive("telegram", "op:leo", "conv:1", "deploy something else",
                   telegram_raw(3, 555, "deploy something else"))
    check("Content-derived deduplication", "A transport resets its sequence numbering",
          d["duplicate_of"] is None, "distinct content not collapsed")

    # identity must participate in the key, or distinct senders collide
    e = ix.receive("telegram", "op:sam", "conv:1", "deploy the thing",
                   telegram_raw(4, 555, "deploy the thing"))
    check("Content-derived deduplication", "Distinct senders send identical content",
          e["duplicate_of"] is None,
          "identity participates in the key; no cross-sender collision")

    f = ix.receive("telegram", "op:leo", "conv:99", "deploy the thing",
                   telegram_raw(5, 999, "deploy the thing"))
    check("Content-derived deduplication", "Distinct senders send identical content",
          f["duplicate_of"] is None,
          "conversation participates in the key")

    # Interaction: redaction must not collapse distinct messages
    ix = Intake()
    t1 = "123456789:AAHfSHFyTvJmL5RkQxWnPzZbCdEfGhIjKlM"
    t2 = "987654321:BBHfSHFyTvJmL5RkQxWnPzZbCdEfGhIjKlM"
    p1 = ix.receive("telegram", "op:leo", "c", f"token {t1}", telegram_raw(1, 5, "x"))
    p2 = ix.receive("telegram", "op:leo", "c", f"token {t2}", telegram_raw(2, 5, "x"))
    check("Content-derived deduplication", "Redacted messages remain distinct",
          p2["duplicate_of"] is None,
          "dedupe uses pre-scrub content; distinct credentials do not collapse")

    # Interaction: a resend of a never-executed message is a retry
    ix = Intake(execution_budget=0)
    q1 = ix.receive("telegram", "op:leo", "c", "do it", telegram_raw(1, 5, "do it"),
                    classify=lambda t: "work_request")
    q2 = ix.receive("telegram", "op:leo", "c", "do it", telegram_raw(2, 5, "do it"),
                    classify=lambda t: "work_request")
    check("Execution admission control", "A throttled message is resent",
          q1["deferred"] and q2["retry_of"] == q1["id"] and q2["duplicate_of"] is None,
          "retry is not swallowed as a duplicate")

    ix = Intake()
    s1 = ix.receive("telegram", "op:leo", "c", "hi", telegram_raw(1, 5, "hi"),
                    classify=lambda t: "status_request")
    s2 = ix.receive("telegram", "op:leo", "c", "hi", telegram_raw(2, 5, "hi"),
                    classify=lambda t: "status_request")
    check("Content-derived deduplication", "A transport replays a delivery",
          s2["duplicate_of"] == s1["id"],
          "genuine replay of an EXECUTED message is still deduped")

    # Requirement: Durable record before interpretation
    ix = Intake()

    def exploding(_):
        raise ValueError("classifier blew up")

    r = ix.receive("telegram", "op:leo", "conv:1", "ambiguous",
                   telegram_raw(1, 555, "ambiguous"), classify=exploding)
    check("Durable record before interpretation", "Interpretation fails",
          len(ix.records) == 1 and r["classification_error"] is not None,
          "record survives classifier failure")

    ix = Intake(execution_budget=1)
    ix.receive("telegram", "op:leo", "c", "one", telegram_raw(1, 5, "one"),
               classify=lambda t: "status_request")
    ix.receive("telegram", "op:leo", "c", "two", telegram_raw(2, 5, "two"),
               classify=lambda t: "status_request")
    ix.receive("telegram", "op:unknown", "c", "three", telegram_raw(3, 5, "three"),
               classify=lambda t: "unrecognized")
    ix.receive("telegram", "op:leo", "c", "one", telegram_raw(4, 5, "one"))
    check("Durable record before interpretation", "An operator inspects history",
          len(ix.records) == 4,
          "duplicates, throttled, unrecognized all retained")

    # Requirement: Credential scrubbing at ingress
    ix = Intake()
    token = "123456789:AAHfSHFyTvJmL5RkQxWnPzZbCdEfGhIjKlM"
    r = ix.receive("telegram", "op:leo", "conv:1",
                   f"use this token {token} please",
                   telegram_raw(1, 555, f"use this token {token} please"))
    stored_blob = repr(ix.records) + repr(ix.events)
    check("Credential scrubbing at ingress", "A credential is pasted into a message",
          REDACTION_MARKER in r["content"], "content redacted")
    check("Credential scrubbing at ingress", "A credential is pasted into a message",
          any(e["type"] == "redaction" for e in ix.events), "redaction event recorded")
    check("Credential scrubbing at ingress", "A credential is pasted into a message",
          token not in stored_blob,
          "NO unredacted copy in any store, including raw payload")

    ix = Intake()
    r = ix.receive("telegram", "op:leo", "conv:1", "just a normal message",
                   telegram_raw(1, 555, "just a normal message"))
    check("Credential scrubbing at ingress", "Ordinary content arrives",
          r["content"] == "just a normal message" and not ix.events,
          "verbatim, no spurious redaction")

    # Requirement: Promotion is explicit and audited
    ix = Intake()
    r = ix.receive("telegram", "op:leo", "conv:1", "build me a dashboard",
                   telegram_raw(1, 555, "build me a dashboard"),
                   classify=lambda t: "work_request")
    check("Promotion is explicit and audited", "A message requests new work",
          len(ix.proposals) == 1
          and ix.proposals[0]["state"] == "awaiting_approval"
          and len(ix.tracked_work) == 0,
          "proposal only, no tracked work before approval")

    w = ix.approve(1, "op:leo", "2026-08-15T17:00:00Z", "telegram")
    p = ix.proposals[0]
    check("Promotion is explicit and audited", "A proposal is approved",
          len(ix.tracked_work) == 1 and w["from_record"] == r["id"]
          and all([p["approved_by"], p["approved_at"], p["approved_channel"]]),
          "work linked to record; approver, time, channel recorded")

    ix = Intake()
    before = (len(ix.proposals), len(ix.tracked_work))
    ix.receive("telegram", "op:leo", "conv:1", "what is the status",
               telegram_raw(1, 555, "what is the status"),
               classify=lambda t: "status_request")
    check("Promotion is explicit and audited", "A message is read-only",
          (len(ix.proposals), len(ix.tracked_work)) == before,
          "read-only path mutates nothing")

    # Requirement: Intake authority is bounded
    ix = Intake()
    r = ix.receive("telegram", "op:leo", "conv:1", "build me a dashboard",
                   telegram_raw(1, 555, "build me a dashboard"),
                   classify=lambda t: "work_request")
    ix.approve(1, "op:leo", "2026-08-15T17:00:00Z", "telegram")
    # conversation is deleted upstream: model as losing all conversation refs
    for rec in ix.records:
        rec["conversation"] = None
    intact = (len(ix.records) == 1 and len(ix.proposals) == 1
              and len(ix.tracked_work) == 1
              and ix.tracked_work[0]["from_record"] == r["id"])
    check("Intake authority is bounded", "Conversation history is unavailable",
          intact, "records, approvals, work survive conversation loss")

    p = ix.proposals[0]
    check("Intake authority is bounded", "An approval is issued",
          p["approved_by"] == "op:leo",
          "authority carried by approver identity, not location")

    # Requirement: Execution admission control
    ix = Intake(execution_budget=2)
    for i in range(5):
        ix.receive("telegram", "op:leo", "conv:1", f"request {i}",
                   telegram_raw(i, 555, f"request {i}"),
                   classify=lambda t: "status_request")
    deferred = [r for r in ix.records if r["deferred"]]
    executed = [r for r in ix.records if r["executed"]]
    check("Execution admission control",
          "Messages arrive faster than the configured execution budget",
          len(ix.records) == 5, "all recorded in full")
    check("Execution admission control",
          "Messages arrive faster than the configured execution budget",
          len(executed) == 2 and len(deferred) == 3, "execution bounded")
    check("Execution admission control",
          "Messages arrive faster than the configured execution budget",
          sum(1 for e in ix.events if e["type"] == "deferral") == 3,
          "deferral recorded per affected message")

    # per-class budget: read-only traffic must not starve work proposals
    ix = Intake(execution_budget=1)
    ix.receive("telegram", "op:leo", "c", "status?", telegram_raw(1, 5, "status?"),
               classify=lambda t: "status_request")
    w = ix.receive("telegram", "op:leo", "c", "build a thing",
                   telegram_raw(2, 5, "build a thing"),
                   classify=lambda t: "work_request")
    check("Execution admission control",
          "Admission control is applied across message classes",
          not w["deferred"] and len(ix.proposals) == 1,
          "read-only traffic does not starve work proposals")

    ix = Intake(execution_budget=1)
    ix.receive("telegram", "op:leo", "c", "s1", telegram_raw(1, 5, "s1"),
               classify=lambda t: "status_request")
    s = ix.receive("telegram", "op:leo", "c", "s2", telegram_raw(2, 5, "s2"),
                   classify=lambda t: "status_request")
    check("Execution admission control",
          "Admission control is applied across message classes",
          s["deferred"] and any(e.get("class") == "status_request"
                                for e in ix.events if e["type"] == "deferral"),
          "same-class budget still enforced; class recorded on deferral")


def main():
    run()
    width = max(len(r) for r, _, _, _ in RESULTS)
    failed = 0
    last = None
    for req, scen, ok, detail in RESULTS:
        if req != last:
            print(f"\n{req}")
            last = req
        if not ok:
            failed += 1
        print(f"  [{'ok' if ok else 'FAIL'}] {scen}"
              f"{('  -- ' + detail) if detail else ''}")
    total = len(RESULTS)
    print(f"\n{total - failed}/{total} scenario assertions passed")
    if failed:
        print(f"{failed} SPEC DEFECT(S): a scenario is contradictory or unimplementable")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
