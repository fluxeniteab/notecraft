import { useAppStore } from "../store/appStore";
import { X, Pin } from "lucide-react";
import { cn } from "../lib/cn";

export default function TabBar() {
  const { tabs, activeTabId, setActiveTab, closeTab } = useAppStore();
  if (tabs.length === 0) return null;
  return (
    <div className="flex items-center gap-0.5 px-2 py-1 border-b border-border bg-bg-elev overflow-x-auto nc-scroll">
      {tabs.map((t) => (
        <div
          key={t.id}
          className={cn(
            "group flex items-center gap-1.5 px-2.5 py-1 rounded-t text-xs cursor-pointer border border-transparent",
            activeTabId === t.id ? "bg-bg border-border" : "hover:bg-bg-muted"
          )}
          onClick={() => setActiveTab(t.id)}
        >
          {t.pinned && <Pin size={10} />}
          <span className="truncate max-w-[160px]">{t.title}</span>
          {t.dirty && <span className="text-accent">*</span>}
          <button
            className="opacity-0 group-hover:opacity-100 p-0.5 rounded hover:bg-bg-muted"
            onClick={(e) => { e.stopPropagation(); closeTab(t.id); }}
          >
            <X size={11} />
          </button>
        </div>
      ))}
    </div>
  );
}
