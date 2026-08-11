import assert from "node:assert/strict";
import test from "node:test";

import {
  ACTION_MESSAGE_TYPES,
  detectBrowserKind,
  installActionMessageHandler,
  installMacBrowserFleetBridge,
} from "../src/background.mjs";
import { ACTIONS, ERROR_CODES } from "../src/action-handler.mjs";

function fakePort() {
  let messageListener;
  let disconnectListener;
  const posted = [];
  return {
    posted,
    onMessage: {
      addListener(next) {
        messageListener = next;
      },
    },
    onDisconnect: {
      addListener(next) {
        disconnectListener = next;
      },
    },
    receive(message) {
      return messageListener(message);
    },
    disconnect() {
      return disconnectListener?.();
    },
    postMessage(message) {
      posted.push(message);
    },
  };
}

function event() {
  const listeners = new Set();
  return {
    listeners,
    addListener(listener) {
      listeners.add(listener);
    },
    removeListener(listener) {
      listeners.delete(listener);
    },
    emit(...args) {
      for (const listener of [...listeners]) listener(...args);
    },
  };
}

function fakeBrowserApi({ windows = [] } = {}) {
  const calls = [];
  const ports = [];
  const api = {
    calls,
    ports,
    runtime: {
      connectNative(hostName) {
        calls.push(["runtime.connectNative", hostName]);
        const port = fakePort();
        ports.push(port);
        return port;
      },
      getManifest() {
        return { version: "1.2.3" };
      },
    },
    tabs: {
      onCreated: event(),
      onUpdated: event(),
      onRemoved: event(),
      onActivated: event(),
      async update(tabId, properties) {
        calls.push(["tabs.update", tabId, properties]);
        return { id: tabId };
      },
    },
    windows: {
      onFocusChanged: event(),
      onRemoved: event(),
      async getAll(options) {
        calls.push(["windows.getAll", options]);
        return structuredClone(windows);
      },
    },
  };
  return api;
}

function ordinaryWindow(id, tabs = [{ id: 10, active: true, url: "https://example.test/path?secret=1", title: "Example" }]) {
  return { id, focused: true, incognito: false, tabs };
}

function flushAsyncWork() {
  return new Promise((resolve) => setImmediate(resolve));
}

function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

test("detects Edge identity from the extension runtime user agent", () => {
  assert.equal(detectBrowserKind("Mozilla/5.0 Chrome/151.0 Edg/151.0"), "edge");
  assert.equal(detectBrowserKind("Mozilla/5.0 Chrome/151.0"), "chrome");
});

test("dispatches native action requests and posts a typed response", async () => {
  const port = fakePort();
  const browserApi = fakeBrowserApi();
  installActionMessageHandler(port, browserApi);

  await port.receive({
    type: ACTION_MESSAGE_TYPES.REQUEST,
    requestId: "req-9",
    action: ACTIONS.NAVIGATE,
    target: { tabId: 3 },
    payload: { url: "https://example.test" },
  });

  assert.deepEqual(browserApi.calls.filter(([name]) => name === "tabs.update"), [
    ["tabs.update", 3, { url: "https://example.test" }],
  ]);
  assert.deepEqual(port.posted, [
    {
      type: ACTION_MESSAGE_TYPES.RESPONSE,
      requestId: "req-9",
      ok: true,
      result: { tabId: 3 },
    },
  ]);
});

test("ignores non-action native messages", async () => {
  const port = fakePort();
  const browserApi = fakeBrowserApi();
  installActionMessageHandler(port, browserApi);

  await port.receive({ type: "inventory_request", requestId: "req-10" });

  assert.deepEqual(browserApi.calls, []);
  assert.deepEqual(port.posted, []);
});

test("posts a bounded failure for malformed action requests", async () => {
  const port = fakePort();
  installActionMessageHandler(port, fakeBrowserApi());

  await port.receive({
    type: ACTION_MESSAGE_TYPES.REQUEST,
    requestId: "req-11",
    action: ACTIONS.NAVIGATE,
    target: {},
    payload: { url: "https://user:password@example.test/?token=secret" },
  });

  assert.equal(port.posted[0].type, ACTION_MESSAGE_TYPES.RESPONSE);
  assert.equal(port.posted[0].ok, false);
  assert.equal(port.posted[0].error.code, ERROR_CODES.INVALID_REQUEST);
  assert.equal(JSON.stringify(port.posted).includes("password"), false);
  assert.equal(JSON.stringify(port.posted).includes("token"), false);
});

test("posts hello and initial browser inventory snapshot after native connection", async () => {
  const browserApi = fakeBrowserApi({ windows: [ordinaryWindow(1)] });

  installMacBrowserFleetBridge(browserApi, { browserKind: "chrome", sessionId: "sess-1" });
  await flushAsyncWork();

  const port = browserApi.ports[0];
  assert.deepEqual(port.posted[0], {
    type: "hello",
    protocolVersion: 1,
    browserKind: "chrome",
    extensionVersion: "1.2.3",
    sessionId: "sess-1",
  });
  assert.equal(port.posted[1].type, "inventory_snapshot");
  assert.equal(port.posted[1].snapshot.browserKind, "chrome");
  assert.equal(port.posted[1].snapshot.generation, 1);
  assert.equal(port.posted[1].snapshot.windows.length, 1);
  assert.equal(port.posted[1].snapshot.windows[0].tabs[0].url, "https://example.test/path");
});

test("event-driven inventory updates are generation tagged and bounded", async () => {
  const windows = [ordinaryWindow(1, [{ id: 10, active: true, url: "https://a.test", title: "A" }])];
  const browserApi = fakeBrowserApi({ windows });

  installMacBrowserFleetBridge(browserApi, { browserKind: "chrome", sessionId: "sess-2", maxChanges: 1 });
  await flushAsyncWork();
  windows[0].tabs = [
    { id: 10, active: false, url: "https://b.test", title: "B" },
    { id: 11, active: true, url: "https://c.test", title: "C" },
  ];
  browserApi.tabs.onUpdated.emit(10, { url: "https://b.test" }, windows[0].tabs[0]);
  await flushAsyncWork();

  const port = browserApi.ports[0];
  const delta = port.posted.at(-1);
  assert.equal(delta.type, "inventory_delta");
  assert.equal(delta.browserKind, "chrome");
  assert.equal(delta.fromGeneration, 1);
  assert.equal(delta.toGeneration, 2);
  assert.equal(delta.addedTabs.length + delta.updatedTabs.length + delta.removedTabRefs.length, 1);
  assert.equal(delta.truncated, true);
});

test("reconnect resets request tracking and publishes a fresh initial snapshot", async () => {
  const browserApi = fakeBrowserApi({ windows: [ordinaryWindow(1)] });

  installMacBrowserFleetBridge(browserApi, { browserKind: "edge", sessionId: "sess-3" });
  await flushAsyncWork();
  const firstPort = browserApi.ports[0];
  await firstPort.receive({ type: "inventory_request", requestId: "same-id" });
  firstPort.disconnect();
  await flushAsyncWork();

  assert.equal(browserApi.ports.length, 2);
  const secondPort = browserApi.ports[1];
  assert.equal(secondPort.posted[1].type, "inventory_snapshot");
  assert.equal(secondPort.posted[1].snapshot.generation, 1);
  await secondPort.receive({ type: "inventory_request", requestId: "same-id" });
  assert.equal(secondPort.posted.at(-1).type, "inventory_snapshot");
});

test("disconnect cleanup makes late events safe while preserving action handling before disconnect", async () => {
  const browserApi = fakeBrowserApi({ windows: [ordinaryWindow(1)] });

  const bridge = installMacBrowserFleetBridge(browserApi, { browserKind: "chrome", sessionId: "sess-4", reconnect: false });
  await flushAsyncWork();
  const port = browserApi.ports[0];
  await port.receive({
    type: "action_request",
    requestId: "act-1",
    generation: 1,
    action: ACTIONS.NAVIGATE,
    target: { tabId: 10 },
    payload: { url: "https://after-action.test" },
  });
  assert.equal(port.posted.at(-1).type, "action_response");
  assert.deepEqual(browserApi.calls.filter(([name]) => name === "tabs.update"), [
    ["tabs.update", 10, { url: "https://after-action.test" }],
  ]);

  port.disconnect();
  browserApi.tabs.onCreated.emit({ id: 12, active: false, url: "https://late.test", title: "Late" });
  await flushAsyncWork();

  assert.equal(port.posted.filter((message) => message.type === "inventory_delta").length, 0);
  assert.equal(browserApi.tabs.onCreated.listeners.size, 0);
  assert.doesNotThrow(() => bridge.disconnect());
});

test("periodically polls native host and executes action requests with host-only numeric IDs", async () => {
  const browserApi = fakeBrowserApi({ windows: [ordinaryWindow(7)] });

  const bridge = installMacBrowserFleetBridge(browserApi, {
    browserKind: "chrome",
    sessionId: "sess-poll",
    actionPollIntervalMs: 5,
    reconnect: false,
  });
  await flushAsyncWork();
  await delay(12);

  const port = browserApi.ports[0];
  assert.ok(port.posted.some((message) => message.type === "action_poll"));
  assert.equal(port.posted[1].snapshot.windows[0].nativeWindowId, 7);
  assert.equal(port.posted[1].snapshot.windows[0].tabs[0].nativeTabId, 10);
  assert.equal(port.posted[1].snapshot.windows[0].tabs[0].tabRef.includes("10"), false);

  await port.receive({ type: "action_idle" });
  await port.receive({
    type: "action_request",
    requestId: "act-poll-1",
    generation: 1,
    action: ACTIONS.NAVIGATE,
    target: { windowId: 7, tabId: 10 },
    payload: { url: "https://polled-action.test" },
  });

  assert.deepEqual(browserApi.calls.filter(([name]) => name === "tabs.update"), [
    ["tabs.update", 10, { url: "https://polled-action.test" }],
  ]);
  assert.equal(port.posted.at(-1).type, "action_response");
  const postedBeforeDisconnect = port.posted.length;
  bridge.disconnect();
  await delay(12);
  assert.equal(port.posted.length, postedBeforeDisconnect);
});

test("registers an alarm so a suspended service worker reconnects", () => {
  // MV3 suspends the service worker when idle. An open native port keeps it
  // alive, but once the browser tears the worker down nothing restarts it, and
  // that browser silently vanishes from the fleet until someone reloads the
  // extension. A periodic alarm is the supported way to wake it back up.
  const alarmCalls = [];
  const alarmEvent = event();
  const browserApi = fakeBrowserApi();
  browserApi.alarms = {
    create(name, info) {
      alarmCalls.push([name, info]);
    },
    onAlarm: alarmEvent,
  };

  installMacBrowserFleetBridge(browserApi, {
    browserKind: "chrome",
    sessionId: "sess-alarm",
  });

  assert.equal(alarmCalls.length, 1, "expected a keepalive alarm to be created");
  const [name, info] = alarmCalls[0];
  assert.ok(typeof name === "string" && name.length > 0);
  assert.ok(
    info.periodInMinutes > 0 && info.periodInMinutes <= 1,
    "keepalive alarm must fire at least once a minute",
  );

  // Losing the port must not be terminal: an alarm tick reconnects.
  const before = browserApi.ports.length;
  browserApi.ports.at(-1).disconnect();
  alarmEvent.emit({ name });
  assert.ok(
    browserApi.ports.length > before,
    "alarm tick should re-establish the native connection",
  );
});
