import { For } from "solid-js";

export interface NavigationItem {
  href: string;
  label: string;
  shortLabel: string;
  count?: number;
  icon: string;
}

export const navigationItems: NavigationItem[] = [
  { href: "/inbox", label: "Decision inbox", shortLabel: "Inbox", count: 0, icon: "⌁" },
  { href: "/ambient", label: "Ambient cycles", shortLabel: "Ambient", count: 0, icon: "◌" },
];

function isActive(href: string, activePath: string) {
  return href === "/inbox"
    ? activePath === "/" || activePath === "/inbox" || activePath.startsWith("/initiatives")
    : activePath === href || activePath.startsWith(`${href}/`);
}

function NavigationLink(props: { item: NavigationItem; activePath: string; mobile?: boolean }) {
  const active = () => isActive(props.item.href, props.activePath);
  return (
    <a
      class={`side-link${active() ? " active" : ""}`}
      classList={{ "mobile-link": props.mobile }}
      href={props.item.href}
      aria-current={active() ? "page" : undefined}
    >
      <span aria-hidden="true">{props.item.icon}</span>
      <b>{props.mobile ? props.item.shortLabel : props.item.label}</b>
      {!props.mobile && <em>{props.item.count ?? ""}</em>}
    </a>
  );
}

export function ConnectionBadge(props: { state: string; reason?: string }) {
  return (
    <div class={`connection ${props.state}`} aria-label={`Connection ${props.state}`} role="status">
      <i aria-hidden="true" />
      <span>
        Jcode {props.state}
        {props.reason ? ` · ${props.reason}` : ""}
      </span>
    </div>
  );
}

export function SideNavigation(props: {
  activePath: string;
  connectionState: string;
  connectionReason?: string;
  onFind: (trigger: HTMLElement) => void;
}) {
  return (
    <aside class="side-nav" aria-label="Primary sidebar">
      <a class="brand" href="/inbox" aria-label="Command Center home">
        <span class="brand-mark" aria-hidden="true">
          ◒
        </span>
        <span>
          <strong>Jcode</strong>
          <small>Command Center</small>
        </span>
      </a>
      <nav class="side-links" aria-label="Command Center">
        <For each={navigationItems}>
          {(item) => <NavigationLink item={item} activePath={props.activePath} />}
        </For>
      </nav>
      <div class="side-foot">
        <ConnectionBadge state={props.connectionState} reason={props.connectionReason} />
        <button
          class="search"
          type="button"
          aria-haspopup="dialog"
          aria-controls="find-drawer"
          onClick={(event) => props.onFind(event.currentTarget)}
        >
          <span aria-hidden="true">⌕</span>
          <span>Find run or receipt</span>
        </button>
      </div>
    </aside>
  );
}

export function MobileNavigation(props: { activePath: string }) {
  return (
    <nav class="mobile-nav" aria-label="Mobile navigation">
      <For each={navigationItems}>
        {(item) => <NavigationLink item={item} activePath={props.activePath} mobile />}
      </For>
    </nav>
  );
}
