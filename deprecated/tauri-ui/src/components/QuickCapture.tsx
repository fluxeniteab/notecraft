import { useEffect, useState } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { toast } from "sonner";
import { CornerDownLeft, Inbox } from "lucide-react";
import { api } from "../lib/tauri";
import { refreshVault } from "../lib/vaultActions";
import { useAppStore } from "../store/appStore";
import { Modal, DialogCloseButton, Spinner } from "./ui";

/**
 * Quick capture: jot a thought and file it into the configured capture note
 * (Settings → Files, default `inbox.md`) without opening a tab first.
 */
export default function QuickCapture({ open, onClose }: { open: boolean; onClose: () => void }) {
  const settings = useAppStore((s) => s.settings);
  const vaultRoot = useAppStore((s) => s.vaultRoot);
  const [text, setText] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (open) {
      setText("");
      setError(null);
      setBusy(false);
    }
  }, [open]);

  const target = settings?.capture_note || "inbox.md";

  const submit = async () => {
    if (!text.trim() || busy) return;
    setBusy(true);
    setError(null);
    try {
      const rel = await api.appendCapture(text);
      await refreshVault();
      toast.success("Captured", {
        description: rel,
        action: { label: "Open", onClick: () => void useAppStore.getState().openTab(rel) },
      });
      setBusy(false);
      onClose();
    } catch (err) {
      setBusy(false);
      setError(String(err).replace(/^Error:\s*/, ""));
    }
  };

  return (
    <Modal open={open} onOpenChange={(next) => !next && onClose()} className="!w-[520px]" labelledBy="nc-capture-title">
      <div className="nc-modal-head">
        <div className="grid h-8 w-8 place-items-center rounded-lg bg-accent-soft text-accent">
          <Inbox size={16} />
        </div>
        <Dialog.Title id="nc-capture-title" className="nc-modal-title">
          Quick capture
        </Dialog.Title>
        <div className="flex-1" />
        <DialogCloseButton onClose={onClose} />
      </div>
      <div className="nc-modal-body">
        <textarea
          autoFocus
          value={text}
          rows={5}
          placeholder="Write a thought, link or task…"
          aria-label="Capture text"
          onChange={(event) => setText(event.target.value)}
          onKeyDown={(event) => {
            if ((event.ctrlKey || event.metaKey) && event.key === "Enter") void submit();
          }}
          className="nc-input h-auto resize-y py-2 text-[12.5px]"
        />
        <div className="mt-2 flex items-center gap-2 text-[10.5px] text-fg-faint">
          <span>
            Saved to <span className="text-fg-secondary">{vaultRoot ? target : "—"}</span>
          </span>
          <div className="flex-1" />
          <span className="inline-flex items-center gap-1">
            <CornerDownLeft size={10} /> Ctrl+Enter to capture
          </span>
        </div>
        {error && (
          <div className="mt-2 rounded-md border border-danger/25 bg-danger/5 px-2.5 py-2 text-[11px] text-danger">
            {error}
          </div>
        )}
      </div>
      <div className="nc-modal-foot">
        <button type="button" className="nc-btn nc-btn-outline" onClick={onClose}>
          Cancel
        </button>
        <button
          type="button"
          className="nc-btn nc-btn-primary"
          onClick={() => void submit()}
          disabled={!text.trim() || busy || !vaultRoot}
        >
          {busy ? <Spinner size={13} /> : <CornerDownLeft size={13} />} Capture
        </button>
      </div>
    </Modal>
  );
}
