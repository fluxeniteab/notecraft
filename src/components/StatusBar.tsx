import { useAppStore } from "../store/appStore";

export default function StatusBar() {
  const { vaultRoot, tabs, activeTabId } = useAppStore();
  const active = tabs.find((t) => t.id === activeTabId);
  return (
    <div className="flex items-center gap-3 px-3 py-1 border-t border-border bg-bg-elev text-[10px] text-fg-muted">
      <span>{vaultRoot ? "Vault open" : "No vault"}</span>
      {active && <span>· {active.path}</span>}
      <div className="flex-1" />
      <span>UTF-8 · Markdown</span>
    </div>
  );
}
