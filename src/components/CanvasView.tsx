import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  ReactFlow,
  Background,
  BackgroundVariant,
  Controls,
  MiniMap,
  addEdge,
  useNodesState,
  useEdgesState,
  type Connection,
  type Edge,
  type Node,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { toast } from "sonner";
import { Check, Loader2, Plus, Save, Workflow, X } from "lucide-react";
import { api } from "../lib/tauri";
import { useAppStore } from "../store/appStore";
import { IconButton } from "./ui";

const CANVAS_PATH = "canvas/main.canvas.json";

interface CanvasDoc {
  nodes: Node[];
  edges: Edge[];
}

export default function CanvasView({ open, onClose }: { open: boolean; onClose: () => void }) {
  const theme = useAppStore((s) => s.theme);
  const [nodes, setNodes, onNodesChange] = useNodesState<Node>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [dirty, setDirty] = useState(false);
  const savedRef = useRef("");

  useEffect(() => {
    if (!open) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape" && !saving) onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onClose, saving]);

  useEffect(() => {
    if (!open) return;
    let cancelled = false;
    setLoading(true);
    api
      .readFile(CANVAS_PATH)
      .then((content) => {
        if (cancelled) return;
        try {
          const doc = JSON.parse(content) as Partial<CanvasDoc>;
          const nextNodes = Array.isArray(doc.nodes) ? doc.nodes : [];
          const nextEdges = Array.isArray(doc.edges) ? doc.edges : [];
          setNodes(nextNodes);
          setEdges(nextEdges);
          savedRef.current = JSON.stringify({ nodes: nextNodes, edges: nextEdges });
          setDirty(false);
        } catch {
          toast.error("This canvas file could not be read", {
            description: "It is not valid JSON. Starting with an empty canvas.",
          });
          setNodes([]);
          setEdges([]);
          savedRef.current = "";
          setDirty(false);
        }
      })
      .catch(() => {
        // A missing canvas is normal on first open.
        if (cancelled) return;
        setNodes([]);
        setEdges([]);
        savedRef.current = "";
        setDirty(false);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [open, setEdges, setNodes]);

  // Track changes without serialising on every render.
  useEffect(() => {
    if (!open || loading) return;
    const serialized = JSON.stringify({ nodes, edges });
    if (savedRef.current && serialized !== savedRef.current) setDirty(true);
  }, [nodes, edges, open, loading]);

  const onConnect = useCallback(
    (connection: Connection) =>
      setEdges((current) =>
        addEdge({ ...connection, id: crypto.randomUUID(), type: "smoothstep" }, current)
      ),
    [setEdges]
  );

  const save = useCallback(async () => {
    if (saving) return;
    setSaving(true);
    try {
      await api.createDir("canvas").catch(() => undefined);
      await api.writeFile(
        CANVAS_PATH,
        JSON.stringify({ nodes, edges, version: 1 }, null, 2)
      );
      savedRef.current = JSON.stringify({ nodes, edges });
      setDirty(false);
      toast.success("Canvas saved", { description: CANVAS_PATH });
    } catch (error) {
      toast.error("Could not save the canvas", { description: String(error) });
    } finally {
      setSaving(false);
    }
  }, [edges, nodes, saving]);

  const addNoteCard = () => {
    const id = crypto.randomUUID();
    const offset = nodes.length * 24;
    setNodes((current) => [
      ...current,
      {
        id,
        position: { x: 120 + offset, y: 120 + offset },
        data: { label: "New note card" },
        type: "default",
      },
    ]);
  };

  const minimapStyle = useMemo(
    () => ({
      background: "var(--nc-surface)",
      border: "1px solid var(--nc-border)",
      borderRadius: "var(--nc-r-sm)",
    }),
    []
  );

  if (!open) return null;

  return (
    <div className="nc-overlay-view nc-anim-fade" role="dialog" aria-modal="true" aria-label="Canvas">
      <div className="nc-overlay-head">
        <Workflow size={14} className="text-accent" />
        <span className="nc-overlay-title">Canvas</span>
        <span className="nc-chip">
          {nodes.length} cards · {edges.length} connections
        </span>
        <span className="nc-save-chip" data-state={dirty ? "dirty" : "saved"}>
          {dirty ? (
            <>
              <span className="h-1.5 w-1.5 rounded-full bg-current" /> Unsaved
            </>
          ) : (
            <>
              <Check size={11} /> Saved
            </>
          )}
        </span>
        <div className="flex-1" />
        <button type="button" className="nc-btn nc-btn-outline nc-btn-sm" onClick={addNoteCard}>
          <Plus size={13} /> Add card
        </button>
        <button
          type="button"
          className="nc-btn nc-btn-primary nc-btn-sm"
          onClick={() => void save()}
          disabled={saving}
        >
          {saving ? <Loader2 size={13} className="animate-spin" /> : <Save size={13} />}
          Save
        </button>
        <IconButton label="Close canvas" shortcut="Esc" small onClick={onClose}>
          <X size={15} />
        </IconButton>
      </div>

      <div className="relative min-h-0 flex-1">
        <ReactFlow
          nodes={nodes}
          edges={edges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          colorMode={theme}
          fitView
          proOptions={{ hideAttribution: true }}
          className="nc-canvas-flow"
        >
          <Background
            variant={BackgroundVariant.Dots}
            gap={18}
            size={1}
            color={theme === "dark" ? "#2a2a33" : "#d8d8e0"}
          />
          <Controls showInteractive={false} />
          <MiniMap style={minimapStyle} pannable zoomable maskColor="rgba(0,0,0,0.35)" />
        </ReactFlow>

        {loading && (
          <div className="pointer-events-none absolute inset-0 grid place-items-center">
            <div className="flex items-center gap-2 text-[11.5px] text-fg-muted">
              <span className="nc-loading-bar w-24" /> Opening canvas…
            </div>
          </div>
        )}

        {!loading && nodes.length === 0 && (
          <div className="pointer-events-none absolute inset-0 grid place-items-center px-6 text-center">
            <div className="max-w-sm">
              <div className="nc-empty-orb mx-auto">
                <Workflow size={18} />
              </div>
              <div className="mt-3 text-[13px] font-medium">A blank canvas</div>
              <p className="mt-1 text-[11.5px] leading-relaxed text-fg-muted">
                Add cards, then drag from a card edge to connect ideas. The canvas is stored as
                plain JSON inside your vault.
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
