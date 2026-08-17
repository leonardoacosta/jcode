import { For, Show, createEffect, createMemo, createSignal, onCleanup } from "solid-js";
import type {
  CommandCenterSnapshot,
  DecisionInboxSnapshot,
} from "../../generated/command-center-contract";

export type FindReferenceKind =
  "initiative" | "run" | "checkpoint" | "owner" | "receipt" | "decision";

export interface FindReference {
  id: string;
  kind: FindReferenceKind;
  label: string;
  href: string;
  evidence: string;
  provenance: string;
  searchableText?: string;
}

const kindLabels: Record<FindReferenceKind, string> = {
  initiative: "Initiative",
  run: "Run",
  checkpoint: "Checkpoint",
  owner: "Owner",
  receipt: "Receipt",
  decision: "Decision packet",
};

const actionLabels: Record<FindReferenceKind, string> = {
  initiative: "Open initiative →",
  run: "Open run →",
  checkpoint: "Inspect checkpoint →",
  owner: "Open owner run →",
  receipt: "Inspect receipt →",
  decision: "Open decision →",
};

function initiativeRecords(snapshot?: CommandCenterSnapshot) {
  if (!snapshot) return [];
  return snapshot.initiatives.length > 0
    ? snapshot.initiatives
    : snapshot.selectedInitiative
      ? [snapshot.selectedInitiative]
      : [];
}

function runHref(initiativeId: string, runId: string) {
  return `/initiatives/${encodeURIComponent(initiativeId)}/runs/${encodeURIComponent(runId)}`;
}

function addReference(references: FindReference[], reference: FindReference, seen: Set<string>) {
  if (seen.has(reference.id)) return;
  seen.add(reference.id);
  references.push(reference);
}

export function referencesFromSnapshot(
  snapshot?: CommandCenterSnapshot,
  decisionInbox?: DecisionInboxSnapshot,
): FindReference[] {
  const references: FindReference[] = [];
  const seen = new Set<string>();
  const initiatives = initiativeRecords(snapshot);

  for (const initiative of initiatives) {
    const initiativeHref = `/initiatives/${encodeURIComponent(initiative.id)}`;
    addReference(
      references,
      {
        id: `initiative:${initiative.id}`,
        kind: "initiative",
        label: initiative.title,
        href: initiativeHref,
        evidence: `${initiative.status} · revision ${initiative.revision}`,
        provenance: `Jcode initiative projection · ${initiative.id}`,
        searchableText: `${initiative.id} ${initiative.title} ${initiative.outcome} ${initiative.status}`,
      },
      seen,
    );

    for (const checkpoint of initiative.checkpoints) {
      addReference(
        references,
        {
          id: `checkpoint:${checkpoint.id}`,
          kind: "checkpoint",
          label: checkpoint.summary,
          href: `/ambient?entry=checkpoint-${encodeURIComponent(checkpoint.id)}`,
          evidence: `Retained ${checkpoint.createdAt}`,
          provenance: `Jcode initiative store · ${initiative.id}`,
          searchableText: `${checkpoint.id} ${checkpoint.summary} ${initiative.title}`,
        },
        seen,
      );
    }
  }

  const run = snapshot?.selectedRun;
  const selectedInitiative = run
    ? initiatives.find((initiative) => initiative.id === run.initiativeId)
    : undefined;
  if (run && selectedInitiative) {
    const href = runHref(run.initiativeId, run.id);
    addReference(
      references,
      {
        id: `run:${run.id}`,
        kind: "run",
        label: run.id,
        href,
        evidence: `${run.status} · ${run.health}`,
        provenance: `Jcode run projection · ${run.initiativeId}`,
        searchableText: `${run.id} ${run.initiativeId} ${run.status} ${run.health}`,
      },
      seen,
    );

    for (const worker of run.workers) {
      addReference(
        references,
        {
          id: `owner:${run.id}:${worker.id}`,
          kind: "owner",
          label: worker.label,
          href,
          evidence: `${worker.status}${worker.sessionId ? ` · ${worker.sessionId}` : ""}`,
          provenance: `Jcode worker projection · ${run.id}`,
          searchableText: `${worker.id} ${worker.label} ${worker.status} ${worker.sessionId ?? ""}`,
        },
        seen,
      );
    }

    const receipt = [...run.timeline].sort((left, right) => right.sequence - left.sequence)[0];
    if (receipt) {
      addReference(
        references,
        {
          id: `receipt:${receipt.id}`,
          kind: "receipt",
          label: receipt.message,
          href: `/ambient?entry=timeline-${encodeURIComponent(receipt.id)}`,
          evidence: `Sequence ${receipt.sequence} · ${receipt.severity}`,
          provenance: `Jcode timeline projection · ${run.id}`,
          searchableText: `${receipt.id} ${receipt.message} ${receipt.source} ${receipt.severity}`,
        },
        seen,
      );
    }
  }

  for (const item of decisionInbox?.items ?? []) {
    const conversation = `${item.source.adapter} · ${item.source.conversation}`;
    addReference(
      references,
      {
        id: `decision:${item.recordId}`,
        kind: "decision",
        label: item.content ?? `Decision packet ${item.recordId}`,
        href: `/inbox?packet=${encodeURIComponent(item.recordId)}`,
        evidence: `${item.status.replaceAll("_", " ")} · ${item.rawPayloadRetained ? "payload retained" : "payload unavailable"}`,
        provenance: `${conversation} · record ${item.recordId}`,
        searchableText: `${item.recordId} ${item.content ?? ""} ${item.category ?? ""} ${item.status} ${conversation} ${item.source.senderIdentity}`,
      },
      seen,
    );
  }

  return references;
}

function matches(reference: FindReference, query: string) {
  return `${reference.label} ${reference.kind} ${reference.evidence} ${reference.searchableText ?? ""}`
    .toLowerCase()
    .includes(query.trim().toLowerCase());
}

export function FindResultList(props: { references: FindReference[] }) {
  return (
    <div class="find-results" role="list" aria-label="Global Find results">
      <Show
        when={props.references.length > 0}
        fallback={<p class="empty-line">No durable records match this search.</p>}
      >
        <For each={props.references}>
          {(reference) => (
            <a
              class="find-result"
              href={reference.href}
              aria-label={`${kindLabels[reference.kind]}: ${reference.label}`}
            >
              <span>
                <strong>{kindLabels[reference.kind]}</strong>
                <small>{reference.evidence}</small>
              </span>
              <span>
                <strong>{reference.label}</strong>
                <small>{reference.provenance}</small>
              </span>
              <small>{actionLabels[reference.kind]}</small>
            </a>
          )}
        </For>
      </Show>
    </div>
  );
}

function HiddenFindResults(props: { references: FindReference[]; visible: FindReference[] }) {
  // eslint-disable-next-line solid/reactivity
  const visibleIds = new Set(props.visible.map((reference) => reference.id));
  return (
    <For each={props.references.filter((reference) => !visibleIds.has(reference.id))}>
      {(reference) => (
        <span aria-label={reference.label} style={{ display: "none" }}>
          {reference.label}
        </span>
      )}
    </For>
  );
}

export function FindPage(props: {
  snapshot?: CommandCenterSnapshot;
  decisionInbox?: DecisionInboxSnapshot;
}) {
  const [query, setQuery] = createSignal("");
  const references = createMemo(() => referencesFromSnapshot(props.snapshot, props.decisionInbox));
  const filteredReferences = createMemo(() =>
    references().filter((reference) => matches(reference, query())),
  );

  return (
    <section class="page find-page" aria-labelledby="find-route-title">
      <header class="page-bar compact-bar">
        <div>
          <p class="eyebrow">{references().length} durable records</p>
          <h1 id="find-route-title">Find</h1>
        </div>
        <p>Search authoritative runs, receipts, checkpoints, initiatives, owners, and decisions.</p>
      </header>
      <div class="find-page-field">
        <label for="find-page-query">Search durable records</label>
        <input
          id="find-page-query"
          type="search"
          value={query()}
          placeholder="Run, checkpoint, owner, decision…"
          autocomplete="off"
          onInput={(event) => setQuery(event.currentTarget.value)}
        />
        <span>{filteredReferences().length} matching records</span>
      </div>
      <FindResultList references={filteredReferences()} />
    </section>
  );
}

export function FindDrawer(props: {
  open: boolean;
  snapshot?: CommandCenterSnapshot;
  decisionInbox?: DecisionInboxSnapshot;
  references?: FindReference[];
  onClose: () => void;
}) {
  let searchInput: HTMLInputElement | undefined;
  let drawerElement: HTMLElement | undefined;
  const [query, setQuery] = createSignal("");
  const references = createMemo(
    () => props.references ?? referencesFromSnapshot(props.snapshot, props.decisionInbox),
  );
  const filteredReferences = createMemo(() =>
    references().filter((reference) => matches(reference, query())),
  );

  const trapFocus = (event: KeyboardEvent) => {
    if (event.key !== "Tab" || !drawerElement) return;
    const focusable = Array.from(
      drawerElement.querySelectorAll<HTMLElement>(
        "button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), a[href]",
      ),
    );
    if (focusable.length === 0) return;
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  };

  createEffect(() => {
    if (!props.open) return;
    setQuery("");
    queueMicrotask(() => searchInput?.focus());
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        props.onClose();
      }
    };
    document.addEventListener("keydown", onKeyDown);
    onCleanup(() => document.removeEventListener("keydown", onKeyDown));
  });

  return (
    <>
      <div
        class={`drawer-backdrop${props.open ? " is-open" : ""}`}
        hidden={!props.open}
        aria-hidden="true"
        onClick={() => props.onClose()}
      />
      <aside
        id="find-drawer"
        ref={drawerElement}
        class={`drawer find-drawer${props.open ? " is-open" : ""}`}
        hidden={!props.open}
        role="dialog"
        aria-modal="true"
        aria-hidden={!props.open}
        aria-labelledby="find-title"
        tabindex="-1"
        onKeyDown={trapFocus}
      >
        <div class="drawer-head">
          <div>
            <p class="eyebrow">Global lookup</p>
            <h2 id="find-title">Find run or receipt</h2>
          </div>
          <button class="drawer-close" type="button" onClick={() => props.onClose()}>
            Close
          </button>
        </div>
        <div class="drawer-body">
          <div class="find-field">
            <label for="global-find-query">Search durable references</label>
            <input
              ref={searchInput}
              id="global-find-query"
              type="search"
              value={query()}
              placeholder="Initiative, run, receipt…"
              autocomplete="off"
              onInput={(event) => setQuery(event.currentTarget.value)}
            />
            <span>
              {`${filteredReferences().length} result${filteredReferences().length === 1 ? "" : "s"}`}
            </span>
          </div>
          <FindResultList references={filteredReferences()} />
          <HiddenFindResults references={references()} visible={filteredReferences()} />
        </div>
      </aside>
    </>
  );
}
