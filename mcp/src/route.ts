// Request routing for the APS MCP server (ORCH-006).
//
// Maps a single free-form `request` string — either a direct CLI command
// ("next auth") or a natural-language request ("what's the next ready item
// in the auth module?") — to a safe `aps` argument vector. Commands are
// allowlisted and arguments validated so the server never executes
// arbitrary input.

export type Route = { argv: string[] } | { error: string };

const COMMANDS = new Set(["next", "start", "complete", "graph", "lint"]);

// Work item IDs: 2-6 uppercase chars, dash, digits (AUTH-001, ORCH-006)
const ID_RE = /\b([A-Z][A-Z0-9]{1,5}-\d{1,4})\b/;

// Safe positional argument: module IDs, file names, relative paths
const SAFE_ARG_RE = /^[A-Za-z0-9._/-]+$/;

// Shell metacharacters — always rejected outright, never reinterpreted
const DANGEROUS_RE = /[;&|$`<>\\!*?(){}[\]~#]/;

// Words that look like a module reference but aren't one
const MODULE_STOPWORDS = new Set([
  "the",
  "this",
  "that",
  "plan",
  "plans",
  "project",
  "repo",
  "repository",
  "module",
  "item",
  "items",
  "work",
]);

export const HELP = `Could not route request. Send a direct command or a natural-language request.

Commands:
  next [module]                     Show the next Ready work item
  start <ID>                        Mark a Ready work item as In Progress
  complete <ID> [--learning "..."]  Mark an In Progress work item as Complete
  graph [module]                    Show work item dependency graph
  lint [file|dir]                   Validate APS documents

Examples:
  "next auth"
  "what's the next ready item in the auth module?"
  "start AUTH-003"
  "complete AUTH-003 with learning: retry on 5xx"
  "show the dependency graph for auth"`;

function fail(detail: string): Route {
  return { error: `${detail}\n\n${HELP}` };
}

// Tokenize a direct command, honouring double/single-quoted segments so
// `complete X --learning "multi word insight"` survives intact.
function tokenize(text: string): string[] | null {
  const tokens: string[] = [];
  const re = /"([^"]*)"|'([^']*)'|(\S+)/g;
  let match: RegExpExecArray | null;
  while ((match = re.exec(text)) !== null) {
    tokens.push(match[1] ?? match[2] ?? match[3]);
  }
  return tokens.length > 0 ? tokens : null;
}

// Parse a direct command. Returns null when the input starts with a command
// word but carries more positionals than the command accepts — that signals
// natural language ("next ready item in the auth module"), not a command.
function routeDirect(tokens: string[]): Route | null {
  const [cmd, ...rest] = tokens;
  const argv = [cmd];
  let positionals = 0;
  for (let i = 0; i < rest.length; i++) {
    const arg = rest[i];
    if (arg === "--learning") {
      const value = rest[++i];
      if (value === undefined) return fail("--learning requires a value.");
      if (cmd !== "complete") return fail(`--learning is only valid for complete.`);
      argv.push("--learning", value);
    } else if (arg.startsWith("-")) {
      return fail(`Unsupported option: ${arg}`);
    } else if (DANGEROUS_RE.test(arg)) {
      return fail(`Unsafe argument: ${arg}`);
    } else if (SAFE_ARG_RE.test(arg)) {
      argv.push(arg);
      positionals++;
    } else {
      return null; // punctuation etc. — treat as natural language
    }
  }
  if (positionals > 1) return null; // looks like natural language
  if ((cmd === "start" || cmd === "complete") && !argv.some((a) => ID_RE.test(a))) {
    return fail(`${cmd} requires a work item ID (e.g. AUTH-003).`);
  }
  return { argv };
}

// Extract a module reference from natural language: "in the auth module",
// "for auth", or a bare lowercase module token after the verb.
function extractModule(text: string): string | null {
  const match = text.match(/\b(?:in|for|of)\s+(?:the\s+)?([A-Za-z][A-Za-z0-9_-]*)(?:\s+module)?/i);
  if (match && !MODULE_STOPWORDS.has(match[1].toLowerCase())) return match[1];
  return null;
}

export function route(request: string): Route {
  const text = request.trim().replace(/^aps\s+/i, "");
  if (!text) return fail("Empty request.");

  const tokens = tokenize(text);
  if (!tokens) return fail("Empty request.");

  // Direct command path; falls through to natural language when the input
  // merely starts with a command word
  if (COMMANDS.has(tokens[0].toLowerCase())) {
    const direct = routeDirect([tokens[0].toLowerCase(), ...tokens.slice(1)]);
    if (direct !== null) return direct;
  }

  // Natural-language path
  const lower = text.toLowerCase();
  const id = text.match(ID_RE)?.[1];

  const completeVerb = /\b(complete|finish|finished|done|close)\b/.test(lower);
  const startVerb = /\b(start|begin|work on|pick up)\b/.test(lower);

  if (id && completeVerb) {
    const learning = text.match(/learning[:\s]+["']?(.+?)["']?$/i)?.[1];
    return { argv: learning ? ["complete", id, "--learning", learning] : ["complete", id] };
  }
  if (id && startVerb) {
    return { argv: ["start", id] };
  }
  if (/\b(graph|depend\w*|visuali[sz]e)\b/.test(lower)) {
    const module = extractModule(text);
    return { argv: module ? ["graph", module] : ["graph"] };
  }
  if (/\b(next|ready|what should)\b/.test(lower)) {
    const module = extractModule(text);
    return { argv: module ? ["next", module] : ["next"] };
  }
  if (/\b(lint|valid\w*)\b/.test(lower)) {
    return { argv: ["lint"] };
  }
  if (completeVerb) return fail("Completing an item requires a work item ID.");
  if (startVerb) return fail("Starting an item requires a work item ID.");

  return fail(`Unrecognised request: ${request.trim()}`);
}
