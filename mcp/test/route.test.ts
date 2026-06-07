import { test } from "node:test";
import assert from "node:assert/strict";

import { route } from "../src/route.ts";

function argvOf(request: string): string[] {
  const result = route(request);
  assert.ok("argv" in result, `expected route, got error: ${(result as { error?: string }).error}`);
  return (result as { argv: string[] }).argv;
}

function errorOf(request: string): string {
  const result = route(request);
  assert.ok("error" in result, `expected error, got argv: ${(result as { argv?: string[] }).argv}`);
  return (result as { error: string }).error;
}

test("direct: next", () => {
  assert.deepEqual(argvOf("next"), ["next"]);
  assert.deepEqual(argvOf("next auth"), ["next", "auth"]);
  assert.deepEqual(argvOf("aps next auth"), ["next", "auth"]);
});

test("direct: start/complete require an ID", () => {
  assert.deepEqual(argvOf("start AUTH-003"), ["start", "AUTH-003"]);
  assert.deepEqual(argvOf("complete AUTH-003"), ["complete", "AUTH-003"]);
  errorOf("start");
  errorOf("complete");
});

test("direct: complete with quoted learning", () => {
  assert.deepEqual(argvOf('complete AUTH-003 --learning "retry on 5xx"'), [
    "complete",
    "AUTH-003",
    "--learning",
    "retry on 5xx",
  ]);
});

test("direct: unknown options rejected", () => {
  errorOf("next --exec rm");
  errorOf("start AUTH-003 --learning oops");
});

test("direct: unsafe arguments rejected", () => {
  errorOf("next auth; rm -rf /");
  errorOf("next $(whoami)");
});

test("natural language: next ready item", () => {
  assert.deepEqual(
    argvOf("What's the next ready work item in the auth module?"),
    ["next", "auth"],
  );
  assert.deepEqual(argvOf("what is ready to work on"), ["next"]);
});

test("natural language: start and complete", () => {
  assert.deepEqual(argvOf("please start AUTH-003"), ["start", "AUTH-003"]);
  assert.deepEqual(argvOf("mark AUTH-003 as done"), ["complete", "AUTH-003"]);
  assert.deepEqual(
    argvOf('complete AUTH-003 with learning: "retry on 5xx"'),
    ["complete", "AUTH-003", "--learning", "retry on 5xx"],
  );
});

test("natural language: graph", () => {
  assert.deepEqual(argvOf("show the dependency graph for auth"), ["graph", "auth"]);
  assert.deepEqual(argvOf("visualize dependencies"), ["graph"]);
});

test("malformed input yields help", () => {
  const error = errorOf("frobnicate the widgets");
  assert.match(error, /Could not route request/);
  errorOf("");
  errorOf("   ");
});
