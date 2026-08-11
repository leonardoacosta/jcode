import { createExtensionActionHandler } from "./action-handler.mjs";

export const ACTION_MESSAGE_TYPES = Object.freeze({
  REQUEST: "action_request",
  RESPONSE: "action_response",
});

export const NATIVE_HOST_NAME = "dev.jcode.mac_browser_fleet";

export function installActionMessageHandler(port, browserApi) {
  if (!port?.onMessage?.addListener || typeof port.postMessage !== "function") {
    throw new TypeError("native messaging port is required");
  }

  const handleAction = createExtensionActionHandler(browserApi);
  port.onMessage.addListener(async (message) => {
    if (message?.type !== ACTION_MESSAGE_TYPES.REQUEST) return;

    const response = await handleAction(message);
    port.postMessage({ type: ACTION_MESSAGE_TYPES.RESPONSE, ...response });
  });
}

export function connectActionNativeHost(browserApi, hostName = NATIVE_HOST_NAME) {
  if (typeof browserApi?.runtime?.connectNative !== "function") {
    throw new TypeError("native messaging runtime API is required");
  }

  const port = browserApi.runtime.connectNative(hostName);
  installActionMessageHandler(port, browserApi);
  return port;
}

if (globalThis.chrome?.runtime?.connectNative) {
  connectActionNativeHost(globalThis.chrome);
}
