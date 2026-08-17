import { For, createEffect, createMemo, createSignal, onCleanup } from "solid-js";
import type { CommandCenterSnapshot } from "../../generated/command-center-contract";

export interface FindReference {
  id: string;
  kind: string;
  label: string;
  href: string;
  searchableText?: string;
}

export function referencesFromSnapshot(snapshot?: CommandCenterSnapshot): FindReference[] {
  const initiative = snapshot?.selectedInitiative;
  const run = snapshot?.selectedRun;
  if (!initiative) return [];

  return [
    {
      id: initiative.id,
      kind: "initiative",
      label: initiative.title,
      href: `/initiatives/${initiative.id}`,
      searchableText: `${initiative.id} ${initiative.title} ${initiative.outcome}`,
    },
    ...(run
      ? [
          {
            id: run.id,
            kind: "run",
            label: run.id,
            href: `/initiatives/${run.initiativeId}/runs/${run.id}`,
            searchableText: `${run.id} ${run.initiativeId} ${run.status}`,
          },
        ]
      : []),
  ];
}

export function FindDrawer(props: {
  open: boolean;
  snapshot?: CommandCenterSnapshot;
  references?: FindReference[];
  onClose: () => void;
}) {
  let searchInput: HTMLInputElement | undefined;
  const [query, setQuery] = createSignal("");
  const references = () => props.references ?? referencesFromSnapshot(props.snapshot);
  const filteredReferences = createMemo(() => {
    const normalized = query().trim().toLowerCase();
    return references().filter((reference) =>
      !normalized
        ? true
        : `${reference.label} ${reference.kind} ${reference.searchableText ?? ""}`
            .toLowerCase()
            .includes(normalized),
    );
  });
  const isVisible = (reference: FindReference) =>
    filteredReferences().some((item) => item.id === reference.id);

  createEffect(() => {
    if (!props.open) return;
    queueMicrotask(() => searchInput?.focus());
  });

  createEffect(() => {
    if (!props.open) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") props.onClose();
    };
    document.addEventListener("keydown", onKeyDown);
    onCleanup(() => document.removeEventListener("keydown", onKeyDown));
  });

  return (
    <>
      <div
        class={`drawer-backdrop${props.open ? " is-open" : ""}`}
        style={{ display: props.open ? "block" : "none" }}
        onClick={() => props.onClose()}
      />
      <aside
        id="find-drawer"
        class={`drawer find-drawer${props.open ? " is-open" : ""}`}
        style={{ display: props.open ? "flex" : "none" }}
        role="dialog"
        aria-modal="true"
        aria-labelledby="find-title"
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
              {filteredReferences().length} result{filteredReferences().length === 1 ? "" : "s"}
            </span>
          </div>
          <div class="find-results">
            <For each={references()} fallback={<p class="empty-line">No durable references.</p>}>
              {(reference) => (
                <a
                  class="find-result"
                  href={reference.href}
                  aria-label={reference.label}
                  style={{
                    display: "grid",
                    opacity: isVisible(reference) ? 1 : 0,
                    "pointer-events": isVisible(reference) ? "auto" : "none",
                  }}
                >
                  <span>{reference.kind}</span>
                  <strong>{reference.label}</strong>
                  <small>Open reference →</small>
                </a>
              )}
            </For>
          </div>
        </div>
      </aside>
    </>
  );
}
