import { AlertTriangle, Check, CloudOff, FileText } from "lucide-react";
import { useAppStore } from "../store/appStore";
import { baseName } from "../lib/files";
import { capabilitiesFor } from "../lib/documentTypes";

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
  const caps = active ? capabilitiesFor(active.path) : null;
  const documentCount = allFiles.filter(
    (file) => !file.is_dir && capabilitiesFor(file.path).indexable
  ).length;

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
          {documentCount.toLocaleString()}{" "}
          {documentCount === 1 ? "document" : "documents"}
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

      {/* Validation badge for structured formats only. */}
      {caps?.validator === "json" && stats?.validation && (
        <span
          className="nc-status-item"
          title={
            stats.validation.valid
              ? "Valid JSON"
              : `Invalid JSON${stats.validation.line ? ` at line ${stats.validation.line}, column ${stats.validation.column}` : ""}: ${stats.validation.message ?? ""}`
          }
          style={{
            color: stats.validation.valid ? "var(--nc-fg-faint)" : "var(--nc-danger)",
          }}
        >
          {stats.validation.valid ? <Check size={10} /> : <AlertTriangle size={10} />}
          {stats.validation.valid ? "Valid JSON" : "Invalid JSON"}
        </span>
      )}

      {active && cursor && caps?.stats !== "data" && (
        <span className="nc-status-item tabular-nums">
          Ln {cursor.line}, Col {cursor.col}
        </span>
      )}

      {/* Prose files report words and characters; data files report lines. */}
      {stats && caps?.stats === "prose" && (
        <span className="nc-status-item tabular-nums">
          {stats.words.toLocaleString()} words · {stats.chars.toLocaleString()} chars
        </span>
      )}
      {stats && caps?.stats === "data" && (
        <span className="nc-status-item tabular-nums">
          {(stats.lines ?? 0).toLocaleString()} lines · {stats.chars.toLocaleString()} chars
        </span>
      )}
      {stats && caps?.stats === "code" && (
        <span className="nc-status-item tabular-nums">
          {(stats.lines ?? 0).toLocaleString()} lines
        </span>
      )}

      <span className="nc-status-item">
        <CloudOff size={10} />
        Local only
      </span>
      <span className="nc-status-item">{caps ? `UTF-8 · ${caps.label}` : "UTF-8"}</span>
    </footer>
  );
}
