import { useEffect, useMemo, useRef, useState } from "react";
import Graph from "graphology";
import Sigma from "sigma";
import { Network, RotateCw, X } from "lucide-react";
import { useAppStore } from "../store/appStore";
import { api } from "../lib/tauri";
import type { GraphData } from "../lib/types";
import { stem } from "../lib/files";
import { IconButton } from "./ui";

function cssVar(name: string, fallback: string): string {
  if (typeof window === "undefined") return fallback;
  const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  return value || fallback;
}

export default function GraphView({ open, onClose }: { open: boolean; onClose: () => void }) {
  const containerRef = useRef<HTMLDivElement>(null);
  const sigmaRef = useRef<Sigma | null>(null);
  const [data, setData] = useState<GraphData | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [reloadKey, setReloadKey] = useState(0);
  const theme = useAppStore((s) => s.theme);
  const openTab = useAppStore((s) => s.openTab);

  useEffect(() => {
    if (!open) return;
    let cancelled = false;
    setLoading(true);
    setError(null);
    api
      .graphData("global")
      .then((graph) => {
        if (!cancelled) setData(graph);
      })
      .catch((err) => {
        if (cancelled) return;
        setData(null);
        setError(String(err).replace(/^Error:\s*/, ""));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [open, reloadKey]);

  useEffect(() => {
    if (!open) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onClose]);

  const counts = useMemo(
    () => ({ nodes: data?.nodes.length ?? 0, edges: data?.edges.length ?? 0 }),
    [data]
  );

  useEffect(() => {
    const host = containerRef.current;
    if (!open || !host || !data) return;
    sigmaRef.current?.kill();
    sigmaRef.current = null;

    const graph = new Graph();
    const seeded = new Set<string>();
    data.nodes.forEach((node, index) => {
      if (seeded.has(node.id)) return;
      seeded.add(node.id);
      const angle = (index / Math.max(1, data.nodes.length)) * Math.PI * 2;
      graph.addNode(node.id, {
        label: node.label || stem(node.id),
        x: node.x ?? Math.cos(angle) * 10,
        y: node.y ?? Math.sin(angle) * 10,
        size: node.size ?? 6,
        color: node.color ?? cssVar("--nc-accent", "#8b5cf6"),
      });
    });

    data.edges.forEach((edge) => {
      if (!graph.hasNode(edge.source) || !graph.hasNode(edge.target)) return;
      if (edge.source === edge.target) return;
      if (graph.hasEdge(edge.source, edge.target)) return;
      try {
        graph.addEdge(edge.source, edge.target, { size: 1 });
      } catch {
        /* ignore parallel / duplicate edges */
      }
    });

    // Make well-connected notes visually larger.
    graph.forEachNode((node, attributes) => {
      graph.setNodeAttribute(node, "size", (attributes.size as number) + Math.min(8, graph.degree(node)));
    });

    const accent = cssVar("--nc-accent", "#8b5cf6");
    const sigma = new Sigma(graph, host, {
      renderLabels: true,
      renderEdgeLabels: false,
      labelDensity: 0.45,
      labelGridCellSize: 70,
      labelRenderedSizeThreshold: 6,
      labelColor: { color: cssVar("--nc-fg-secondary", "#a1a1aa") },
      labelSize: 11,
      labelFont: "Inter Variable, Inter, system-ui, sans-serif",
      defaultNodeColor: accent,
      defaultEdgeColor: cssVar("--nc-border-strong", "#373742"),
      minCameraRatio: 0.08,
      maxCameraRatio: 8,
      allowInvalidContainer: true,
    });
    sigma.on("clickNode", ({ node }) => {
      openTab(node);
      onClose();
    });
    sigmaRef.current = sigma;

    return () => {
      sigma.kill();
      if (sigmaRef.current === sigma) sigmaRef.current = null;
    };
  }, [open, data, theme, openTab, onClose]);

  if (!open) return null;

  const empty = !loading && !error && data && data.nodes.length === 0;

  return (
    <div className="nc-overlay-view nc-anim-fade" role="dialog" aria-modal="true" aria-label="Knowledge graph">
      <div className="nc-overlay-head">
        <Network size={14} className="text-accent" />
        <span className="nc-overlay-title">Knowledge graph</span>
        <span className="nc-chip">
          {counts.nodes.toLocaleString()} notes · {counts.edges.toLocaleString()} links
        </span>
        <div className="flex-1" />
        <span className="hidden text-[10px] text-fg-faint sm:inline">
          Click a note to open it
        </span>
        <IconButton label="Reload graph" small onClick={() => setReloadKey((key) => key + 1)}>
          <RotateCw size={13} />
        </IconButton>
        <IconButton label="Close graph" shortcut="Esc" small onClick={onClose}>
          <X size={15} />
        </IconButton>
      </div>

      <div className="relative min-h-0 flex-1">
        <div ref={containerRef} className="h-full w-full" />

        {loading && (
          <div className="pointer-events-none absolute inset-0 grid place-items-center">
            <div className="flex items-center gap-2 text-[11.5px] text-fg-muted">
              <span className="nc-loading-bar w-24" /> Laying out your notes…
            </div>
          </div>
        )}

        {error && (
          <div className="absolute inset-0 grid place-items-center px-6 text-center">
            <div className="max-w-sm">
              <div className="nc-empty-orb mx-auto text-danger">
                <Network size={18} />
              </div>
              <div className="mt-3 text-[13px] font-medium">The graph could not be built</div>
              <p className="mt-1 text-[11.5px] leading-relaxed text-fg-muted">{error}</p>
              <button
                type="button"
                className="nc-btn nc-btn-outline mt-3"
                onClick={() => setReloadKey((key) => key + 1)}
              >
                <RotateCw size={13} /> Try again
              </button>
            </div>
          </div>
        )}

        {empty && (
          <div className="absolute inset-0 grid place-items-center px-6 text-center">
            <div className="max-w-sm">
              <div className="nc-empty-orb mx-auto">
                <Network size={18} />
              </div>
              <div className="mt-3 text-[13px] font-medium">No notes to graph yet</div>
              <p className="mt-1 text-[11.5px] leading-relaxed text-fg-muted">
                Create notes and connect them with <span className="text-fg">[[double
                brackets]]</span>. Relationships appear here automatically.
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
