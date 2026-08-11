import assert from "node:assert/strict";
import test from "node:test";

import {
  ACTION_MESSAGE_TYPES,
  installActionMessageHandler,
} from "../src/background.mjs";
import { ACTIONS, ERROR_CODES } from "../src/action-handler.mjs";

function fakePort() {
  let listener;
  const posted = [];
  return {
    posted,
    onMessage: {
      addListener(next) {
        listener = next;
      },
    },
    receive(message) {
      return listener(message);
    },
    postMessage(message) {
      posted.push(message);
    },
  };
}

function fakeBrowserApi() {
  const calls = [];
  return {
    calls,
    tabs: {
      async update(tabId, properties) {
        calls.push(["tabs.update", tabId, properties]);
      },
    },
    windows: {},
  };
}

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

  assert.deepEqual(browserApi.calls, [
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
