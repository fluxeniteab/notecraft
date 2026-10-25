import { useEffect, useRef, useState } from "react";
import { useAppStore } from "../store/appStore";
import { api } from "../lib/tauri";
import type { GraphData } from "../lib/types";
import Graph from "graphology";
import Sigma from "sigma";
import { X } from "lucide-react";

export default function GraphView({ open, onClose }: { open: boolean; onClose: () => void }) {
  const containerRef = useRef<HTMLDivElement>(null);
  const sigmaRef = useRef<Sigma | null>(null);
  const [data, setData] = useState<GraphData | null>(null);
  const { openTab } = useAppStore();

  useEffect(() => {
    if (!open) return;
    api.graphData("global").then(setData).catch(() => setData(null));
  }, [open]);

  useEffect(() => {
    if (!open || !containerRef.current || !data) return;
    const g = new Graph();
    data.nodes.forEach((n, i) => {
      g.addNode(n.id, { label: n.label, x: n.x ?? Math.cos(i) * 10, y: n.y ?? Math.sin(i) * 10, size: n.size ?? 6, color: n.color ?? "#8b5cf6" });
    });
    data.edges.forEach((e) => {
      if (g.hasNode(e.source) && g.hasNode(e.target) && !g.hasEdge(e.source, e.target)) {
        try { g.addEdge(e.source, e.target, { size: 1 }); } catch { /* ignore */ }
      }
    });
    const sigma = new Sigma(g, containerRef.current, { renderLabels: true });
    sigma.on("clickNode", ({ node }) => { openTab(node); onClose(); });
    sigmaRef.current = sigma;
    return () => { sigma.kill(); sigmaRef.current = null; };
  }, [open, data]);

  if (!open) return null;
  return (
    <div className="fixed inset-0 z-50 bg-bg/98 flex flex-col nc-anim-fade">
      <div className="flex items-center px-3 py-2 border-b border-border">
        <span className="font-semibold text-sm">Knowledge Graph</span>
        <div className="flex-1" />
        <button onClick={onClose}><X size={16} /></button>
      </div>
      <div ref={containerRef} className="flex-1" />
      {!data && <div className="absolute inset-0 flex items-center justify-center text-fg-muted">Loading…</div>}
    </div>
  );
}
