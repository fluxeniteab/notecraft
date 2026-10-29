<script lang="ts">
  import { onMount } from "svelte";
  import { Loader2, RefreshCw, X } from "lucide-svelte";
  import type { GraphData } from "../lib/types";
  import { graphOpen, openTab, pushToast } from "../lib/state";
  import { api } from "../lib/ipc";
  import { baseName } from "../lib/files";

  interface Point {
    id: string;
    label: string;
    x: number;
    y: number;
    vx: number;
    vy: number;
    degree: number;
  }

  let loading = true;
  let error = "";
  let nodes: Point[] = [];
  let edges: Array<{ a: Point; b: Point }> = [];
  let totalNodes = 0;
  let totalEdges = 0;
  let zoom = 1;
  let panX = 0;
  let panY = 0;
  let dragging = false;
  let lastX = 0;
  let lastY = 0;
  let hover: string | null = null;
  let filter = "";

  const width = 1000;
  const height = 640;
  const LIMIT = 320;

  onMount(() => {
    void load();
    const onEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") graphOpen.set(false);
    };
    window.addEventListener("keydown", onEscape);
    return () => window.removeEventListener("keydown", onEscape);
  });

  async function load(): Promise<void> {
    loading = true;
    error = "";
    try {
      const data: GraphData = await api.graphData("global");
      totalNodes = data.nodes.length;
      totalEdges = data.edges.length;
      const degree = new Map<string, number>();
      for (const edge of data.edges) {
        degree.set(edge.source, (degree.get(edge.source) ?? 0) + 1);
        degree.set(edge.target, (degree.get(edge.target) ?? 0) + 1);
      }
      const chosen = [...data.nodes]
        .sort((a, b) => (degree.get(b.id) ?? 0) - (degree.get(a.id) ?? 0))
        .slice(0, LIMIT);
      const ids = new Set(chosen.map((node) => node.id));
      const points = new Map<string, Point>();
      chosen.forEach((node, index) => {
        const angle = (index / Math.max(1, chosen.length)) * Math.PI * 2;
        const radius = 220 + (index % 5) * 22;
        points.set(node.id, {
          id: node.id,
          label: node.label || baseName(node.id),
          x: width / 2 + Math.cos(angle) * radius,
          y: height / 2 + Math.sin(angle) * radius,
          vx: 0,
          vy: 0,
          degree: degree.get(node.id) ?? 0,
        });
      });
      nodes = [...points.values()];
      edges = data.edges
        .filter((edge) => ids.has(edge.source) && ids.has(edge.target))
        .map((edge) => ({ a: points.get(edge.source)!, b: points.get(edge.target)! }));

      // A compact force layout: repulsion between all pairs, springs along
      // links, gravity toward the centre. Deterministic, so the graph is stable.
      for (let step = 0; step < 220; step += 1) {
        const alpha = 1 - step / 230;
        for (let i = 0; i < nodes.length; i += 1) {
          const a = nodes[i];
          for (let j = i + 1; j < nodes.length; j += 1) {
            const b = nodes[j];
            let dx = a.x - b.x;
            let dy = a.y - b.y;
            let dist = Math.hypot(dx, dy) || 1;
            const force = (1400 * alpha) / (dist * dist);
            dx = (dx / dist) * force;
            dy = (dy / dist) * force;
            a.vx += dx;
            a.vy += dy;
            b.vx -= dx;
            b.vy -= dy;
          }
        }
        for (const edge of edges) {
          const dx = edge.b.x - edge.a.x;
          const dy = edge.b.y - edge.a.y;
          const dist = Math.hypot(dx, dy) || 1;
          const force = (dist - 90) * 0.012 * alpha;
          edge.a.vx += (dx / dist) * force;
          edge.a.vy += (dy / dist) * force;
          edge.b.vx -= (dx / dist) * force;
          edge.b.vy -= (dy / dist) * force;
        }
        for (const node of nodes) {
          node.vx += (width / 2 - node.x) * 0.0015 * alpha;
          node.vy += (height / 2 - node.y) * 0.0015 * alpha;
          node.x = Math.max(22, Math.min(width - 22, node.x + node.vx * 0.4));
          node.y = Math.max(22, Math.min(height - 22, node.y + node.vy * 0.4));
          node.vx *= 0.72;
          node.vy *= 0.72;
        }
      }
    } catch (problem) {
      error = String(problem);
    } finally {
      loading = false;
    }
  }

  function close(): void {
    graphOpen.set(false);
  }

  function onWheel(event: WheelEvent): void {
    event.preventDefault();
    zoom = Math.max(0.35, Math.min(3, zoom * (event.deltaY < 0 ? 1.12 : 0.89)));
  }

  function startDrag(event: PointerEvent): void {
    dragging = true;
    lastX = event.clientX;
    lastY = event.clientY;
  }

  function drag(event: PointerEvent): void {
    if (!dragging) return;
    panX += event.clientX - lastX;
    panY += event.clientY - lastY;
    lastX = event.clientX;
    lastY = event.clientY;
  }

  function open(node: Point): void {
    close();
    void openTab(node.id).catch((problem) => pushToast("error", "Could not open that document", String(problem)));
  }

  $: needle = filter.trim().toLowerCase();
  $: highlighted = new Set(
    nodes
      .filter((node) => !needle || node.label.toLowerCase().includes(needle))
      .map((node) => node.id)
  );
  $: shown = highlighted.size;
</script>

<div class="overlay center" role="presentation" on:click={(event) => event.target === event.currentTarget && close()}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div tabindex="-1" class="dialog wide" role="dialog" aria-label="Knowledge graph" on:click|stopPropagation style="height:min(82vh, 760px); display:flex; flex-direction:column">
    <div class="dialog-header" style="align-items:center">
      <div class="titles">
        <h3>Knowledge graph</h3>
        <p>
          {totalNodes.toLocaleString()} documents · {totalEdges.toLocaleString()} links
          {#if totalNodes > LIMIT}· showing the {LIMIT} most connected{/if}
        </p>
      </div>
      <input class="input" style="width:180px" bind:value={filter} placeholder="Highlight node…" spellcheck="false" />
      <button type="button" class="icon-btn" title="Rebuild layout" aria-label="Rebuild layout" on:click={() => void load()}>
        <RefreshCw size={14} />
      </button>
      <button type="button" class="icon-btn" title="Close" aria-label="Close graph" on:click={close}><X size={15} /></button>
    </div>

    <div class="graph-wrap">
      {#if loading}
        <div class="editor-loading"><Loader2 size={17} class="spin" /> Building the graph…</div>
      {:else if error}
        <div class="editor-error"><strong>Could not load the graph</strong><div class="hint">{error}</div></div>
      {:else if nodes.length === 0}
        <div class="empty" style="background:transparent">
          <div class="empty-mark">·</div>
          <h2>No links yet</h2>
          <p>Graph nodes appear when documents reference each other with <code>[[wiki links]]</code>.</p>
        </div>
      {:else}
        <button
          type="button"
          style="position:absolute; inset:0; width:100%; height:100%; cursor:grab; background:none"
          aria-label="Pan the graph"
          on:wheel={onWheel}
          on:pointerdown={startDrag}
          on:pointermove={drag}
          on:pointerup={() => (dragging = false)}
          on:pointerleave={() => (dragging = false)}
        ></button>
        <svg viewBox="0 0 {width} {height}" style="position:relative; pointer-events:none">
          <g transform="translate({panX} {panY}) scale({zoom})">
            {#each edges as edge}
              <line class="graph-edge" x1={edge.a.x} y1={edge.a.y} x2={edge.b.x} y2={edge.b.y} />
            {/each}
            {#each nodes as node (node.id)}
              <g
                class="graph-node"
                class:on={hover === node.id}
                style="pointer-events:auto; opacity:{highlighted.has(node.id) ? 1 : 0.22}"
                transform="translate({node.x} {node.y})"
                role="button"
                tabindex="0"
                on:click={() => open(node)}
                on:keydown={(event) => {
                  if (event.key === "Enter") open(node);
                }}
                on:mouseenter={() => (hover = node.id)}
                on:mouseleave={() => (hover = null)}
              >
                <circle r={5 + Math.min(8, Math.sqrt(node.degree) * 2.2)} />
                <text text-anchor="middle" y={-11 - Math.min(8, Math.sqrt(node.degree) * 2.2)}>{node.label}</text>
              </g>
            {/each}
          </g>
        </svg>
        <div class="graph-legend">
          {shown === nodes.length ? `${nodes.length} visible` : `${shown} of ${nodes.length}`} · scroll to zoom, drag to pan
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .graph-legend {
    position: absolute;
    left: 12px;
    bottom: 10px;
    color: var(--nc-fg-faint);
    font-size: 10px;
    pointer-events: none;
  }
</style>
