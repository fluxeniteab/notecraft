<script lang="ts">
  import { onMount } from "svelte";
  import { Check, Loader2, Plus, Save, Trash2, X } from "lucide-svelte";
  import { activeTab, allFiles, canvasOpen, openTab, pushToast } from "../lib/state";
  import { api } from "../lib/ipc";
  import { baseName, isDocument } from "../lib/files";

  const CANVAS_PATH = "canvas/main.canvas.json";

  interface CanvasNode {
    id: string;
    x: number;
    y: number;
    label: string;
    path: string;
  }

  interface CanvasEdge {
    id: string;
    source: string;
    target: string;
  }

  let nodes: CanvasNode[] = [];
  let edges: CanvasEdge[] = [];
  let loading = true;
  let saving = false;
  let dirty = false;
  let selected: string | null = null;
  let picking = false;
  let filter = "";

  let zoom = 1;
  let panX = 120;
  let panY = 90;
  let panning = false;
  let panStart = { x: 0, y: 0, panX: 0, panY: 0 };

  let moving: string | null = null;
  let moveStart = { x: 0, y: 0, nodeX: 0, nodeY: 0 };

  let linking: { from: string; x: number; y: number; wx: number; wy: number } | null = null;

  const CARD_W = 168;
  const CARD_H = 52;

  $: documents = $allFiles
    .filter((file) => !file.is_dir && isDocument(file.path))
    .filter((file) => (filter ? file.path.toLowerCase().includes(filter.toLowerCase()) : true))
    .slice(0, 60);

  onMount(() => {
    void load();
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        if (picking) picking = false;
        else if (linking) linking = null;
        else canvasOpen.set(false);
      }
      if ((event.key === "Delete" || event.key === "Backspace") && selected && !picking) {
        event.preventDefault();
        removeNode(selected);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  function toWorld(clientX: number, clientY: number, rect: DOMRect): { x: number; y: number } {
    return {
      x: (clientX - rect.left - panX) / zoom,
      y: (clientY - rect.top - panY) / zoom,
    };
  }

  async function load(): Promise<void> {
    loading = true;
    try {
      let content = "";
      try {
        content = await api.readFile(CANVAS_PATH);
      } catch {
        content = "";
      }
      if (!content.trim()) {
        nodes = [];
        edges = [];
        return;
      }
      const parsed = JSON.parse(content) as {
        nodes?: Array<Record<string, unknown>>;
        edges?: Array<Record<string, unknown>>;
      };
      nodes = (parsed.nodes ?? [])
        .filter((node) => typeof node?.id === "string")
        .map((node) => {
          const position = (node.position ?? {}) as { x?: number; y?: number };
          const data = (node.data ?? {}) as { label?: string; path?: string };
          return {
            id: String(node.id),
            x: Number(node.x ?? position.x ?? 0),
            y: Number(node.y ?? position.y ?? 0),
            label: data.label ?? baseName(data.path ?? String(node.id)),
            path: data.path ?? String(node.id),
          };
        });
      edges = (parsed.edges ?? [])
        .filter((edge) => typeof edge?.source === "string" && typeof edge?.target === "string")
        .map((edge) => ({
          id: String(edge.id ?? `${edge.source}-${edge.target}`),
          source: String(edge.source),
          target: String(edge.target),
        }));
      dirty = false;
    } catch (problem) {
      nodes = [];
      edges = [];
      pushToast("error", "This canvas could not be read", String(problem));
    } finally {
      loading = false;
    }
  }

  function serialize(): string {
    return JSON.stringify(
      {
        nodes: nodes.map((node) => ({
          id: node.id,
          position: { x: Math.round(node.x), y: Math.round(node.y) },
          data: { label: node.label, path: node.path },
        })),
        edges: edges.map((edge) => ({ id: edge.id, source: edge.source, target: edge.target })),
      },
      null,
      2
    );
  }

  async function save(): Promise<void> {
    saving = true;
    const content = serialize();
    try {
      try {
        await api.writeFile(CANVAS_PATH, content);
      } catch {
        await api.createFile(CANVAS_PATH, content);
      }
      dirty = false;
      pushToast("success", "Canvas saved", CANVAS_PATH);
    } catch (problem) {
      pushToast("error", "Could not save the canvas", String(problem));
    } finally {
      saving = false;
    }
  }

  function addNode(path: string): void {
    if (nodes.some((node) => node.path === path)) {
      pushToast("info", "That note is already on the canvas");
      return;
    }
    const index = nodes.length;
    nodes = [
      ...nodes,
      {
        id: path,
        x: 40 + (index % 4) * (CARD_W + 34),
        y: 40 + Math.floor(index / 4) * (CARD_H + 34),
        label: baseName(path),
        path,
      },
    ];
    dirty = true;
    picking = false;
    filter = "";
  }

  function removeNode(id: string): void {
    nodes = nodes.filter((node) => node.id !== id);
    edges = edges.filter((edge) => edge.source !== id && edge.target !== id);
    if (selected === id) selected = null;
    dirty = true;
  }

  function zoomAt(event: WheelEvent, rect: DOMRect): void {
    event.preventDefault();
    const before = toWorld(event.clientX, event.clientY, rect);
    const factor = event.deltaY < 0 ? 1.1 : 0.9;
    zoom = Math.min(2.2, Math.max(0.35, zoom * factor));
    panX = event.clientX - rect.left - before.x * zoom;
    panY = event.clientY - rect.top - before.y * zoom;
  }

  function startPan(event: PointerEvent): void {
    if (event.button !== 0) return;
    panning = true;
    selected = null;
    panStart = { x: event.clientX, y: event.clientY, panX, panY };
  }

  function movePan(event: PointerEvent): void {
    if (panning) {
      panX = panStart.panX + (event.clientX - panStart.x);
      panY = panStart.panY + (event.clientY - panStart.y);
      return;
    }
    if (moving) {
      const node = nodes.find((entry) => entry.id === moving);
      if (!node) return;
      const dx = (event.clientX - moveStart.x) / zoom;
      const dy = (event.clientY - moveStart.y) / zoom;
      nodes = nodes.map((entry) =>
        entry.id === moving
          ? { ...entry, x: Math.max(-400, moveStart.nodeX + dx), y: Math.max(-400, moveStart.nodeY + dy) }
          : entry
      );
      dirty = true;
      return;
    }
    if (linking) {
      const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
      const world = toWorld(event.clientX, event.clientY, rect);
      linking = { ...linking, wx: world.x, wy: world.y };
    }
  }

  function endPointer(event: PointerEvent): void {
    if (linking) {
      const target = (event.target as HTMLElement | null)?.closest("[data-node-id]");
      const targetId = target?.getAttribute("data-node-id");
      if (targetId && targetId !== linking.from) {
        const exists = edges.some(
          (edge) =>
            (edge.source === linking!.from && edge.target === targetId) ||
            (edge.source === targetId && edge.target === linking!.from)
        );
        if (!exists) {
          edges = [...edges, { id: `${linking.from}->${targetId}`, source: linking.from, target: targetId }];
          dirty = true;
        }
      }
      linking = null;
    }
    panning = false;
    moving = null;
  }

  function startMove(node: CanvasNode, event: PointerEvent): void {
    event.stopPropagation();
    selected = node.id;
    moving = node.id;
    moveStart = { x: event.clientX, y: event.clientY, nodeX: node.x, nodeY: node.y };
  }

  function startLink(node: CanvasNode, event: PointerEvent): void {
    event.stopPropagation();
    linking = { from: node.id, x: node.x + CARD_W, y: node.y + CARD_H / 2, wx: node.x + CARD_W, wy: node.y + CARD_H / 2 };
  }

  function nodeCenter(id: string): { x: number; y: number } {
    const node = nodes.find((entry) => entry.id === id);
    return node ? { x: node.x + CARD_W / 2, y: node.y + CARD_H / 2 } : { x: 0, y: 0 };
  }

  function close(): void {
    canvasOpen.set(false);
  }
</script>

<div class="overlay center" role="presentation" on:click={(event) => event.target === event.currentTarget && close()}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    tabindex="-1"
    class="dialog wide"
    role="dialog"
    aria-label="Canvas"
    on:click|stopPropagation
    style="height:min(84vh, 780px); display:flex; flex-direction:column"
  >
    <div class="dialog-header" style="align-items:center">
      <div class="titles">
        <h3>Canvas</h3>
        <p>Arrange notes spatially, then open any card. Saved to {CANVAS_PATH}.</p>
      </div>
      <button type="button" class="btn sm" on:click={() => (picking = !picking)}><Plus size={12} /> Add note</button>
      <button type="button" class="btn sm" disabled={saving || !dirty} on:click={() => void save()}>
        {#if saving}<Loader2 size={11} class="spin" />{:else}<Save size={11} />{/if}
        {dirty ? "Save" : "Saved"}
      </button>
      <button type="button" class="icon-btn" title="Close" aria-label="Close canvas" on:click={close}>
        <X size={15} />
      </button>
    </div>

    {#if picking}
      <div class="picker">
        <input bind:value={filter} placeholder="Find a note to place…" spellcheck="false" />
        <div class="picker-list">
          {#each documents as file (file.path)}
            <button type="button" class="picker-row" on:click={() => addNode(file.path)}>
              <span class="truncate">{file.name}</span>
              <span class="meta truncate">{file.path}</span>
            </button>
          {/each}
          {#if documents.length === 0}
            <p class="panel-empty">No documents match.</p>
          {/if}
        </div>
        <div class="picker-foot">
          <button
            type="button"
            class="btn sm"
            disabled={!$activeTab}
            on:click={() => $activeTab && addNode($activeTab.path)}
          >
            Add the open note
          </button>
        </div>
      </div>
    {/if}

    <div
      class="canvas"
      role="presentation"
      on:pointerdown={startPan}
      on:pointermove={movePan}
      on:pointerup={endPointer}
      on:pointerleave={endPointer}
      on:wheel={(event) => zoomAt(event, event.currentTarget.getBoundingClientRect())}
    >
      {#if loading}
        <div class="editor-loading"><Loader2 size={15} class="spin" /> Loading the canvas…</div>
      {:else}
        <svg class="canvas-svg">
          <g transform="translate({panX} {panY}) scale({zoom})">
            {#each edges as edge (edge.id)}
              <line
                class="canvas-edge"
                x1={nodeCenter(edge.source).x}
                y1={nodeCenter(edge.source).y}
                x2={nodeCenter(edge.target).x}
                y2={nodeCenter(edge.target).y}
              />
            {/each}
            {#if linking}
              <line
                class="canvas-edge pending"
                x1={nodeCenter(linking.from).x}
                y1={nodeCenter(linking.from).y}
                x2={linking.wx}
                y2={linking.wy}
              />
            {/if}
          </g>
        </svg>

        <div class="canvas-layer" style="transform: translate({panX}px, {panY}px) scale({zoom})">
          {#each nodes as node (node.id)}
            <div
              class="canvas-node"
              class:on={selected === node.id}
              data-node-id={node.id}
              style="left:{node.x}px; top:{node.y}px; width:{CARD_W}px; height:{CARD_H}px"
              role="button"
              tabindex="0"
              on:pointerdown={(event) => startMove(node, event)}
              on:dblclick={() => void openTab(node.path)}
              on:keydown={(event) => {
                if (event.key === "Enter") void openTab(node.path);
              }}
            >
              <span class="node-title truncate">{node.label}</span>
              <span class="node-path truncate">{node.path}</span>
              <button
                type="button"
                class="node-handle"
                title="Drag to connect"
                aria-label="Connect node"
                on:pointerdown={(event) => startLink(node, event)}
              ></button>
              <button
                type="button"
                class="node-remove"
                title="Remove from canvas"
                aria-label="Remove from canvas"
                on:pointerdown={(event) => event.stopPropagation()}
                on:click={() => removeNode(node.id)}
              >
                <Trash2 size={9} />
              </button>
            </div>
          {/each}
        </div>

        {#if nodes.length === 0}
          <div class="empty" style="background:transparent; pointer-events:none">
            <div class="empty-mark">·</div>
            <h2>An empty canvas</h2>
            <p>Add notes, drag them anywhere, and pull from a card's right edge to draw a connection.</p>
            <div class="empty-actions" style="pointer-events:auto">
              <button type="button" class="btn primary" on:click={() => (picking = true)}><Plus size={12} /> Add note</button>
            </div>
          </div>
        {/if}

        <div class="canvas-hint">
          <span>Drag to pan</span>
          <span>Scroll to zoom</span>
          <span>Double-click a card to open</span>
          {#if selected}<span><Check size={10} /> selected</span>{/if}
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .canvas {
    position: relative;
    flex: 1;
    min-height: 0;
    overflow: hidden;
    background: var(--nc-bg);
    cursor: grab;
    touch-action: none;
  }
  .canvas:active {
    cursor: grabbing;
  }
  .canvas-svg {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
  }
  .canvas-edge {
    stroke: var(--nc-border-strong);
    stroke-width: 1.2;
  }
  .canvas-edge.pending {
    stroke: var(--nc-accent);
    stroke-dasharray: 4 3;
  }
  .canvas-layer {
    position: absolute;
    inset: 0;
    transform-origin: 0 0;
  }
  .canvas-node {
    position: absolute;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 1px;
    padding: 0 12px;
    border: 1px solid var(--nc-border);
    border-radius: var(--nc-r-md);
    background: var(--nc-elev);
    box-shadow: var(--nc-shadow-sm);
    cursor: grab;
    user-select: none;
  }
  .canvas-node:hover {
    border-color: var(--nc-border-strong);
  }
  .canvas-node.on {
    border-color: var(--nc-accent);
    box-shadow: 0 0 0 1px var(--nc-accent-ring);
  }
  .node-title {
    font-size: 12px;
    color: var(--nc-fg);
  }
  .node-path {
    font-size: 9.5px;
    font-family: var(--nc-font-mono);
    color: var(--nc-fg-faint);
  }
  .node-handle {
    position: absolute;
    right: -5px;
    top: 50%;
    width: 9px;
    height: 9px;
    margin-top: -4.5px;
    border-radius: 50%;
    border: 1px solid var(--nc-border-strong);
    background: var(--nc-panel);
    cursor: crosshair;
  }
  .node-handle:hover {
    border-color: var(--nc-accent);
    background: var(--nc-accent);
  }
  .node-remove {
    position: absolute;
    left: -4px;
    top: -4px;
    display: none;
    place-items: center;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--nc-raised);
    border: 1px solid var(--nc-border);
    color: var(--nc-fg-muted);
  }
  .canvas-node:hover .node-remove,
  .canvas-node.on .node-remove {
    display: grid;
  }
  .node-remove:hover {
    color: var(--nc-danger);
    border-color: var(--nc-danger);
  }
  .canvas-hint {
    position: absolute;
    left: 12px;
    bottom: 10px;
    display: flex;
    gap: 12px;
    font-size: 10px;
    color: var(--nc-fg-faint);
    pointer-events: none;
  }
  .canvas-hint span {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }
  .picker {
    border-bottom: 1px solid var(--nc-border-soft);
    padding: 8px 12px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    background: var(--nc-panel);
  }
  .picker input {
    height: 28px;
    padding: 0 8px;
    border: 1px solid var(--nc-border);
    border-radius: var(--nc-r-sm);
    background: var(--nc-bg);
    color: var(--nc-fg);
    font: inherit;
    font-size: 11.5px;
  }
  .picker input:focus-visible {
    border-color: var(--nc-accent);
    outline: none;
  }
  .picker-list {
    max-height: 168px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }
  .picker-row {
    display: flex;
    align-items: baseline;
    gap: 8px;
    padding: 4px 6px;
    border-radius: var(--nc-r-sm);
    font-size: 11.5px;
    color: var(--nc-fg);
    text-align: left;
  }
  .picker-row:hover {
    background: var(--nc-hover);
  }
  .picker-foot {
    display: flex;
    justify-content: flex-end;
  }
</style>
