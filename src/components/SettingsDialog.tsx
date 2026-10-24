import { useEffect, useState } from "react";
import { useAppStore } from "../store/appStore";
import { api } from "../lib/tauri";
import type { Settings } from "../lib/types";
import { X } from "lucide-react";

export default function SettingsDialog({ open, onClose }: { open: boolean; onClose: () => void }) {
  const { settings, setSettings, setTheme } = useAppStore();
  const [draft, setDraft] = useState<Settings | null>(settings);

  useEffect(() => { setDraft(settings); }, [settings, open]);

  if (!open || !draft) return null;

  const save = async () => {
    setSettings(draft);
    setTheme(draft.theme as "system" | "light" | "dark");
    await api.saveSettings(draft).catch(console.error);
    onClose();
  };

  const upd = (k: keyof Settings, v: any) => setDraft({ ...draft, [k]: v });

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40" onClick={onClose}>
      <div className="w-[600px] max-w-[90vw] max-h-[80vh] overflow-auto nc-scroll bg-bg-elev border border-border rounded-lg shadow-xl p-4" onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center mb-3">
          <h2 className="font-semibold">Settings</h2>
          <div className="flex-1" />
          <button onClick={onClose}><X size={16} /></button>
        </div>
        <div className="space-y-3 text-sm">
          <Row label="Theme">
            <select value={draft.theme} onChange={(e) => upd("theme", e.target.value)} className="bg-bg-muted rounded px-2 py-1">
              <option value="system">System</option>
              <option value="light">Light</option>
              <option value="dark">Dark</option>
            </select>
          </Row>
          <Row label="Accent color">
            <input type="color" value={draft.accent} onChange={(e) => upd("accent", e.target.value)} />
          </Row>
          <Row label="Editor font size">
            <input type="number" value={draft.editor_font_size} onChange={(e) => upd("editor_font_size", +e.target.value)} className="bg-bg-muted rounded px-2 py-1 w-20" />
          </Row>
          <Row label="Editor line height">
            <input type="number" step="0.1" value={draft.editor_line_height} onChange={(e) => upd("editor_line_height", +e.target.value)} className="bg-bg-muted rounded px-2 py-1 w-20" />
          </Row>
          <Row label="Reading width (px)">
            <input type="number" value={draft.reading_width} onChange={(e) => upd("reading_width", +e.target.value)} className="bg-bg-muted rounded px-2 py-1 w-20" />
          </Row>
          <Row label="Autosave (ms)">
            <input type="number" value={draft.autosave_ms} onChange={(e) => upd("autosave_ms", +e.target.value)} className="bg-bg-muted rounded px-2 py-1 w-20" />
          </Row>
          <Row label="Snapshot interval (min)">
            <input type="number" value={draft.snapshot_interval_min} onChange={(e) => upd("snapshot_interval_min", +e.target.value)} className="bg-bg-muted rounded px-2 py-1 w-20" />
          </Row>
          <Row label="Snapshot retention">
            <input type="number" value={draft.snapshot_retention} onChange={(e) => upd("snapshot_retention", +e.target.value)} className="bg-bg-muted rounded px-2 py-1 w-20" />
          </Row>
          <Row label="Hidden files">
            <input type="checkbox" checked={draft.hidden_files} onChange={(e) => upd("hidden_files", e.target.checked)} />
          </Row>
          <Row label="Reduced motion">
            <input type="checkbox" checked={draft.reduced_motion} onChange={(e) => upd("reduced_motion", e.target.checked)} />
          </Row>
          <Row label="High contrast">
            <input type="checkbox" checked={draft.high_contrast} onChange={(e) => upd("high_contrast", e.target.checked)} />
          </Row>
          <Row label="Telemetry opt-in">
            <input type="checkbox" checked={draft.telemetry_opt_in} onChange={(e) => upd("telemetry_opt_in", e.target.checked)} />
          </Row>
          <Row label="Semantic search">
            <input type="checkbox" checked={draft.semantic_search_enabled} onChange={(e) => upd("semantic_search_enabled", e.target.checked)} />
          </Row>
        </div>
        <div className="mt-4 flex justify-end gap-2">
          <button onClick={onClose} className="px-3 py-1.5 rounded border border-border">Cancel</button>
          <button onClick={save} className="px-3 py-1.5 rounded bg-accent text-accent-fg">Save</button>
        </div>
      </div>
    </div>
  );
}

function Row({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-3">
      <span className="text-fg-muted">{label}</span>
      {children}
    </div>
  );
}
