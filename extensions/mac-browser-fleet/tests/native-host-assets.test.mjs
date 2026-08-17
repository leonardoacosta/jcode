import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

import { NATIVE_HOST_NAME } from "../src/background.mjs";

const hostManifestPaths = {
  chrome: new URL("../native-host/chrome/dev.jcode.mac_browser_fleet.json", import.meta.url),
  edge: new URL("../native-host/edge/dev.jcode.mac_browser_fleet.json", import.meta.url),
};

const expectedExtensionIds = {
  chrome: "__JCODE_CHROME_EXTENSION_ID__",
  edge: "__JCODE_EDGE_EXTENSION_ID__",
};

for (const [browser, manifestUrl] of Object.entries(hostManifestPaths)) {
  test(`${browser} native host manifest is explicit and protocol-compatible`, async () => {
    const manifest = JSON.parse(await readFile(manifestUrl, "utf8"));

    assert.equal(manifest.name, NATIVE_HOST_NAME);
    assert.equal(manifest.type, "stdio");
    assert.equal(manifest.path, "/__JCODE_MAC_BROWSER_FLEET_HOST_PATH__");
    assert.deepEqual(manifest.allowed_origins, [
      `chrome-extension://${expectedExtensionIds[browser]}/`,
    ]);
    assert.equal(manifest.allowed_origins.some((origin) => origin.includes("*")), false);
    assert.deepEqual(Object.keys(manifest).sort(), [
      "allowed_origins",
      "description",
      "name",
      "path",
      "type",
    ]);
  });
}

test("Chrome and Edge native host grants use distinct extension identities", async () => {
  const manifests = await Promise.all(
    Object.values(hostManifestPaths).map(async (manifestUrl) =>
      JSON.parse(await readFile(manifestUrl, "utf8")),
    ),
  );

  assert.notEqual(manifests[0].allowed_origins[0], manifests[1].allowed_origins[0]);
});
