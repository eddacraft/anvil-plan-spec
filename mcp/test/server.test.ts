// End-to-end test for the APS MCP server (ORCH-006 validation criteria):
// tool discovery succeeds, `aps next` works through MCP, and malformed
// input is handled gracefully.

import { after, before, test } from "node:test";
import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";

const here = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(here, "../..");
const fixturePlans = path.join(repoRoot, "test/fixtures/orchestrate/plans");

const client = new Client({ name: "aps-mcp-test", version: "0.0.0" });

before(async () => {
  const transport = new StdioClientTransport({
    command: process.execPath,
    args: [path.join(here, "../src/index.ts")],
    env: { ...process.env, APS_PLANS: fixturePlans },
  });
  await client.connect(transport);
});

after(async () => {
  await client.close();
});

type ToolResult = { content: Array<{ type: string; text?: string }>; isError?: boolean };

function textOf(result: ToolResult): string {
  return result.content.map((c) => c.text ?? "").join("\n");
}

test("tool discovery succeeds", async () => {
  const { tools } = await client.listTools();
  const aps = tools.find((t) => t.name === "aps");
  assert.ok(aps, "aps tool not listed");
  assert.ok(aps.inputSchema.properties?.request, "request property missing");
});

test("aps next works through MCP", async () => {
  const result = (await client.callTool({
    name: "aps",
    arguments: { request: "next ready item in the auth module" },
  })) as ToolResult;
  assert.ok(!result.isError, `unexpected error: ${textOf(result)}`);
  assert.match(textOf(result), /AUTH-003/);
});

test("direct command works through MCP", async () => {
  const result = (await client.callTool({
    name: "aps",
    arguments: { request: "graph AUTH" },
  })) as ToolResult;
  assert.ok(!result.isError, `unexpected error: ${textOf(result)}`);
  assert.match(textOf(result), /AUTH-001/);
});

test("malformed input handled gracefully", async () => {
  const result = (await client.callTool({
    name: "aps",
    arguments: { request: "frobnicate the widgets" },
  })) as ToolResult;
  assert.ok(result.isError, "expected isError for unroutable request");
  assert.match(textOf(result), /Could not route request/);
});

test("CLI failure surfaces as tool error, not crash", async () => {
  const result = (await client.callTool({
    name: "aps",
    arguments: { request: "start AUTH-001" }, // already Complete → invalid transition
  })) as ToolResult;
  assert.ok(result.isError, "expected isError for invalid transition");
  // Server must still answer subsequent requests
  const followUp = (await client.callTool({
    name: "aps",
    arguments: { request: "next" },
  })) as ToolResult;
  assert.ok(!followUp.isError, "server unresponsive after CLI failure");
});
