import { ai, type AiConfig, type AiMessage } from "./ai";
import { api } from "./ipc";
import { isDocument } from "./files";
import { openTab } from "./state";
import { persistSettings } from "./actions";
import type { Settings } from "./types";

export interface EditProposal {
  path: string;
  summary: string;
  before: string;
  after: string;
}

export interface AgentStep {
  kind: "thought" | "tool" | "proposal" | "final";
  text: string;
}

export type AgentEvent = AgentStep;

const SYSTEM_PROMPT = `You are the assistant inside NoteCraft, a local-first Markdown notes app.
You are talking to the owner of the vault and you can inspect and change it with tools.

Reply with exactly one JSON object and nothing else. Two shapes are allowed:

1) Use a tool:
{"thought": "short reasoning", "tool": "<tool name>", "input": { ... }}

2) Finish:
{"thought": "short reasoning", "final": "answer in Markdown"}

Tools:
- list_notes {} — list note paths in the vault (Markdown, MDX and JSON).
- read_note {path} — read a note.
- search {query} — full-text search, returns path and snippet.
- create_note {path, content} — create a NEW note. Never overwrite.
- propose_edit {path, content, summary} — propose replacing a note's full text. The user must approve it before it is written, so always use this for edits.
- open_note {path} — open a note in the editor.
- rename_note {from, to} — rename or move a note inside the vault.
- create_daily_note {date?} — create (or open) the daily note for YYYY-MM-DD.
- list_tasks {} — list open tasks across the vault.
- set_theme {theme} — switch the app theme ("dark" or "light").

Rules:
- Paths are vault-relative and use forward slashes, e.g. "notes/idea.md".
- Use propose_edit for every change to an existing note; it is shown to the user with a diff and Accept/Reject buttons.
- Keep edits minimal and preserve the user's voice and formatting.
- Do not invent note contents; read first when you are unsure.
- Answer in the user's language. Markdown is fine in "final", plain text in other fields.`;

interface RawAction {
  thought?: string;
  tool?: string;
  input?: Record<string, unknown>;
  final?: string;
}

function parseAction(reply: string): RawAction | null {
  let text = reply.trim();
  const fence = text.match(/```(?:json)?\s*([\s\S]*?)```/i);
  if (fence) text = fence[1].trim();
  if (!text.startsWith("{")) {
    const start = text.indexOf("{");
    const end = text.lastIndexOf("}");
    if (start === -1 || end === -1 || end <= start) return null;
    text = text.slice(start, end + 1);
  }
  try {
    const parsed: unknown = JSON.parse(text);
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) return null;
    return parsed as RawAction;
  } catch {
    return null;
  }
}

function stringInput(input: Record<string, unknown> | undefined, key: string): string {
  const value = input?.[key];
  return typeof value === "string" ? value : "";
}

async function runTool(
  name: string,
  input: Record<string, unknown> | undefined,
  emit: (event: AgentEvent) => void
): Promise<string> {
  try {
    switch (name) {
      case "list_notes": {
        const files = await api.listAllFiles();
        const paths = files
          .filter((file) => !file.is_dir && isDocument(file.path))
          .map((file) => file.path)
          .slice(0, 400);
        emit({ kind: "tool", text: `list_notes → ${paths.length} notes` });
        return paths.join("\n") || "(the vault has no notes yet)";
      }
      case "read_note": {
        const path = stringInput(input, "path");
        if (!path) return "error: path is required";
        const content = await api.readFile(path);
        emit({ kind: "tool", text: `read_note → ${path}` });
        return content.slice(0, 20000);
      }
      case "search": {
        const query = stringInput(input, "query");
        if (!query) return "error: query is required";
        const results = await api.search(query);
        emit({ kind: "tool", text: `search → ${results.length} results` });
        return (
          results
            .slice(0, 12)
            .map((result) => `${result.path}: ${result.snippet.replace(/\s+/g, " ").slice(0, 200)}`)
            .join("\n") || "(no matches)"
        );
      }
      case "create_note": {
        const path = stringInput(input, "path");
        const content = stringInput(input, "content");
        if (!path) return "error: path is required";
        await api.createFile(path, content);
        await openTab(path);
        emit({ kind: "tool", text: `create_note → ${path}` });
        return "created";
      }
      case "open_note": {
        const path = stringInput(input, "path");
        if (!path) return "error: path is required";
        await openTab(path);
        emit({ kind: "tool", text: `open_note → ${path}` });
        return "opened";
      }
      case "rename_note": {
        const from = stringInput(input, "from");
        const to = stringInput(input, "to");
        if (!from || !to) return "error: from and to are required";
        await api.renamePath(from, to);
        emit({ kind: "tool", text: `rename_note → ${to}` });
        return `renamed to ${to}`;
      }
      case "create_daily_note": {
        const date = stringInput(input, "date");
        const path = await api.createDailyNote(date || undefined);
        await openTab(path);
        emit({ kind: "tool", text: `create_daily_note → ${path}` });
        return path;
      }
      case "list_tasks": {
        const tasks = await api.listTasks();
        const open = tasks.filter((task) => !task.done).slice(0, 60);
        emit({ kind: "tool", text: `list_tasks → ${open.length} open` });
        return (
          open.map((task) => `${task.note_path}:${task.line} — ${task.text}`).join("\n") ||
          "(no open tasks)"
        );
      }
      case "set_theme": {
        const theme: Settings["theme"] = stringInput(input, "theme") === "light" ? "light" : "dark";
        await persistSettings({ theme });
        emit({ kind: "tool", text: `set_theme → ${theme}` });
        return `theme is now ${theme}`;
      }
      case "propose_edit": {
        const path = stringInput(input, "path");
        const after = stringInput(input, "content");
        const summary = stringInput(input, "summary") || "Proposed edit";
        if (!path) return "error: path is required";
        const before = await api.readFile(path).catch(() => "");
        emit({ kind: "proposal", text: JSON.stringify({ path, summary, before, after }) });
        return "The edit was shown to the user for approval. Do not repeat it.";
      }
      default:
        return `error: unknown tool "${name}"`;
    }
  } catch (error) {
    return `error: ${String(error)}`;
  }
}

export interface AgentResult {
  final: string;
  messages: AiMessage[];
}

/**
 * Run one user turn: send the conversation, execute tool calls, and stop when
 * the model answers. Edits are emitted as proposals for the user to approve, so
 * nothing is written to the vault without an explicit click.
 */
export async function runAgentTurn({
  config,
  history,
  message,
  onEvent,
}: {
  config: AiConfig;
  history: AiMessage[];
  message: string;
  onEvent: (event: AgentEvent) => void;
}): Promise<AgentResult> {
  const messages: AiMessage[] = [...history, { role: "user", content: message }];
  let final = "";

  for (let step = 0; step < 8; step += 1) {
    const reply = await ai.chat(config, SYSTEM_PROMPT, messages);
    const action = parseAction(reply);

    if (!action) {
      final = reply.trim();
      onEvent({ kind: "final", text: final });
      messages.push({ role: "assistant", content: final });
      return { final, messages };
    }
    if (action.thought) onEvent({ kind: "thought", text: action.thought });

    if (action.final) {
      final = action.final.trim();
      onEvent({ kind: "final", text: final });
      messages.push({ role: "assistant", content: final });
      return { final, messages };
    }
    if (!action.tool) {
      final = reply.trim();
      onEvent({ kind: "final", text: final });
      messages.push({ role: "assistant", content: final });
      return { final, messages };
    }

    messages.push({ role: "assistant", content: reply });
    const result = await runTool(action.tool, action.input, onEvent);
    messages.push({
      role: "user",
      content: `Tool "${action.tool}" returned:\n${result}`,
    });
  }

  final =
    "I stopped after several steps to avoid looping. Ask me to continue, or make the request more specific.";
  onEvent({ kind: "final", text: final });
  messages.push({ role: "assistant", content: final });
  return { final, messages };
}

export type DiffLine = { kind: "same" | "add" | "remove"; text: string };

/** Small line diff for the approval card. Falls back to a replace block. */
export function diffLines(before: string, after: string): DiffLine[] {
  const a = before.split("\n");
  const b = after.split("\n");
  if (a.length > 500 || b.length > 500) {
    return [
      ...a.map((text) => ({ kind: "remove" as const, text })),
      ...b.map((text) => ({ kind: "add" as const, text })),
    ];
  }
  const table: number[][] = Array.from({ length: a.length + 1 }, () =>
    new Array<number>(b.length + 1).fill(0)
  );
  for (let i = a.length - 1; i >= 0; i -= 1) {
    for (let j = b.length - 1; j >= 0; j -= 1) {
      table[i][j] = a[i] === b[j] ? table[i + 1][j + 1] + 1 : Math.max(table[i + 1][j], table[i][j + 1]);
    }
  }
  const out: DiffLine[] = [];
  let i = 0;
  let j = 0;
  while (i < a.length && j < b.length) {
    if (a[i] === b[j]) {
      out.push({ kind: "same", text: a[i] });
      i += 1;
      j += 1;
    } else if (table[i + 1][j] >= table[i][j + 1]) {
      out.push({ kind: "remove", text: a[i] });
      i += 1;
    } else {
      out.push({ kind: "add", text: b[j] });
      j += 1;
    }
  }
  while (i < a.length) {
    out.push({ kind: "remove", text: a[i] });
    i += 1;
  }
  while (j < b.length) {
    out.push({ kind: "add", text: b[j] });
    j += 1;
  }
  return out;
}
