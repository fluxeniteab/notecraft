import { useEffect, useRef } from "react";
import { toast } from "sonner";
import { FileText, Plus, X } from "lucide-react";
import { useAppStore } from "../store/appStore";
import { flushTab } from "../lib/saveRegistry";
import { createNote } from "../lib/noteActions";
import { Tip } from "./ui";

export default function TabBar() {
  const tabs = useAppStore((s) => s.tabs);
  const activeTabId = useAppStore((s) => s.activeTabId);
  const setActiveTab = useAppStore((s) => s.setActiveTab);
  const closeTab = useAppStore((s) => s.closeTab);
  const reorderTabs = useAppStore((s) => s.reorderTabs);
  const tabRefs = useRef(new Map<string, HTMLDivElement>());
  const dragFrom = useRef<number | null>(null);

  useEffect(() => {
    if (!activeTabId) return;
    const el = tabRefs.current.get(activeTabId);
    el?.scrollIntoView({ block: "nearest", inline: "nearest" });
  }, [activeTabId]);

  const handleClose = async (id: string) => {
    await flushTab(id);
    if (useAppStore.getState().saveStatus[id] === "error") {
      toast.error("This note could not be saved, so it was kept open");
      return;
    }
    closeTab(id);
  };

  if (tabs.length === 0) return null;

  return (
    <div className="nc-tabstrip nc-scroll" role="tablist" aria-label="Open notes">
      {tabs.map((tab, index) => (
        <div
          key={tab.id}
          ref={(node) => {
            if (node) tabRefs.current.set(tab.id, node);
            else tabRefs.current.delete(tab.id);
          }}
          role="tab"
          tabIndex={0}
          aria-selected={tab.id === activeTabId}
          data-active={tab.id === activeTabId}
          className="nc-tab"
          title={tab.path}
          draggable
          onClick={() => setActiveTab(tab.id)}
          onKeyDown={(event) => {
            if (event.key === "Enter" || event.key === " ") setActiveTab(tab.id);
            if (event.key === "Delete" || (event.key === "w" && (event.ctrlKey || event.metaKey))) {
              event.preventDefault();
              void handleClose(tab.id);
            }
          }}
          onAuxClick={(event) => {
            if (event.button === 1) {
              event.preventDefault();
              void handleClose(tab.id);
            }
          }}
          onDragStart={(event) => {
            dragFrom.current = index;
            event.dataTransfer.effectAllowed = "move";
            event.dataTransfer.setData("text/notecraft-tab", String(index));
          }}
          onDragOver={(event) => {
            event.preventDefault();
            event.dataTransfer.dropEffect = "move";
          }}
          onDrop={(event) => {
            event.preventDefault();
            const from = Number(event.dataTransfer.getData("text/notecraft-tab") || dragFrom.current);
            if (Number.isInteger(from) && from >= 0) reorderTabs(from, index);
            dragFrom.current = null;
          }}
        >
          <FileText size={12} className={tab.id === activeTabId ? "text-accent" : "text-fg-faint"} />
          <span className="nc-tab-title">{tab.title}</span>
          {tab.dirty && (
            <span
              className="h-1.5 w-1.5 shrink-0 rounded-full bg-accent"
              title="Unsaved changes"
              style={{ animation: "nc-pulse-dot 2.4s ease-in-out infinite" }}
            />
          )}
          <button
            type="button"
            className="nc-tab-close"
            aria-label={`Close ${tab.title}`}
            onClick={(event) => {
              event.stopPropagation();
              void handleClose(tab.id);
            }}
          >
            <X size={11} />
          </button>
        </div>
      ))}
      <Tip label="New note" shortcut="Ctrl N">
        <button type="button" className="nc-tab-add" aria-label="New note" onClick={() => void createNote()}>
          <Plus size={14} />
        </button>
      </Tip>
    </div>
  );
}
