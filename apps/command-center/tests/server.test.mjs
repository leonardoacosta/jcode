import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { mkdtemp, mkdir, symlink, writeFile } from "node:fs/promises";
import http from "node:http";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { createCommandCenterServer } from "../server.mjs";

async function listen(server) {
  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  return `http://127.0.0.1:${server.address().port}`;
}

async function fixture() {
  const root = await mkdtemp(path.join(os.tmpdir(), "jcode-command-center-"));
  await mkdir(path.join(root, "assets"));
  await writeFile(path.join(root, "index.html"), "<main>command center</main>");
  await writeFile(path.join(root, "assets", "app.js"), "console.log('ok')");
  return root;
}

test("serves health, static assets, and SPA fallback", async (t) => {
  const publicDir = await fixture();
  const server = createCommandCenterServer({ publicDir, apiUrl: "http://127.0.0.1:9" });
  t.after(() => server.close());
  const base = await listen(server);

  assert.deepEqual(await (await fetch(`${base}/healthz`)).json(), { status: "ok" });
  assert.equal(await (await fetch(`${base}/assets/app.js`)).text(), "console.log('ok')");
  assert.equal(
    await (await fetch(`${base}/initiatives/demo`)).text(),
    "<main>command center</main>",
  );
  assert.equal((await fetch(`${base}/../package.json`)).status, 404);
});

test("proxies API method, body, headers, and upstream status", async (t) => {
  let observed;
  const upstream = http.createServer(async (request, response) => {
    const chunks = [];
    for await (const chunk of request) chunks.push(chunk);
    observed = {
      method: request.method,
      url: request.url,
      body: Buffer.concat(chunks).toString(),
      authorization: request.headers.authorization,
      origin: request.headers.origin,
    };
    response.writeHead(201, { "content-type": "application/json", "x-upstream": "yes" });
    response.end(JSON.stringify({ proxied: true }));
  });
  t.after(() => upstream.close());
  const upstreamBase = await listen(upstream);

  const server = createCommandCenterServer({ publicDir: await fixture(), apiUrl: upstreamBase });
  t.after(() => server.close());
  const base = await listen(server);
  const response = await fetch(`${base}/api/command-center/bootstrap?x=1`, {
    method: "POST",
    headers: { authorization: "Bearer test", origin: base, "content-type": "application/json" },
    body: JSON.stringify({ hello: "world" }),
  });

  assert.equal(response.status, 201);
  assert.equal(response.headers.get("x-upstream"), "yes");
  assert.deepEqual(await response.json(), { proxied: true });
  assert.deepEqual(observed, {
    method: "POST",
    url: "/api/command-center/bootstrap?x=1",
    body: '{"hello":"world"}',
    authorization: "Bearer test",
    origin: undefined,
  });
});

test("CLI entrypoint listens until terminated", async (t) => {
  const publicDir = await fixture();
  const probe = http.createServer();
  const probeBase = await listen(probe);
  const port = new URL(probeBase).port;
  await new Promise((resolve) => probe.close(resolve));

  const entrypoint = path.join(
    await mkdtemp(path.join(os.tmpdir(), "jcode-command-center-link-")),
    "server.mjs",
  );
  await symlink(path.resolve(import.meta.dirname, "..", "server.mjs"), entrypoint);
  const child = spawn(process.execPath, [entrypoint], {
    cwd: path.resolve(import.meta.dirname, ".."),
    env: {
      ...process.env,
      JCODE_COMMAND_CENTER_PUBLIC_DIR: publicDir,
      JCODE_COMMAND_CENTER_UI_BIND: `127.0.0.1:${port}`,
      JCODE_COMMAND_CENTER_API_URL: "http://127.0.0.1:9",
    },
    stdio: "pipe",
  });
  t.after(() => child.kill("SIGTERM"));

  let response;
  for (let attempt = 0; attempt < 30; attempt += 1) {
    try {
      response = await fetch(`http://127.0.0.1:${port}/healthz`);
      break;
    } catch {
      await new Promise((resolve) => setTimeout(resolve, 25));
    }
  }
  assert.equal(child.exitCode, null);
  assert.equal(response?.status, 200);
});

test("returns 502 when Jcode is unavailable while health remains available", async (t) => {
  const server = createCommandCenterServer({
    publicDir: await fixture(),
    apiUrl: "http://127.0.0.1:9",
  });
  t.after(() => server.close());
  const base = await listen(server);

  const response = await fetch(`${base}/api/command-center/bootstrap`, { method: "POST" });
  assert.equal(response.status, 502);
  assert.match(await response.text(), /Jcode Command Center API is unavailable/);
  assert.equal((await fetch(`${base}/healthz`)).status, 200);
});
