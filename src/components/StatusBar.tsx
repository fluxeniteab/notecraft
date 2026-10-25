import { useAppStore } from "../store/appStore";

export default function StatusBar() {
  const { vaultRoot, tabs, activeTabId } = useAppStore();
  const active = tabs.find((t) => t.id === activeTabId);
  return (
    <div className="flex items-center gap-3 px-4 py-1.5 border-t border-border bg-bg-elev/60 text-[11px] text-fg-muted backdrop-blur-sm">
      <span className="flex items-center gap-1.5">
        <span className="w-1.5 h-1.5 rounded-full bg-success" />
        {vaultRoot ? "Vault open" : "No vault"}
      </span>
      {active && <span className="truncate max-w-[40%]">· {active.path}</span>}
      <div className="flex-1" />
      <span className="tabular-nums">UTF-8 · Markdown</span>
    </div>
  );
}
