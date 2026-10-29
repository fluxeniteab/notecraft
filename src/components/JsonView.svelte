<script lang="ts">
  import { AlertTriangle, Check, Minimize2, Wand2 } from "lucide-svelte";
  import type { JsonTreeNode } from "../lib/types";
  import type { Tab } from "../lib/state";
  import { pushToast, updateContent } from "../lib/state";
  import { api } from "../lib/ipc";

  export let tab: Tab;

  interface Row {
    node: JsonTreeNode;
    depth: number;
  }

  let root: JsonTreeNode | null = null;
  let error: { message: string; line: number; column: number } | null = null;
  let meta = { keys: 0, nodes: 0, depth: 0, truncated: false };
  let collapsed = new Set<string>();
  let rows: Row[] = [];
  let parsedKey = "";
  let lastRoot: JsonTreeNode | null = null;
  let timer = 0;
  let seq = 0;
  let busy = true;

  $: scheduleParse(tab.content);
  $: if (root && root !== lastRoot) {
    lastRoot = root;
    collapsed = defaultCollapsed(root);
  }
  $: rows = root ? flatten(root, collapsed) : [];

  function scheduleParse(content: string): void {
    if (content === parsedKey) return;
    parsedKey = content;
    window.clearTimeout(timer);
    timer = window.setTimeout(() => void parse(content), 200);
  }

  async function parse(content: string): Promise<void> {
    const mine = ++seq;
    busy = true;
    try {
      const analysis = await api.jsonAnalyze(content);
      if (mine !== seq) return;
      root = analysis.root;
      error = analysis.error
        ? { message: analysis.error.message, line: analysis.error.line, column: analysis.error.column }
        : null;
      meta = {
        keys: analysis.keys,
        nodes: analysis.nodes,
        depth: analysis.depth,
        truncated: analysis.truncated,
      };
    } catch (problem) {
      if (mine !== seq) return;
      error = { message: String(problem), line: 0, column: 0 };
      root = null;
    } finally {
      if (mine === seq) busy = false;
    }
  }

  function isContainer(node: JsonTreeNode): boolean {
    return node.kind === "array" || node.kind === "object";
  }

  function defaultCollapsed(node: JsonTreeNode): Set<string> {
    const set = new Set<string>();
    const walk = (item: JsonTreeNode, depth: number): void => {
      if (isContainer(item) && depth >= 2) set.add(item.pointer);
      if (!set.has(item.pointer)) for (const child of item.children) walk(child, depth + 1);
    };
    walk(node, 0);
    return set;
  }

  function flatten(node: JsonTreeNode, closed: Set<string>, depth = 0, out: Row[] = []): Row[] {
    out.push({ node, depth });
    if (isContainer(node) && !closed.has(node.pointer)) {
      for (const child of node.children) flatten(child, closed, depth + 1, out);
    }
    return out;
  }

  function toggle(pointer: string): void {
    const next = new Set(collapsed);
    if (next.has(pointer)) next.delete(pointer);
    else next.add(pointer);
    collapsed = next;
  }

  async function format(): Promise<void> {
    try {
      const next = await api.jsonFormat(tab.content, 2);
      if (next !== tab.content) updateContent(tab.id, next);
      pushToast("success", "JSON formatted");
    } catch (problem) {
      pushToast("error", "Cannot format invalid JSON", String(problem));
    }
  }

  async function minify(): Promise<void> {
    try {
      const next = await api.jsonMinify(tab.content);
      if (next !== tab.content) updateContent(tab.id, next);
      pushToast("success", "JSON minified");
    } catch (problem) {
      pushToast("error", "Cannot minify invalid JSON", String(problem));
    }
  }

  function summary(node: JsonTreeNode): string {
    if (node.kind === "array") return `[ ${node.children.length}${node.truncated ? "+" : ""} items ]`;
    if (node.kind === "object") return `{ ${node.children.length}${node.truncated ? "+" : ""} keys }`;
    return node.preview;
  }
</script>

<div class="json-toolbar">
  {#if error}
    <span class="chip danger"><AlertTriangle size={10} /> Invalid JSON</span>
  {:else}
    <span class="chip"><Check size={10} style="color: var(--nc-success)" /> Valid JSON</span>
  {/if}
  <span class="chip">{meta.keys.toLocaleString()} keys</span>
  <span class="chip">{meta.nodes.toLocaleString()} nodes</span>
  <span class="chip">depth {meta.depth}</span>
  {#if meta.truncated}<span class="chip warn">truncated</span>{/if}
  <div style="flex:1"></div>
  <button type="button" class="btn sm" on:click={() => void format()} title="Format with 2-space indentation">
    <Wand2 size={12} /> Format
  </button>
  <button type="button" class="btn sm" on:click={() => void minify()} title="Remove all insignificant whitespace">
    <Minimize2 size={12} /> Minify
  </button>
</div>

{#if error}
  <div class="json-invalid">
    <strong>Cannot parse this JSON</strong>
    <div style="margin-top:5px">{error.message}</div>
    {#if error.line}
      <div style="margin-top:5px">Line {error.line}, column {error.column}</div>
    {/if}
    <div style="margin-top:8px; color: var(--nc-fg-muted)">
      Your source is unchanged. Switch to Source and fix the syntax — the editor keeps every character.
    </div>
  </div>
{/if}

<div class="json-view">
  {#if busy && !root}
    <div class="editor-loading">Parsing…</div>
  {:else if rows.length === 0}
    <div class="panel-empty">This file is empty. Start with <code>{'{ }'}</code> in the source editor.</div>
  {:else}
    <div class="json-node">
      {#each rows as row (row.node.pointer)}
        <div class="json-row" style="padding-left:{6 + row.depth * 16}px">
          {#if isContainer(row.node)}
            <button
              type="button"
              class="caret"
              aria-label={collapsed.has(row.node.pointer) ? "Expand" : "Collapse"}
              on:click={() => toggle(row.node.pointer)}
            >{collapsed.has(row.node.pointer) ? "▸" : "▾"}</button>
          {:else}
            <span class="caret"></span>
          {/if}

          {#if row.node.key && row.node.key !== "$"}
            <span class="key">"{row.node.key}"</span><span class="punctuation">:</span>
          {/if}

          {#if row.node.kind === "string"}
            <span class="value string">"{row.node.preview}"</span>
          {:else if isContainer(row.node)}
            <span class="punctuation">{row.node.kind === "array" ? "[" : "{"}</span>
            <span class="count">{summary(row.node)}</span>
          {:else}
            <span class="value {row.node.kind}">{row.node.preview}</span>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>
