import assert from "node:assert/strict";
import test from "node:test";

import {
  buildInventorySnapshot,
  diffInventorySnapshots,
} from "../src/inventory.mjs";

const chrome = {
  browserKind: "chrome",
  displayName: "Google Chrome",
  profileLabel: "Profile 1",
};

function windowWithTabs(tabs, overrides = {}) {
  return {
    id: 4,
    focused: true,
    incognito: false,
    tabs,
    ...overrides,
  };
}

test("builds generation-tagged stable opaque window and tab references", () => {
  const first = buildInventorySnapshot({
    ...chrome,
    generation: 7,
    windows: [
      windowWithTabs([
        {
          id: 9,
          windowId: 4,
          active: true,
          title: "Example",
          url: "https://user:secret@example.com/path?q=private#token",
        },
      ]),
    ],
  });
  const second = buildInventorySnapshot({
    ...chrome,
    generation: 7,
    windows: [
      windowWithTabs([
        {
          id: 9,
          windowId: 4,
          active: false,
          title: "Renamed",
          url: "https://example.com/path?changed=yes",
        },
      ]),
    ],
  });

  assert.equal(first.generation, 7);
  assert.match(first.windows[0].windowRef, /^win_[a-z0-9_-]+$/);
  assert.match(first.windows[0].tabs[0].tabRef, /^tab_[a-z0-9_-]+$/);
  assert.equal(first.windows[0].nativeWindowId, 4);
  assert.equal(first.windows[0].tabs[0].nativeWindowId, 4);
  assert.equal(first.windows[0].tabs[0].nativeTabId, 9);
  assert.equal(first.windows[0].windowRef.includes("4"), false);
  assert.equal(first.windows[0].tabs[0].tabRef.includes("9"), false);
  assert.equal(first.windows[0].windowRef, second.windows[0].windowRef);
  assert.equal(
    first.windows[0].tabs[0].tabRef,
    second.windows[0].tabs[0].tabRef,
  );
  assert.equal(first.windows[0].tabs[0].url, "https://example.com/path");
  assert.deepEqual(first.windows[0].tabs[0].capabilities, [
    "activate_tab",
    "close_tab",
    "navigate",
  ]);
});

test("filters incognito and policy-hidden metadata without widening capabilities", () => {
  const snapshot = buildInventorySnapshot({
    ...chrome,
    generation: 3,
    policy: { hideTitle: true, hidePath: true },
    windows: [
      windowWithTabs([
        {
          id: 1,
          windowId: 4,
          active: true,
          title: "Secret",
          url: "https://example.com/private?q=x",
        },
      ]),
      windowWithTabs(
        [
          {
            id: 2,
            windowId: 5,
            active: true,
            title: "Incognito",
            url: "https://example.com",
          },
        ],
        { id: 5, incognito: true },
      ),
    ],
  });

  assert.equal(snapshot.windows.length, 1);
  assert.equal(snapshot.windows[0].tabs[0].title, undefined);
  assert.equal(snapshot.windows[0].tabs[0].url, "https://example.com");
  assert.equal(
    snapshot.windows[0].tabs[0].capabilities.includes("inspect_content"),
    false,
  );
});

test("omits invalid and privileged URLs rather than leaking their values", () => {
  const snapshot = buildInventorySnapshot({
    ...chrome,
    generation: 1,
    windows: [
      windowWithTabs([
        {
          id: 1,
          windowId: 4,
          active: true,
          title: "Settings",
          url: "chrome://settings/passwords",
        },
        {
          id: 2,
          windowId: 4,
          active: false,
          title: "Broken",
          url: "not a url?secret=1",
        },
      ]),
    ],
  });

  assert.equal(snapshot.windows[0].tabs[0].url, undefined);
  assert.equal(snapshot.windows[0].tabs[0].controllable, false);
  assert.deepEqual(snapshot.windows[0].tabs[0].capabilities, []);
  assert.equal(snapshot.windows[0].tabs[1].url, undefined);
});

test("reports bounded added, updated, and removed inventory deltas", () => {
  const before = buildInventorySnapshot({
    ...chrome,
    generation: 10,
    windows: [
      windowWithTabs([
        { id: 1, windowId: 4, active: true, title: "A", url: "https://a.test" },
      ]),
    ],
  });
  const after = buildInventorySnapshot({
    ...chrome,
    generation: 11,
    windows: [
      windowWithTabs([
        {
          id: 1,
          windowId: 4,
          active: false,
          title: "A2",
          url: "https://a.test/next",
        },
        { id: 2, windowId: 4, active: true, title: "B", url: "https://b.test" },
      ]),
    ],
  });

  const delta = diffInventorySnapshots(before, after, { maxChanges: 3 });
  assert.equal(delta.fromGeneration, 10);
  assert.equal(delta.toGeneration, 11);
  assert.deepEqual(delta.removedTabRefs, []);
  assert.equal(delta.addedTabs.length, 1);
  assert.equal(delta.updatedTabs.length, 1);
  assert.equal(delta.truncated, false);

  const bounded = diffInventorySnapshots(before, after, { maxChanges: 1 });
  assert.equal(bounded.addedTabs.length + bounded.updatedTabs.length, 1);
  assert.equal(bounded.truncated, true);

  const final = buildInventorySnapshot({
    ...chrome,
    generation: 12,
    windows: [
      windowWithTabs([
        { id: 2, windowId: 4, active: true, title: "B", url: "https://b.test" },
      ]),
    ],
  });
  const removal = diffInventorySnapshots(after, final);
  assert.deepEqual(removal.removedTabRefs, [after.windows[0].tabs[0].tabRef]);
});

test("rejects stale or non-monotonic generations", () => {
  const snapshot = buildInventorySnapshot({
    ...chrome,
    generation: 4,
    windows: [],
  });
  assert.throws(
    () => diffInventorySnapshots(snapshot, snapshot),
    /newer inventory generation/,
  );
});
