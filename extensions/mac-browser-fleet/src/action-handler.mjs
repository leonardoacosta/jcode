export const ACTIONS = Object.freeze({
  ACTIVATE_TAB: "activate_tab",
  NAVIGATE: "navigate",
  RELOAD: "reload",
  GO_BACK: "go_back",
  GO_FORWARD: "go_forward",
  CREATE_TAB: "create_tab",
  CLOSE_TAB: "close_tab",
});

export const ERROR_CODES = Object.freeze({
  INVALID_REQUEST: "invalid_request",
  UNSUPPORTED_ACTION: "unsupported_action",
  BROWSER_API_FAILURE: "browser_api_failure",
});

const SUPPORTED_ACTIONS = new Set(Object.values(ACTIONS));

function failure(requestId, code, message) {
  return { requestId, ok: false, error: { code, message } };
}

function success(requestId, result) {
  return { requestId, ok: true, result };
}

function isId(value) {
  return Number.isSafeInteger(value) && value >= 0;
}

function validateRequest(request) {
  if (!request || typeof request !== "object" || Array.isArray(request)) {
    return "request must be an object";
  }
  if (typeof request.requestId !== "string" || request.requestId.length === 0) {
    return "requestId is required";
  }
  if (typeof request.action !== "string" || request.action.length === 0) {
    return "action is required";
  }
  if (!SUPPORTED_ACTIONS.has(request.action)) {
    return null;
  }

  const target = request.target;
  const payload = request.payload ?? {};
  if (!target || typeof target !== "object" || Array.isArray(target)) {
    return "target is required";
  }
  if (!payload || typeof payload !== "object" || Array.isArray(payload)) {
    return "payload must be an object";
  }

  if (request.action === ACTIONS.CREATE_TAB) {
    if (!isId(target.windowId)) return "windowId is required";
  } else if (!isId(target.tabId)) {
    return "tabId is required";
  }

  if (request.action === ACTIONS.ACTIVATE_TAB && !isId(target.windowId)) {
    return "windowId is required";
  }
  if (
    request.action === ACTIONS.NAVIGATE &&
    (typeof payload.url !== "string" || payload.url.length === 0)
  ) {
    return "navigation URL is required";
  }
  if (
    request.action === ACTIONS.CREATE_TAB &&
    payload.url !== undefined &&
    (typeof payload.url !== "string" || payload.url.length === 0)
  ) {
    return "tab URL must be a non-empty string";
  }
  if (
    request.action === ACTIONS.CREATE_TAB &&
    payload.active !== undefined &&
    typeof payload.active !== "boolean"
  ) {
    return "active must be a boolean";
  }
  if (
    request.action === ACTIONS.RELOAD &&
    payload.bypassCache !== undefined &&
    typeof payload.bypassCache !== "boolean"
  ) {
    return "bypassCache must be a boolean";
  }

  return null;
}

export function createExtensionActionHandler(browserApi) {
  if (!browserApi?.tabs || !browserApi?.windows) {
    throw new TypeError("browserApi tabs and windows APIs are required");
  }

  return async function handleExtensionAction(request) {
    const requestId =
      request && typeof request.requestId === "string"
        ? request.requestId
        : "unknown";
    const validationError = validateRequest(request);
    if (validationError) {
      return failure(requestId, ERROR_CODES.INVALID_REQUEST, validationError);
    }
    if (!SUPPORTED_ACTIONS.has(request.action)) {
      return failure(
        requestId,
        ERROR_CODES.UNSUPPORTED_ACTION,
        "action is not supported by ordinary extension tabs",
      );
    }

    const target = request.target;
    const payload = request.payload ?? {};

    try {
      switch (request.action) {
        case ACTIONS.ACTIVATE_TAB:
          await browserApi.tabs.update(target.tabId, { active: true });
          await browserApi.windows.update(target.windowId, { focused: true });
          return success(requestId, {
            tabId: target.tabId,
            windowId: target.windowId,
          });
        case ACTIONS.NAVIGATE:
          await browserApi.tabs.update(target.tabId, { url: payload.url });
          return success(requestId, { tabId: target.tabId });
        case ACTIONS.RELOAD:
          await browserApi.tabs.reload(target.tabId, {
            bypassCache: payload.bypassCache ?? false,
          });
          return success(requestId, { tabId: target.tabId });
        case ACTIONS.GO_BACK:
          await browserApi.tabs.goBack(target.tabId);
          return success(requestId, { tabId: target.tabId });
        case ACTIONS.GO_FORWARD:
          await browserApi.tabs.goForward(target.tabId);
          return success(requestId, { tabId: target.tabId });
        case ACTIONS.CREATE_TAB: {
          const tab = await browserApi.tabs.create({
            windowId: target.windowId,
            ...(payload.url === undefined ? {} : { url: payload.url }),
            active: payload.active ?? true,
          });
          return success(requestId, { tabId: tab.id, windowId: tab.windowId });
        }
        case ACTIONS.CLOSE_TAB:
          await browserApi.tabs.remove(target.tabId);
          return success(requestId, { tabId: target.tabId });
        default:
          return failure(
            requestId,
            ERROR_CODES.UNSUPPORTED_ACTION,
            "action is not supported by ordinary extension tabs",
          );
      }
    } catch {
      return failure(
        requestId,
        ERROR_CODES.BROWSER_API_FAILURE,
        "browser action failed",
      );
    }
  };
}
