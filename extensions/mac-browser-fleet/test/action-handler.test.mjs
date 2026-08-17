import assert from "node:assert/strict";
import test from "node:test";

import {
  ACTIONS,
  ERROR_CODES,
  createExtensionActionHandler,
} from "../src/action-handler.mjs";

function fakeBrowserApi() {
  const calls = [];
  return {
    calls,
    tabs: {
      async create(properties) {
        calls.push(["tabs.create", properties]);
        return { id: 91, windowId: 4, active: true, url: properties.url };
      },
      async goBack(tabId) {
        calls.push(["tabs.goBack", tabId]);
      },
      async goForward(tabId) {
        calls.push(["tabs.goForward", tabId]);
      },
      async reload(tabId, properties) {
        calls.push(["tabs.reload", tabId, properties]);
      },
      async remove(tabId) {
        calls.push(["tabs.remove", tabId]);
      },
      async update(tabId, properties) {
        calls.push(["tabs.update", tabId, properties]);
        return { id: tabId, windowId: 4, active: properties.active ?? false, url: properties.url };
      },
    },
    windows: {
      async update(windowId, properties) {
        calls.push(["windows.update", windowId, properties]);
        return { id: windowId, focused: properties.focused };
      },
    },
  };
}

function request(action, overrides = {}) {
  return {
    requestId: "req-1",
    action,
    target: { tabId: 17, windowId: 4 },
    payload: {},
    ...overrides,
  };
}

test("activates the requested ordinary tab and focuses its window", async () => {
  const browserApi = fakeBrowserApi();
  const handle = createExtensionActionHandler(browserApi);

  const response = await handle(request(ACTIONS.ACTIVATE_TAB));

  assert.deepEqual(browserApi.calls, [
    ["tabs.update", 17, { active: true }],
    ["windows.update", 4, { focused: true }],
  ]);
  assert.deepEqual(response, {
    requestId: "req-1",
    ok: true,
    result: { tabId: 17, windowId: 4 },
  });
});

test("executes navigation and history actions through the tabs API", async (t) => {
  const cases = [
    {
      name: "navigate",
      action: ACTIONS.NAVIGATE,
      payload: { url: "https://example.test/path" },
      call: ["tabs.update", 17, { url: "https://example.test/path" }],
    },
    {
      name: "reload",
      action: ACTIONS.RELOAD,
      payload: { bypassCache: true },
      call: ["tabs.reload", 17, { bypassCache: true }],
    },
    { name: "go back", action: ACTIONS.GO_BACK, call: ["tabs.goBack", 17] },
    { name: "go forward", action: ACTIONS.GO_FORWARD, call: ["tabs.goForward", 17] },
  ];

  for (const item of cases) {
    await t.test(item.name, async () => {
      const browserApi = fakeBrowserApi();
      const handle = createExtensionActionHandler(browserApi);
      const response = await handle(
        request(item.action, { payload: item.payload ?? {} }),
      );

      assert.deepEqual(browserApi.calls, [item.call]);
      assert.equal(response.ok, true);
    });
  }
});

test("creates and closes ordinary tabs", async (t) => {
  await t.test("create", async () => {
    const browserApi = fakeBrowserApi();
    const handle = createExtensionActionHandler(browserApi);
    const response = await handle(
      request(ACTIONS.CREATE_TAB, {
        target: { windowId: 4 },
        payload: { url: "https://example.test", active: false },
      }),
    );

    assert.deepEqual(browserApi.calls, [
      ["tabs.create", { windowId: 4, url: "https://example.test", active: false }],
    ]);
    assert.deepEqual(response.result, { tabId: 91, windowId: 4 });
  });

  await t.test("close", async () => {
    const browserApi = fakeBrowserApi();
    const handle = createExtensionActionHandler(browserApi);
    const response = await handle(request(ACTIONS.CLOSE_TAB));

    assert.deepEqual(browserApi.calls, [["tabs.remove", 17]]);
    assert.deepEqual(response.result, { tabId: 17 });
  });
});

test("rejects unsupported actions without calling browser APIs", async () => {
  const browserApi = fakeBrowserApi();
  const handle = createExtensionActionHandler(browserApi);

  const response = await handle(request("evaluate"));

  assert.deepEqual(browserApi.calls, []);
  assert.deepEqual(response, {
    requestId: "req-1",
    ok: false,
    error: {
      code: ERROR_CODES.UNSUPPORTED_ACTION,
      message: "action is not supported by ordinary extension tabs",
    },
  });
});

test("fails closed on malformed requests and browser API failures", async (t) => {
  await t.test("missing target", async () => {
    const handle = createExtensionActionHandler(fakeBrowserApi());
    const response = await handle(request(ACTIONS.NAVIGATE, { target: {} }));

    assert.equal(response.ok, false);
    assert.equal(response.error.code, ERROR_CODES.INVALID_REQUEST);
  });

  await t.test("missing navigation URL", async () => {
    const handle = createExtensionActionHandler(fakeBrowserApi());
    const response = await handle(request(ACTIONS.NAVIGATE));

    assert.equal(response.ok, false);
    assert.equal(response.error.code, ERROR_CODES.INVALID_REQUEST);
  });

  await t.test("browser error is bounded and does not echo request payload", async () => {
    const browserApi = fakeBrowserApi();
    browserApi.tabs.update = async () => {
      throw new Error("secret typed value should not escape");
    };
    const handle = createExtensionActionHandler(browserApi);
    const response = await handle(
      request(ACTIONS.NAVIGATE, {
        payload: { url: "https://user:password@example.test/?token=secret" },
      }),
    );

    assert.deepEqual(response, {
      requestId: "req-1",
      ok: false,
      error: {
        code: ERROR_CODES.BROWSER_API_FAILURE,
        message: "browser action failed",
      },
    });
  });
});
