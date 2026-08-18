import { For, Show, createEffect, createMemo, createSignal } from "solid-js";
import type {
  DecisionInboxSnapshot,
  DecisionInboxItem,
} from "../generated/command-center-contract";

export { AppShell } from "./shell/AppShell";
export { FindDrawer, FindPage, referencesFromSnapshot } from "./shell/FindDrawer";
export { AmbientActivity } from "./AmbientActivity";

const label = (value: string) =>
  value.replaceAll("_", " ").replace(/^./, (first) => first.toUpperCase());

type InboxFilter = "all" | "approval" | "question" | "revisit" | "receipt";

const filterLabels: Array<[InboxFilter, string]> = [
  ["all", "All"],
  ["approval", "Approvals"],
  ["question", "Questions"],
  ["revisit", "Revisits"],
  ["receipt", "Receipts"],
];

function packetKind(item: DecisionInboxItem): Exclude<InboxFilter, "all"> {
  if (item.status === "awaiting_approval" || item.proposal) return "approval";
  if (item.category === "status_request") return "question";
  if (
    item.status === "deferred" ||
    item.status === "unrecognized" ||
    item.status === "classification_failed" ||
    item.category === "unrecognized"
  ) {
    return "revisit";
  }
  return "receipt";
}

function packetDate(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(date);
}

/** Compact age used by the queue list and header, matching the proposal mock. */
function packetAge(value: string, now = Date.now()) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  const seconds = Math.max(0, Math.round((now - date.getTime()) / 1000));
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.round(seconds / 60);
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.round(minutes / 60);
  if (hours < 24) return `${hours}h`;
  return `${Math.round(hours / 24)}d`;
}

function packetEvidence(item: DecisionInboxItem) {
  const delivery = [
    item.duplicateDeliveries > 0
      ? `${item.duplicateDeliveries} duplicate ${item.duplicateDeliveries === 1 ? "delivery" : "deliveries"} retained`
      : undefined,
    item.retryDeliveries > 0
      ? `${item.retryDeliveries} retry ${item.retryDeliveries === 1 ? "delivery" : "deliveries"}`
      : undefined,
  ].filter(Boolean);
  return delivery.length > 0 ? delivery.join(" · ") : "Single delivery recorded";
}

export function DecisionInbox(props: {
  snapshot?: DecisionInboxSnapshot;
  initialRecordId?: number;
  loading?: boolean;
  error?: unknown;
}) {
  const [filter, setFilter] = createSignal<InboxFilter>("all");
  const [sort, setSort] = createSignal<"newest" | "oldest">("newest");
  const [selectedId, setSelectedId] = createSignal<number | undefined>(props.initialRecordId);
  createEffect(() => setSelectedId(props.initialRecordId));
  const items = () => props.snapshot?.items ?? [];
  const visiblePackets = createMemo(() => {
    const filtered = items().filter((item) => filter() === "all" || packetKind(item) === filter());
    const direction = sort() === "newest" ? -1 : 1;
    return filtered.sort((left, right) => {
      return direction * (Date.parse(left.receivedAt) - Date.parse(right.receivedAt));
    });
  });
  const selectedPacket = createMemo(() => {
    const selected = selectedId();
    return selected === undefined
      ? visiblePackets()[0]
      : visiblePackets().find((item) => item.recordId === selected);
  });
  const countFor = (kind: InboxFilter) =>
    kind === "all" ? items().length : items().filter((item) => packetKind(item) === kind).length;

  return (
    <section aria-labelledby="decision-inbox-title" class="decision-inbox">
      <header class="page-bar compact-bar">
        <div>
          <p class="eyebrow">{items().length} open</p>
          <h1 id="decision-inbox-title">Decision queue</h1>
        </div>
        <span class="toolbar-note">
          Updated {props.snapshot ? `${packetAge(props.snapshot.generatedAt)} ago` : "when connected"}
        </span>
      </header>

      <div class="toolbar queue-toolbar" aria-label="Decision queue controls">
        <div class="filter-group" role="group" aria-label="Filter by type">
          <For each={filterLabels}>
            {([kind, name]) => (
              <button
                class={`tab ${filter() === kind ? "active" : ""}`}
                type="button"
                aria-pressed={filter() === kind}
                onClick={() => setFilter(kind)}
              >
                {name}
                <Show when={kind === "all"}> {countFor(kind)}</Show>
              </button>
            )}
          </For>
        </div>
        <label class="sort-control">
          Sort
          <select
            aria-label="Sort packets"
            value={sort()}
            onChange={(event) => setSort(event.currentTarget.value as "newest" | "oldest")}
          >
            <option value="newest">Newest first</option>
            <option value="oldest">Oldest first</option>
          </select>
        </label>
      </div>

      <div class="workspace inbox-workspace">
        <section class="surface packet-surface" aria-labelledby="packet-list-title">
          <div class="surface-head">
            <h2 id="packet-list-title">Durable packets</h2>
            <span>Telegram-backed</span>
          </div>
          <div class="inbox-list staggered-list" role="list" aria-label="Durable decision packets">
            <Show
              when={!props.loading}
              fallback={<p class="empty-line">Loading durable packets…</p>}
            >
              <Show
                when={!props.error}
                fallback={
                  <p class="empty-line" role="alert">
                    Decision packets could not be loaded. Retry after the daemon recovers.
                  </p>
                }
              >
                <Show
                  when={visiblePackets().length > 0}
                  fallback={
                    <p class="empty-line">
                      {items().length === 0
                        ? "No durable packets are available."
                        : "No packets match this filter."}
                    </p>
                  }
                >
                  <For each={visiblePackets()}>
                    {(item, index) => (
                      <button
                        type="button"
                        class={`inbox-item ${selectedPacket()?.recordId === item.recordId ? "active" : ""}`}
                        style={{ "--packet-index": index() }}
                        aria-label={`${item.content ?? "Untitled"} packet`}
                        aria-pressed={selectedPacket()?.recordId === item.recordId}
                        onClick={() => setSelectedId(item.recordId)}
                      >
                        <i class={`signal ${packetKind(item)}`} aria-hidden="true" />
                        <span>
                          <span class="item-type">
                            <span>{item.category ? label(item.category) : "Unclassified"}</span> ·{" "}
                            <span>{label(packetKind(item))}</span>
                          </span>
                          <h3 aria-hidden="true">{item.content ?? "No text content"}</h3>
                          <p>
                            {item.source.senderIdentity} · {item.source.conversation}
                          </p>
                          <span class="meta">
                            <span>{label(item.status)}</span>
                            <span>{packetEvidence(item)}</span>
                            <span>{item.redacted ? "Content redacted" : "Content retained"}</span>
                          </span>
                        </span>
                        <time class="item-time" dateTime={item.receivedAt} title={packetDate(item.receivedAt)}>
                          {packetAge(item.receivedAt)}
                        </time>
                      </button>
                    )}
                  </For>
                </Show>
              </Show>
            </Show>
            <Show when={filter() !== "all"}>
              <For each={items().filter((item) => packetKind(item) !== filter())}>
                {(item) => <span hidden>{item.content ?? "No text content"}</span>}
              </For>
            </Show>
          </div>
        </section>

        <section
          class="surface detail decision-detail"
          role="dialog"
          aria-modal="true"
          aria-labelledby="decision-detail-title"
          aria-describedby="decision-detail-note"
        >
          <button
            class="mobile-sheet-close"
            type="button"
            aria-label={items().length === 0 ? "Back to packet list" : "Back to queue"}
            onClick={() => setSelectedId(undefined)}
          >
            ← Decision queue
          </button>
          <Show
            when={selectedPacket()}
            fallback={
              <>
                <div class="detail-top">
                  <div>
                    <p class="eyebrow">Packet detail</p>
                    <h2 id="decision-detail-title">Decision packet detail</h2>
                    <p class="detail-sub">Select a durable packet to inspect its evidence.</p>
                  </div>
                </div>
                <p id="decision-detail-note" class="decision-note" role="status">
                  No packet is selected.
                </p>
              </>
            }
          >
            {(item) => (
              <>
                <div class="detail-top">
                  <div>
                    <p class="eyebrow" id="detail-meta">
                      {label(item().source.adapter)} · {item().source.conversation}
                    </p>
                    <h2 id="decision-detail-title">
                      {packetKind(item()) === "approval"
                        ? (item().content ?? "No text content")
                        : `${label(packetKind(item()))}: ${item().content ?? "No text content"}`}
                    </h2>
                    <p class="detail-sub">
                      Received {packetDate(item().receivedAt)} from {item().source.senderIdentity}
                    </p>
                  </div>
                  <span class="chip">{label(packetKind(item()))}</span>
                </div>
                <div class="actions" aria-label="Packet actions">
                  <button type="button" class="button primary" disabled title="Unsupported action">
                    Approve delivery
                  </button>
                  <button type="button" class="button" disabled title="Unsupported action">
                    Open source
                  </button>
                </div>
                <p id="decision-detail-note" class="decision-note" role="status">
                  Actions are disabled because delivery actions are unsupported by the current inbox
                  transport.
                </p>
                <div class="phase-line" aria-label="Packet lifecycle">
                  <span class="phase done">Received</span>
                  <span class="phase done">Classified</span>
                  <span class="phase current">Reviewed</span>
                  <span class="phase">Executed</span>
                  <span class="phase">Accepted</span>
                </div>
                <div class="detail-body">
                  <section class="detail-section">
                    <div class="section-heading">
                      <h3>Source</h3>
                      <p>Durable origin retained by the inbox projection.</p>
                    </div>
                    <div class="detail-grid">
                      <div class="info-pair">
                        <small>Adapter</small>
                        <p>{label(item().source.adapter)}</p>
                      </div>
                      <div class="info-pair">
                        <small>Conversation</small>
                        <p>{item().source.conversation}</p>
                      </div>
                      <div class="info-pair">
                        <small>Sender identity</small>
                        <p>{item().source.senderIdentity}</p>
                      </div>
                    </div>
                  </section>
                  <section class="detail-section">
                    <div class="section-heading">
                      <h3>Authority</h3>
                      <p>Authority is never inferred from an external message.</p>
                    </div>
                    <p class="detail-copy">
                      No authority claim is present in the generated inbox contract.
                    </p>
                  </section>
                  <section class="detail-section">
                    <div class="section-heading">
                      <h3>Execution</h3>
                      <p>Execution state remains separate from acceptance.</p>
                    </div>
                    <p class="detail-copy">This packet does not authorize or start execution.</p>
                  </section>
                  <section class="detail-section">
                    <div class="section-heading">
                      <h3>Acceptance</h3>
                      <p>Acceptance must be evidenced by a separate authoritative record.</p>
                    </div>
                    <p class="detail-copy">No acceptance result is inferred from packet status.</p>
                  </section>
                  <section class="detail-section">
                    <div class="section-heading">
                      <h3>Evidence</h3>
                      <p>Delivery and retention facts from the Telegram-backed transport.</p>
                    </div>
                    <div class="detail-grid">
                      <div class="info-pair">
                        <small>Delivery</small>
                        <p>Record: {packetEvidence(item())}</p>
                      </div>
                      <div class="info-pair">
                        <small>Dedupe key</small>
                        <p>
                          <code>{item().dedupeKey}</code>
                        </p>
                      </div>
                      <div class="info-pair">
                        <small>Payload</small>
                        <p>
                          {item().rawPayloadRetained
                            ? "Raw payload retained"
                            : "Raw payload not retained"}
                        </p>
                      </div>
                    </div>
                  </section>
                  <section class="detail-section">
                    <div class="section-heading">
                      <h3>Owner trail</h3>
                      <p>Only explicit transport identities are shown.</p>
                    </div>
                    <p class="detail-copy">
                      {item().source.senderIdentity} is the recorded sender. No owner is inferred.
                    </p>
                  </section>
                  <section class="detail-section">
                    <div class="section-heading">
                      <h3>Blast radius and rollback</h3>
                      <p>Bounded actions fail closed until an authoritative command exists.</p>
                    </div>
                    <p class="detail-copy">
                      No blast-radius or rollback record is available from the current inbox
                      transport.
                    </p>
                  </section>
                </div>
              </>
            )}
          </Show>
        </section>
      </div>
    </section>
  );
}
