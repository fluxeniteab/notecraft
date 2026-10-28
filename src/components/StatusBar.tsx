import { CloudOff, FileText } from "lucide-react";
import { useAppStore } from "../store/appStore";
import { baseName } from "../lib/files";

export default function StatusBar() {
  const vaultRoot = useAppStore((s) => s.vaultRoot);
  const tabs = useAppStore((s) => s.tabs);
  const activeTabId = useAppStore((s) => s.activeTabId);
  const tabStats = useAppStore((s) => s.tabStats);
  const cursors = useAppStore((s) => s.cursors);
  const saveStatus = useAppStore((s) => s.saveStatus);
  const allFiles = useAppStore((s) => s.allFiles);

  const active = tabs.find((tab) => tab.id === activeTabId) ?? null;
  const stats = active ? tabStats[active.id] : undefined;
  const cursor = active ? cursors[active.id] : undefined;
  const status = active ? saveStatus[active.id] ?? "idle" : null;
  const noteCount = allFiles.filter((file) => !file.is_dir && /\.(md|markdown)$/i.test(file.path)).length;

  const statusLabel =
    status === "dirty"
      ? "Unsaved changes"
      : status === "saving"
        ? "Saving…"
        : status === "error"
          ? "Save failed"
          : status === "saved"
            ? "All changes saved"
            : null;

  return (
    <footer className="nc-statusbar">
      <span className="nc-status-item">
        <span className="nc-status-dot" data-off={!vaultRoot || undefined} />
        {vaultRoot ? baseName(vaultRoot) : "No vault"}
      </span>

      {vaultRoot && (
        <span className="nc-status-item text-fg-faint">
          <FileText size={10} />
          {noteCount.toLocaleString()} {noteCount === 1 ? "note" : "notes"}
        </span>
      )}

      {active && (
        <span className="nc-status-item min-w-0 text-fg-faint">
          <span className="truncate" title={active.path}>
            {active.path}
          </span>
          {statusLabel && (
            <>
              <span className="opacity-40">·</span>
              <span
                className="truncate"
                style={{
                  color:
                    status === "error"
                      ? "var(--nc-danger)"
                      : status === "dirty"
                        ? "var(--nc-warning)"
                        : "var(--nc-fg-faint)",
                }}
              >
                {statusLabel}
              </span>
            </>
          )}
        </span>
      )}

      <div className="flex-1" />

      {active && cursor && (
        <span className="nc-status-item tabular-nums">
          Ln {cursor.line}, Col {cursor.col}
        </span>
      )}
      {stats && (
        <span className="nc-status-item tabular-nums">
          {stats.words.toLocaleString()} words · {stats.chars.toLocaleString()} chars
        </span>
      )}
      <span className="nc-status-item">
        <CloudOff size={10} />
        Local only
      </span>
      <span className="nc-status-item">UTF-8 · Markdown</span>
    </footer>
  );
}
