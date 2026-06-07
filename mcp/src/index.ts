#!/usr/bin/env node
// APS MCP server (ORCH-006, D-004).
//
// Exposes the `aps` CLI command surface as a single MCP codemode tool over
// stdio. The server shells out to whichever binary provides `aps` — it is
// agnostic to the bash/Rust implementation question (D-006).
//
// Environment:
//   APS_BIN    Path to the aps executable (default: sibling bin/aps, then PATH)
//   APS_PLANS  Plan root directory passed as --plans (default: CLI default)

import { execFile } from "node:child_process";
import { existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod";

import { route } from "./route.ts";

const here = path.dirname(fileURLToPath(import.meta.url));
const siblingBin = path.resolve(here, "../../bin/aps");
const APS_BIN = process.env.APS_BIN ?? (existsSync(siblingBin) ? siblingBin : "aps");
const APS_PLANS = process.env.APS_PLANS;

type CliResult = { exitCode: number; output: string };

function runAps(argv: string[]): Promise<CliResult> {
  const args = [...argv];
  if (APS_PLANS) {
    // lint takes a positional path; orchestration commands take --plans
    if (args[0] === "lint") {
      if (args.length === 1) args.push(APS_PLANS);
    } else {
      args.push("--plans", APS_PLANS);
    }
  }
  return new Promise((resolve) => {
    execFile(
      APS_BIN,
      args,
      { timeout: 30_000, maxBuffer: 4 * 1024 * 1024 },
      (error, stdout, stderr) => {
        const exitCode = error
          ? typeof error.code === "number"
            ? error.code
            : 1
          : 0;
        const output = [stdout, stderr].filter(Boolean).join("\n").trim();
        resolve({
          exitCode,
          output: output || (error ? String(error.message) : ""),
        });
      },
    );
  });
}

const server = new McpServer({ name: "aps-orchestrate", version: "0.1.0" });

server.registerTool(
  "aps",
  {
    title: "APS plan orchestration",
    description:
      "Navigate an APS plan. Send a direct command (next, start, complete, " +
      "graph, lint) or a natural-language request like " +
      '"what\'s the next ready work item in the auth module?".',
    inputSchema: {
      request: z
        .string()
        .describe("Direct aps command or natural-language request"),
    },
  },
  async ({ request }: { request: string }) => {
    const routed = route(request);
    if ("error" in routed) {
      return {
        content: [{ type: "text" as const, text: routed.error }],
        isError: true,
      };
    }
    const result = await runAps(routed.argv);
    const text =
      `$ aps ${routed.argv.join(" ")}\n${result.output}`.trim() ||
      "(no output)";
    return {
      content: [{ type: "text" as const, text }],
      isError: result.exitCode !== 0,
    };
  },
);

const transport = new StdioServerTransport();
await server.connect(transport);
