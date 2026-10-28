import { useCallback, useEffect, useMemo, useState } from "react";
import { toast } from "sonner";
import { Camera, ClipboardCopy, Loader2, RotateCcw, Sparkles } from "lucide-react";
import { api } from "../lib/tauri";
import { formatRelativeTime } from "../lib/files";
import { useAppStore } from "../store/appStore";
import type { Snapshot } from "../lib/types";
import { ConfirmDialog } from "./ui";

/**
 * Version history for the active note: automatic + manual snapshots, a line
 * diff against the current file, full restore, and partial recovery — select
 * lines from an old version and insert them back into the editor.
 */
export default function HistoryPanel({ activePath }: { activePath: string | null }) {
  const [snapshots, setSnapshots] = useState<Snapshot[]>([]);
  const [loading, setLoading] = useState(false);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [snapshotText, setSnapshotText] = useState<string | null>(null);
  const [currentText, setCurrentText] = useState<string>("");
  const [selectedLines, setSelectedLines] = useState<Set<number>>(new Set());
  const [confirmRestore, setConfirmRestore] = useState(false);
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    if (!activePath) return;
    setLoading(true);
    try {
      setSnapshots(await api.listSnapshots(activePath));
    } catch (error) {
      toast.error("Could not load version history", { description: String(error) });
      setSnapshots([]);
    } finally {
      setLoading(false);
    }
  }, [activePath]);

  useEffect(() => {
    setSelectedId(null);
    setSnapshotText(null);
    setSelectedLines(new Set());
    void refresh();
  }, [refresh]);

  // Fetch the selected snapshot and the current file for the diff.
  useEffect(() => {
    if (!activePath) return;
    let alive = true;
    void api.readFile(activePath).then((text) => alive && setCurrentText(text)).catch(() => {});
    if (!selectedId) {
      setSnapshotText(null);
      return () => {
        alive = false;
      };
    }
    setSnapshotText(null);
    void api
      .getSnapshot(selectedId)
      .then((text) => alive && setSnapshotText(text))
      .catch((error) => {
        if (alive) {
          setSnapshotText("");
          toast.error("Could not read that version", { description: String(error) });
        }
      });
    return () => {
      alive = false;
    };
  }, [activePath, selectedId]);

  const currentLines = useMemo(() => new Set(currentText.split("\n")), [currentText]);
  const snapshotLines = useMemo(
    () => (snapshotText === null ? [] : snapshotText.split("\n")),
    [snapshotText]
  );
  const changedCount = useMemo(
    () => snapshotLines.filter((line) => !currentLines.has(line)).length,
    [currentLines, snapshotLines]
  );

  const toggleLine = (index: number) => {
    setSelectedLines((current) => {
      const next = new Set(current);
      if (next.has(index)) next.delete(index);
      else next.add(index);
      return next;
    });
  };

  const recoverSelection = () => {
    if (!activePath || selectedLines.size === 0 || snapshotText === null) return;
    const text =
      [...selectedLines]
        .sort((a, b) => a - b)
        .map((index) => snapshotLines[index])
        .join("\n") + "\n";
    useAppStore.getState().requestInsert(activePath, text);
    setSelectedLines(new Set());
  };

  const restore = async () => {
    if (!selectedId) return;
    setBusy(true);
    try {
      await api.restoreSnapshot(selectedId);
      if (activePath) {
        setCurrentText(await api.readFile(activePath).catch(() => currentText));
      }
      toast.success("Version restored", { description: "The note now holds this version." });
    } catch (error) {
      toast.error("Could not restore that version", { description: String(error) });
    } finally {
      setBusy(false);
    }
  };

  if (!activePath) return <div className="nc-panel-empty">Open a note to see its history.</div>;

  return (
    <div>
      <div className="mb-2 flex items-center gap-1.5">
        <button
          type="button"
          className="nc-btn nc-btn-ghost !h-7 !px-2 text-[10px]"
          onClick={async () => {
            try {
              await api.snapshotNow(activePath, "manual");
              toast.success("Snapshot created");
              await refresh();
            } catch (error) {
              toast.error("Could not create a snapshot", { description: String(error) });
            }
          }}
        >
          <Camera size={11} /> Snapshot now
        </button>
        <div className="flex-1" />
        <span className="text-[9.5px] tabular-nums text-fg-faint">{snapshots.length} versions</span>
      </div>

      {loading && (
        <div className="flex items-center gap-2 px-1 py-3 text-[11px] text-fg-muted">
          <Loader2 size={12} className="animate-spin" /> Loading history…
        </div>
      )}

      {!loading && snapshots.length === 0 && (
        <div className="nc-panel-empty">
          No versions yet. Snapshots are created automatically while you write — or press
          “Snapshot now”.
        </div>
      )}

      {!loading &&
        snapshots.map((snapshot) => (
          <button
            key={snapshot.id}
            type="button"
            className="nc-hist-row"
            data-active={selectedId === snapshot.id || undefined}
            onClick={() => {
              setSelectedId((current) => (current === snapshot.id ? null : snapshot.id));
              setSelectedLines(new Set());
            }}
          >
            <span className="min-w-0 flex-1 truncate text-[11px] text-fg-secondary">
              {formatRelativeTime(snapshot.created_at)}
            </span>
            {snapshot.label && (
              <span className="nc-chip !py-0 !text-[9px]">{snapshot.label}</span>
            )}
            <span className="text-[9.5px] tabular-nums text-fg-faint">
              {Math.max(1, Math.round(snapshot.size / 1024))} KB
            </span>
          </button>
        ))}

      {selectedId && snapshotText === null && (
        <div className="flex items-center gap-2 px-1 py-3 text-[11px] text-fg-muted">
          <Loader2 size={12} className="animate-spin" /> Loading version…
        </div>
      )}

      {selectedId && snapshotText !== null && (
        <div className="mt-2">
          <div className="mb-1.5 flex items-center gap-1.5 text-[10px] text-fg-muted">
            <span>
              {changedCount} line{changedCount === 1 ? "" : "s"} differ from the current note
            </span>
            <div className="flex-1" />
            {selectedLines.size > 0 && (
              <span className="text-fg-secondary">{selectedLines.size} selected</span>
            )}
          </div>

          <div className="nc-hist-lines" role="list" aria-label="Version contents">
            {snapshotLines.map((line, index) => (
              <div
                key={`${index}-${line.slice(0, 24)}`}
                role="listitem"
                className="nc-hist-line"
                data-changed={currentLines.has(line) ? undefined : ""}
                data-selected={selectedLines.has(index) ? "" : undefined}
                onClick={() => toggleLine(index)}
              >
                <span>{index + 1}</span>
                <span>{line || " "}</span>
              </div>
            ))}
          </div>

          <div className="mt-2 flex flex-wrap items-center gap-1.5">
            <button
              type="button"
              className="nc-btn nc-btn-ghost !h-7 !px-2 text-[10px]"
              disabled={selectedLines.size === 0}
              onClick={recoverSelection}
              title="Insert the selected lines into the editor at the cursor"
            >
              <Sparkles size={11} /> Recover {selectedLines.size || ""} selected
            </button>
            <button
              type="button"
              className="nc-btn nc-btn-ghost !h-7 !px-2 text-[10px]"
              onClick={async () => {
                try {
                  await navigator.clipboard.writeText(snapshotText);
                  toast.success("Version copied to clipboard");
                } catch {
                  toast.error("Clipboard is not available");
                }
              }}
            >
              <ClipboardCopy size={11} /> Copy
            </button>
            <div className="flex-1" />
            <button
              type="button"
              className="nc-btn nc-btn-outline !h-7 !px-2 text-[10px]"
              onClick={() => setConfirmRestore(true)}
            >
              <RotateCcw size={11} /> Restore
            </button>
          </div>
        </div>
      )}

      <ConfirmDialog
        open={confirmRestore}
        title="Restore this version?"
        danger={false}
        message={
          <>
            The note will be replaced with the selected version. Your current text is kept as a
            snapshot first, so you can switch back.
          </>
        }
        confirmLabel="Restore version"
        onClose={() => setConfirmRestore(false)}
        onConfirm={async () => {
          if (activePath && !busy) {
            try {
              await api.snapshotNow(activePath, "pre-restore");
            } catch {
              /* restore still proceeds; the note's own history remains */
            }
          }
          await restore();
          await refresh();
        }}
      />
    </div>
  );
}
