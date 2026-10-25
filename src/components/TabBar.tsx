import { useAppStore } from "../store/appStore";
import { X } from "lucide-react";
import { cn } from "../lib/cn";

export default function TabBar() {
  const { tabs, activeTabId, setActiveTab, closeTab } = useAppStore();
  if (tabs.length === 0) return null;
  return (
    <div className="flex items-center gap-1 px-3 py-1.5 border-b border-border bg-bg-elev/60 backdrop-blur-sm overflow-x-auto nc-scroll">
      {tabs.map((t) => (
        <div
          key={t.id}
          className={cn(
            "group relative flex items-center gap-2 px-3 py-1.5 rounded-md text-xs cursor-pointer transition-all duration-150",
            activeTabId === t.id ? "bg-bg-muted text-fg" : "text-fg-muted hover:bg-bg-muted/60 hover:text-fg"
          )}
          onClick={() => setActiveTab(t.id)}
        >
          {activeTabId === t.id && (
            <span className="absolute bottom-0 left-2 right-2 h-0.5 bg-accent rounded-full nc-anim-fade" />
          )}
          <span className="truncate max-w-[160px]">{t.title}</span>
          {t.dirty && <span className="w-1.5 h-1.5 rounded-full bg-accent" />}
          <button
            className="opacity-0 group-hover:opacity-100 p-0.5 rounded hover:bg-bg transition-opacity"
            onClick={(e) => { e.stopPropagation(); closeTab(t.id); }}
          >
            <X size={11} />
          </button>
        </div>
      ))}
    </div>
  );
}
