import { useCallback, useEffect } from "react";
import { ReactFlow, Background, Controls, MiniMap, addEdge, useNodesState, useEdgesState, type Connection, type Edge, type Node } from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { api } from "../lib/tauri";
import { X, Plus } from "lucide-react";

const CANVAS_PATH = "canvas/main.canvas.json";

export default function CanvasView({ open, onClose }: { open: boolean; onClose: () => void }) {
  const [nodes, setNodes, onNodesChange] = useNodesState<Node>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);


  useEffect(() => {
    if (!open) return;
    api.readFile(CANVAS_PATH).then((c) => {
      try {
        const d = JSON.parse(c);
        setNodes(d.nodes ?? []);
        setEdges(d.edges ?? []);
      } catch { /* ignore */ }
    }).catch(() => { /* ignore */ });
  }, [open]);

  const onConnect = useCallback((c: Connection) => setEdges((eds) => addEdge({ ...c, id: crypto.randomUUID() }, eds)), [setEdges]);

  const save = async () => {
    try {
      await api.createDir("canvas");
    } catch {}
    await api.writeFile(CANVAS_PATH, JSON.stringify({ nodes, edges, version: 1 }, null, 2));
    alert("Canvas saved");
  };

  const addNoteCard = () => {
    const id = crypto.randomUUID();
    setNodes((ns) => [...ns, { id, position: { x: 100, y: 100 }, data: { label: "New note card" }, type: "default" }]);
  };

  if (!open) return null;
  return (
    <div className="fixed inset-0 z-50 bg-bg flex flex-col">
      <div className="flex items-center gap-2 px-3 py-2 border-b border-border bg-bg-elev">
        <span className="font-semibold text-sm">Canvas</span>
        <button onClick={addNoteCard} className="flex items-center gap-1 text-xs px-2 py-1 rounded bg-bg-muted hover:bg-bg"><Plus size={12} /> Add card</button>
        <button onClick={save} className="text-xs px-2 py-1 rounded bg-accent text-accent-fg">Save</button>
        <div className="flex-1" />
        <button onClick={onClose}><X size={16} /></button>
      </div>
      <div className="flex-1">
        <ReactFlow nodes={nodes} edges={edges} onNodesChange={onNodesChange} onEdgesChange={onEdgesChange} onConnect={onConnect} fitView>
          <Background />
          <Controls />
          <MiniMap />
        </ReactFlow>
      </div>
    </div>
  );
}
