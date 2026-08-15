#!/usr/bin/env python3
"""Executable acceptance model for transport adapters.

Every assertion maps to a named scenario in the channel-adapter-telegram
spec. A second, deliberately different adapter (Slack-shaped) is driven
through the SAME intake core to test the neutrality claim by execution
rather than by comparing specification text.

Usage: adapter-acceptance-model.py    (exit 0 all pass, 1 any fail)
"""
import importlib.util
import pathlib
import sys

_spec = importlib.util.spec_from_file_location(
    "intake_model", pathlib.Path(__file__).parent / "intake-acceptance-model.py")
_m = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_m)
Intake = _m.Intake

RESULTS = []


def check(requirement, scenario, condition, detail=""):
    RESULTS.append((requirement, scenario, bool(condition), detail))


class AdapterBase:
    """What every adapter must do, with no transport knowledge."""

    name = None

    def __init__(self, intake, identity_map, bot_handle="@jcode"):
        self.intake = intake
        self.identity_map = identity_map
        self.bot_handle = bot_handle
        self.outbound = []
        self.unhandled = []
        self.last_handoff = None

    # -- transport-specific hooks, implemented per adapter ----------------
    def parse(self, raw):
        raise NotImplementedError

    def deliver(self, conversation, text):
        self.outbound.append({"conversation": conversation, "text": text})

    # -- shared gating, identical for every adapter -----------------------
    def handle(self, raw, classify=None):
        parsed = self.parse(raw)
        if parsed is not None:
            # Record what crosses the boundary, so neutrality can be asserted
            # on the handoff itself rather than only on the resulting record.
            self.last_handoff = dict(parsed)
        if parsed is None:
            self.unhandled.append(self.variant_of(raw))
            # unhandled payloads are still recorded, never dropped
            rec = self.intake.receive(
                self.name, "unknown", "unknown", None, raw)
            rec["unhandled_variant"] = self.variant_of(raw)
            return rec

        if parsed["is_group"] and not parsed["addresses_bot"]:
            return None  # not forwarded, no record

        operator = self.identity_map.get(parsed["sender"])
        rec = self.intake.receive(
            self.name,
            operator or f"unmapped:{parsed['sender']}",
            parsed["conversation"],
            parsed["text"],
            raw,
            operator=operator,
            classify=classify if operator else None,
        )
        rec["authorized"] = operator is not None
        if operator is None:
            rec["classification"] = "unauthorized"

        if any(e["type"] == "redaction" and e["record"] == rec["id"]
               for e in self.intake.events):
            self.deliver(parsed["conversation"],
                         "Note: credential-shaped content was redacted before storage.")
        return rec

    def variant_of(self, raw):
        raise NotImplementedError


class TelegramAdapter(AdapterBase):
    name = "telegram"

    def parse(self, raw):
        msg = raw.get("message") or raw.get("channel_post")
        if msg is None or "text" not in msg:
            return None
        chat = msg["chat"]
        is_group = chat.get("type") in ("group", "supergroup")
        text = msg["text"]
        addresses = (self.bot_handle in text
                     or (msg.get("reply_to_message", {})
                         .get("from", {}).get("is_bot") is True))
        return {
            "sender": f"tg:{msg['from']['id']}",
            "conversation": f"tg:{chat['id']}",
            "text": text,
            "is_group": is_group,
            "addresses_bot": addresses,
        }

    def variant_of(self, raw):
        for k in raw:
            if k != "update_id":
                return k
        return "unknown"


class SlackAdapter(AdapterBase):
    """Deliberately different shape: different nesting, different id fields,
    different group semantics, different unhandled-variant vocabulary."""

    name = "slack"

    def parse(self, raw):
        ev = raw.get("event", {})
        if ev.get("type") != "message" or "text" not in ev:
            return None
        channel = ev["channel"]
        is_group = not channel.startswith("D")  # D = DM in Slack
        text = ev["text"]
        addresses = (self.bot_handle in text or "thread_ts" in ev)
        return {
            "sender": f"sl:{ev['user']}",
            "conversation": f"sl:{channel}",
            "text": text,
            "is_group": is_group,
            "addresses_bot": addresses,
        }

    def variant_of(self, raw):
        return raw.get("event", {}).get("type", "unknown")


def tg(update_id, chat_id, text, chat_type="private", user=7, reply_bot=False):
    msg = {"chat": {"id": chat_id, "type": chat_type},
           "from": {"id": user}, "text": text}
    if reply_bot:
        msg["reply_to_message"] = {"from": {"is_bot": True}}
    return {"update_id": update_id, "message": msg}


def sl(channel, text, user="U7", thread=False):
    ev = {"type": "message", "channel": channel, "user": user, "text": text}
    if thread:
        ev["thread_ts"] = "1.2"
    return {"event": ev}


def run():
    IDS = {"tg:7": "op:leo", "sl:U7": "op:leo"}

    # Requirement: Telegram update mapping
    ix = Intake()
    a = TelegramAdapter(ix, IDS)
    r = a.handle(tg(1, 555, "hello"))
    leaked = [k for k in r if k in ("update_id", "chat_id", "thread_ts")]
    check("Telegram update mapping", "A supported update arrives",
          r is not None and not leaked and r["adapter"] == "telegram",
          f"leaked={leaked}")
    check("Telegram update mapping", "A supported update arrives",
          r["raw_payload"]["update_id"] == 1,
          "raw Update retained for audit")

    ix = Intake()
    a = TelegramAdapter(ix, IDS)
    r = a.handle({"update_id": 2, "callback_query": {"id": "cb1"}})
    check("Telegram update mapping", "An unsupported update type arrives",
          len(ix.records) == 1 and r.get("unhandled_variant") == "callback_query",
          "raw payload recorded, variant named, not dropped")

    # Requirement: Delivery identifiers are not trusted for deduplication
    ix = Intake()
    a = TelegramAdapter(ix, IDS)
    first = a.handle(tg(10, 555, "deploy"), classify=lambda t: "status_request")
    later = a.handle(tg(987654321, 555, "deploy"), classify=lambda t: "status_request")
    check("Delivery identifiers are not trusted for deduplication",
          "Telegram randomizes update_id after inactivity",
          later["duplicate_of"] == first["id"],
          "dedupe unaffected by randomized update_id")

    # Requirement: Group activation requires explicit address
    ix = Intake()
    a = TelegramAdapter(ix, IDS)
    r = a.handle(tg(1, -100, "@jcode status?", chat_type="group"))
    check("Group activation requires explicit address",
          "A group message mentions the bot",
          r is not None and len(ix.records) == 1, "forwarded")

    r = a.handle(tg(2, -100, "unrelated chatter", chat_type="group"))
    check("Group activation requires explicit address",
          "A group message does not mention the bot",
          r is None and len(ix.records) == 1, "not forwarded, no record created")

    r = a.handle(tg(3, -100, "sure thing", chat_type="group", reply_bot=True))
    check("Group activation requires explicit address",
          "A group message mentions the bot",
          r is not None, "reply to bot counts as addressing it")

    r = a.handle(tg(4, 555, "no mention needed here"))
    check("Group activation requires explicit address", "A direct message arrives",
          r is not None, "DM forwarded without mention")

    # Requirement: Sender authorization
    ix = Intake()
    a = TelegramAdapter(ix, IDS)
    r = a.handle(tg(1, 555, "deploy prod", user=999),
                 classify=lambda t: "work_request")
    check("Sender authorization", "A sender who is not on the allowlist messages the bot",
          r["classification"] == "unauthorized" and not r["authorized"],
          "recorded as unauthorized")
    check("Sender authorization", "A sender who is not on the allowlist messages the bot",
          len(ix.proposals) == 0 and len(ix.tracked_work) == 0
          and not a.outbound,
          "not promoted, not executed, no repository content returned")

    check("Sender authorization", "A sender who is not on the allowlist messages the bot",
          "999" in r["sender_identity"],
          "sender identifier recorded so the operator can self-configure")

    r = a.handle(tg(2, 555, "deploy prod", user=7),
                 classify=lambda t: "work_request")
    check("Sender authorization", "An allowlisted sender messages the bot",
          r["operator"] == "op:leo" and len(ix.proposals) == 1,
          "forwarded carrying operator identity")

    # Requirement: Outbound delivery and redaction notice
    ix = Intake()
    a = TelegramAdapter(ix, IDS)
    a.handle(tg(1, 555, "ping"), classify=lambda t: "status_request")
    a.deliver("tg:555", "pong")
    check("Outbound delivery and redaction notice", "Intake produces a response",
          a.outbound and a.outbound[-1]["conversation"] == "tg:555",
          "delivered to originating conversation")

    ix = Intake()
    a = TelegramAdapter(ix, IDS)
    token = "123456789:AAHfSHFyTvJmL5RkQxWnPzZbCdEfGhIjKlM"
    a.handle(tg(1, 555, f"my token is {token}"))
    notice = a.outbound[-1]["text"] if a.outbound else ""
    check("Outbound delivery and redaction notice",
          "A credential was redacted at ingress",
          "redact" in notice.lower(), "operator notified")
    check("Outbound delivery and redaction notice",
          "A credential was redacted at ingress",
          token not in notice, "notice does not restate the redacted value")

    # ---- NEUTRALITY BY EXECUTION -----------------------------------------
    # Same intake core, two structurally different adapters.
    ix = Intake()
    tga = TelegramAdapter(ix, IDS)
    sla = SlackAdapter(ix, IDS)

    rt = tga.handle(tg(1, 555, "do the thing"), classify=lambda t: "work_request")
    rs = sla.handle(sl("D01", "do the thing"), classify=lambda t: "work_request")

    check("Adapter neutrality (execution)", "Two adapters share one intake core",
          set(rt) == set(rs),
          "identical envelope shape from different transports")
    check("Adapter neutrality (execution)", "Two adapters share one intake core",
          rt["adapter"] == "telegram" and rs["adapter"] == "slack"
          and rt["operator"] == rs["operator"] == "op:leo",
          "same operator identity resolved through different transports")
    check("Adapter neutrality (execution)", "Two adapters share one intake core",
          rt["duplicate_of"] is None and rs["duplicate_of"] is None,
          "identical text on different transports is not a false duplicate")
    check("Adapter neutrality (execution)", "Two adapters share one intake core",
          len(ix.proposals) == 2,
          "both promote through the same approval path")

    # group gating differs per transport but the core never learns of it
    before = len(ix.records)
    sla.handle(sl("C01", "unrelated channel chatter"))
    tga.handle(tg(9, -100, "unrelated group chatter", chat_type="group"))
    check("Adapter neutrality (execution)", "Transport-specific gating stays in adapters",
          len(ix.records) == before,
          "both adapters gated locally; core unchanged")

    sla.handle(sl("C01", "@jcode status"), classify=lambda t: "status_request")
    check("Adapter neutrality (execution)", "Transport-specific gating stays in adapters",
          len(ix.records) == before + 1, "addressed channel message forwarded")

    # unhandled variants use different vocabulary, same core handling
    ix = Intake()
    sla = SlackAdapter(ix, IDS)
    r = sla.handle({"event": {"type": "reaction_added", "user": "U7"}})
    check("Adapter neutrality (execution)", "Unhandled variants recorded per transport",
          r.get("unhandled_variant") == "reaction_added" and len(ix.records) == 1,
          "Slack variant vocabulary, identical core behavior")

    # the core's public surface never names a transport
    core_fields = set(rt) - {"unhandled_variant", "authorized"}
    transport_words = {"update_id", "chat_id", "thread_ts", "channel_post",
                       "callback_query", "message_thread_id"}
    check("Adapter neutrality (execution)", "Core surface names no transport",
          not (core_fields & transport_words),
          f"fields={sorted(core_fields)}")

    # The boundary is what the adapter HANDS OVER, not only what is stored.
    # Checking the stored record alone misses a leak in the handoff.
    ix = Intake()
    for adapter, raw in ((TelegramAdapter(ix, IDS), tg(1, 555, "x")),
                         (SlackAdapter(ix, IDS), sl("D01", "x"))):
        adapter.handle(raw)
        handed = set(adapter.last_handoff)
        check("Adapter neutrality (execution)",
              "The adapter hands a message to intake",
              not (handed & transport_words),
              f"{adapter.name} hands: {sorted(handed)}")

    # Structural guarantee: intake accepts a fixed named signature, so an
    # adapter cannot smuggle extra fields through. If this ever grows
    # **kwargs, transport leakage becomes possible and this fails.
    import inspect
    sig = inspect.signature(Intake.receive)
    has_kwargs = any(p.kind == inspect.Parameter.VAR_KEYWORD
                     for p in sig.parameters.values())
    check("Adapter neutrality (execution)",
          "The adapter hands a message to intake",
          not has_kwargs,
          f"params={[p for p in sig.parameters if p != 'self']}")


def main():
    run()
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
    print(f"\n{total - failed}/{total} adapter assertions passed")
    if failed:
        print(f"{failed} DEFECT(S)")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
