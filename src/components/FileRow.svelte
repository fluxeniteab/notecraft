<script lang="ts">
  import { Braces, ChevronDown, ChevronRight, Code2, FileText, Folder, FolderOpen } from "lucide-svelte";
  import type { TreeNode } from "../lib/tree";
  import { capabilitiesFor } from "../lib/documentTypes";

  export let node: TreeNode;
  export let depth = 0;
  export let expanded: Set<string>;
  export let selected: string | null = null;
  export let ontoggle: (path: string) => void = () => {};
  export let onopen: (path: string) => void = () => {};
  export let oncontext: (event: MouseEvent, node: TreeNode) => void = () => {};
  export let onmove: (from: string, toDir: string) => void = () => {};

  $: open = expanded.has(node.path);
  $: kind = node.isDir ? null : capabilitiesFor(node.path).kind;
  $: isCode = kind === "javascript" || kind === "typescript" || kind === "css" || kind === "html";

  function activate(): void {
    if (node.isDir) ontoggle(node.path);
    else onopen(node.path);
  }

  function onkey(event: KeyboardEvent): void {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      activate();
    }
  }

  function dragStart(event: DragEvent): void {
    if (node.isDir || !event.dataTransfer) return;
    event.dataTransfer.effectAllowed = "move";
    event.dataTransfer.setData("text/notecraft-path", node.path);
  }

  function dragOver(event: DragEvent): void {
    if (!node.isDir) return;
    event.preventDefault();
    if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
  }

  function drop(event: DragEvent): void {
    if (!node.isDir) return;
    event.preventDefault();
    const from = event.dataTransfer?.getData("text/notecraft-path");
    if (from && from !== node.path) onmove(from, node.path);
  }
</script>

<div
  class="tree-row"
  class:selected={selected === node.path}
  role="button"
  tabindex="0"
  aria-expanded={node.isDir ? open : undefined}
  title={node.path}
  draggable={!node.isDir}
  style="padding-left:{6 + depth * 13}px"
  on:click={activate}
  on:keydown={onkey}
  on:contextmenu|preventDefault={(event) => oncontext(event, node)}
  on:dragstart={dragStart}
  on:dragover={dragOver}
  on:drop={drop}
>
  <span class="twisty">
    {#if node.isDir}
      {#if open}<ChevronDown size={12} />{:else}<ChevronRight size={12} />{/if}
    {/if}
  </span>

  {#if node.isDir}
    {#if open}<FolderOpen size={13} />{:else}<Folder size={13} />{/if}
  {:else if kind === "json"}
    <Braces size={13} />
  {:else if isCode}
    <Code2 size={13} />
  {:else}
    <FileText size={13} />
  {/if}

  <span class="tree-label">{node.name}</span>
  {#if !node.isDir}<span class="ext">{node.extension}</span>{/if}
</div>

