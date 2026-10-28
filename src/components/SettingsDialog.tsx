import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { toast } from "sonner";
import {
  Check,
  Download,
  FileCog,
  Info,
  Loader2,
  Moon,
  Palette,
  RotateCcw,
  Sun,
  Type,
  Upload,
  X,
} from "lucide-react";
import { useAppStore } from "../store/appStore";
import { applyTheme, normalizeTheme } from "../lib/theme";
import { persistSettings } from "../lib/settingsActions";
import { api } from "../lib/tauri";
import type { Settings, SyncFileStatus, Theme } from "../lib/types";
import { Switch, Kbd, Modal } from "./ui";
import { Puzzle, RefreshCw } from "lucide-react";

const accents = ["#8b5cf6", "#6366f1", "#3b82f6", "#06b6d4", "#10b981", "#f59e0b", "#f43f5e"];

type Section = "appearance" | "editor" | "files" | "plugins" | "about";

const sections: Array<{ id: Section; label: string; icon: React.ReactNode }> = [
  { id: "appearance", label: "Appearance", icon: <Palette size={14} /> },
  { id: "editor", label: "Editor", icon: <Type size={14} /> },
  { id: "files", label: "Files & history", icon: <FileCog size={14} /> },
  { id: "plugins", label: "Plugins", icon: <Puzzle size={14} /> },
  { id: "about", label: "About", icon: <Info size={14} /> },
];

const clamp = (value: number, min: number, max: number) =>
  Math.min(max, Math.max(min, value));

function numberOr(value: unknown, fallback: number): number {
  return typeof value === "number" && Number.isFinite(value) ? value : fallback;
}

function coerceSettings(base: Settings, raw: unknown): Settings {
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) {
    throw new Error("This file does not contain a NoteCraft settings object.");
  }
  const data = raw as Record<string, unknown>;
  const bool = (key: keyof Settings, fallback: boolean) =>
    typeof data[key] === "boolean" ? (data[key] as boolean) : fallback;
  const accent =
    typeof data.accent === "string" && /^#[0-9a-f]{6}$/i.test(data.accent)
      ? data.accent
      : base.accent;
  return {
    theme: normalizeTheme(data.theme),
    accent,
    editor_font_size: clamp(numberOr(data.editor_font_size, base.editor_font_size), 12, 20),
    editor_line_height: clamp(
      numberOr(data.editor_line_height, base.editor_line_height),
      1.4,
      2.1
    ),
    editor_font_family:
      typeof data.editor_font_family === "string"
        ? data.editor_font_family.slice(0, 160)
        : base.editor_font_family,
    reading_width: clamp(numberOr(data.reading_width, base.reading_width), 560, 1000),
    autosave_ms: clamp(numberOr(data.autosave_ms, base.autosave_ms), 200, 5000),
    snapshot_interval_min: clamp(
      numberOr(data.snapshot_interval_min, base.snapshot_interval_min),
      1,
      120
    ),
    snapshot_retention: clamp(
      numberOr(data.snapshot_retention, base.snapshot_retention),
      1,
      500
    ),
    excluded_folders: Array.isArray(data.excluded_folders)
      ? data.excluded_folders
          .filter((entry): entry is string => typeof entry === "string")
          .map((entry) => entry.trim())
          .filter(Boolean)
          .slice(0, 100)
      : base.excluded_folders,
    hidden_files: bool("hidden_files", base.hidden_files),
    reduced_motion: bool("reduced_motion", base.reduced_motion),
    high_contrast: bool("high_contrast", base.high_contrast),
    telemetry_opt_in: false,
    semantic_search_enabled: false,
    daily_note_folder: typeof data.daily_note_folder === "string" ? data.daily_note_folder.slice(0, 160) : base.daily_note_folder,
    daily_note_template: typeof data.daily_note_template === "string" ? data.daily_note_template.slice(0, 240) : base.daily_note_template,
    capture_note:
      typeof data.capture_note === "string" && data.capture_note.trim()
        ? data.capture_note.slice(0, 240)
        : base.capture_note,
    explorer_sort:
      data.explorer_sort === "modified" || data.explorer_sort === "size" || data.explorer_sort === "name"
        ? data.explorer_sort
        : base.explorer_sort,
    explorer_group_dirs: bool("explorer_group_dirs", base.explorer_group_dirs),
    shortcuts:
      data.shortcuts && typeof data.shortcuts === "object" && !Array.isArray(data.shortcuts)
        ? (Object.fromEntries(
            Object.entries(data.shortcuts as Record<string, unknown>).filter(
              (entry): entry is [string, string] => typeof entry[1] === "string"
            )
          ) as Record<string, string>)
        : base.shortcuts,
    writing_goal_words: clamp(numberOr(data.writing_goal_words, base.writing_goal_words), 0, 100000),
    sync_folder: typeof data.sync_folder === "string" ? data.sync_folder : base.sync_folder ?? null,
    data_dir: typeof data.data_dir === "string" ? data.data_dir : base.data_dir,
  };
}

function applyDraft(theme: Theme, draft: Settings) {
  applyTheme(theme, {
    accent: draft.accent,
    reducedMotion: draft.reduced_motion,
    highContrast: draft.high_contrast,
    editorFontSize: draft.editor_font_size,
    editorLineHeight: draft.editor_line_height,
    readingWidth: draft.reading_width,
  });
}

export default function SettingsDialog({ open, onClose }: { open: boolean; onClose: () => void }) {
  const settings = useAppStore((s) => s.settings);
  const vaultRoot = useAppStore((s) => s.vaultRoot);
  const [allTemplates, setAllTemplates] = useState<Array<{ path: string; name: string }>>([]);
  const [section, setSection] = useState<Section>("appearance");
  const [draft, setDraft] = useState<Settings | null>(null);
  const [saving, setSaving] = useState(false);
  const [importError, setImportError] = useState<string | null>(null);
  const [excludedText, setExcludedText] = useState("");
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [pluginList, setPluginList] = useState<
    Array<{ id: string; name: string; version: string; description: string; enabled: boolean; permissions: string[] }>
  >([]);
  const [catalog, setCatalog] = useState<
    Array<{ id: string; name: string; version: string; description: string; installed: boolean }>
  >([]);
  const [syncStatuses, setSyncStatuses] = useState<SyncFileStatus[] | null>(null);
  const [syncBusy, setSyncBusy] = useState<"push" | "pull" | "refresh" | null>(null);

  const loadPlugins = useCallback(() => {
    void api.listPlugins().then(setPluginList).catch(() => setPluginList([]));
    void api.pluginCatalog().then(setCatalog).catch(() => setCatalog([]));
  }, []);

  const refreshSync = useCallback(async () => {
    try {
      setSyncStatuses(await api.syncStatusFolder());
    } catch {
      setSyncStatuses(null);
    }
  }, []);

  useEffect(() => {
    if (!open) return;
    loadPlugins();
    void refreshSync();
  }, [open, loadPlugins, refreshSync]);

  useEffect(() => {
    if (!open || !vaultRoot) {
      setAllTemplates([]);
      return;
    }
    void api.listTemplates().then(setAllTemplates).catch(() => setAllTemplates([]));
  }, [open, vaultRoot]);

  useEffect(() => {
    if (!open) return;
    setImportError(null);
    setSaving(false);
    if (settings) {
      setDraft({ ...settings, theme: normalizeTheme(settings.theme) });
      setExcludedText(settings.excluded_folders.join("\n"));
    }
  }, [open, settings]);

  // Live preview: appearance changes apply instantly while the dialog is open.
  useEffect(() => {
    if (!open || !draft) return;
    applyDraft(normalizeTheme(draft.theme), draft);
  }, [open, draft]);

  const revert = () => {
    const store = useAppStore.getState();
    const base = store.settings;
    applyTheme(store.theme, {
      accent: base?.accent,
      reducedMotion: base?.reduced_motion,
      highContrast: base?.high_contrast,
      editorFontSize: base?.editor_font_size,
      editorLineHeight: base?.editor_line_height,
      readingWidth: base?.reading_width,
    });
  };

  const close = () => {
    revert();
    onClose();
  };

  const update = <K extends keyof Settings>(key: K, value: Settings[K]) => {
    setDraft((current) => (current ? { ...current, [key]: value } : current));
  };

  const save = async () => {
    if (!draft || saving) return;
    setSaving(true);
    const excluded = excludedText
      .split(/\r?\n|,/)
      .map((entry) => entry.trim())
      .filter((entry) => entry && !entry.includes(".."))
      .slice(0, 100);
    const next: Settings = {
      ...draft,
      theme: normalizeTheme(draft.theme),
      excluded_folders: excluded,
      editor_font_size: clamp(draft.editor_font_size, 12, 20),
      editor_line_height: clamp(draft.editor_line_height, 1.4, 2.1),
      reading_width: clamp(draft.reading_width, 560, 1000),
      autosave_ms: clamp(draft.autosave_ms, 200, 5000),
      snapshot_interval_min: clamp(draft.snapshot_interval_min, 1, 120),
      snapshot_retention: clamp(draft.snapshot_retention, 1, 500),
      daily_note_folder: draft.daily_note_folder.trim().replace(/[/\\]+/g, "/").replace(/^[/\\]+|[/\\]+$/g, "") || "daily",
      daily_note_template: draft.daily_note_template?.trim() || null,
      capture_note:
        draft.capture_note.trim().replace(/[/\\]+/g, "/").replace(/^[/\\]+|[/\\]+$/g, "") ||
        "inbox.md",
      explorer_sort: draft.explorer_sort === "modified" || draft.explorer_sort === "size" ? draft.explorer_sort : "name",
      writing_goal_words: clamp(Math.round(draft.writing_goal_words), 0, 100000),
    };
    useAppStore.getState().setTheme(next.theme);
    await persistSettings(next);
    setSaving(false);
    onClose();
    toast.success("Settings saved");
  };

  const exportSettings = () => {
    if (!draft) return;
    const blob = new Blob([JSON.stringify(draft, null, 2)], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = "notecraft-settings.json";
    anchor.click();
    URL.revokeObjectURL(url);
  };

  const importSettings = async (file: File) => {
    if (!draft) return;
    try {
      const parsed: unknown = JSON.parse(await file.text());
      const next = coerceSettings(draft, parsed);
      setDraft(next);
      setExcludedText(next.excluded_folders.join("\n"));
      setImportError(null);
      toast.success("Settings loaded", { description: "Review the values, then save." });
    } catch (error) {
      setImportError(String(error).replace(/^Error:\s*/, ""));
    }
  };

  const themePreviews = useMemo(
    () => [
      {
        id: "dark" as const,
        name: "Midnight",
        description: "Deep black, soft violet",
        swatch: ["#09090b", "#141419", "#8b5cf6"],
      },
      {
        id: "light" as const,
        name: "Paper",
        description: "Clean white, warm ink",
        swatch: ["#f7f7f8", "#ffffff", "#7c3aed"],
      },
    ],
    []
  );

  return (
    <Modal open={open} onOpenChange={(next) => !next && close()} className="nc-settings" labelledBy="nc-settings-title">
      <div className="nc-modal-head">
        <Dialog.Title id="nc-settings-title" className="nc-modal-title">
          Settings
        </Dialog.Title>
        <div className="flex-1" />
        <Dialog.Close asChild>
          <button type="button" className="nc-icon-btn" aria-label="Close settings" onClick={close}>
            <X size={15} />
          </button>
        </Dialog.Close>
      </div>

      {!draft ? (
        <div className="grid flex-1 place-items-center text-[12px] text-fg-muted">
          <span className="flex items-center gap-2">
            <Loader2 size={14} className="animate-spin" /> Loading settings…
          </span>
        </div>
      ) : (
        <>
          <div className="nc-settings-layout">
            <nav className="nc-settings-nav" aria-label="Settings sections">
              {sections.map((entry) => (
                <button
                  key={entry.id}
                  type="button"
                  className="nc-settings-nav-btn"
                  data-active={section === entry.id}
                  onClick={() => setSection(entry.id)}
                >
                  {entry.icon}
                  <span>{entry.label}</span>
                </button>
              ))}
            </nav>

            <div className="nc-settings-content nc-scroll">
              {section === "appearance" && (
                <>
                  <div className="nc-setting-group">
                    <div className="nc-setting-group-title">Theme</div>
                    <div className="nc-setting-hint mb-2.5">
                      NoteCraft has two hand-tuned themes. It never follows the operating
                      system automatically — you stay in control.
                    </div>
                    <div className="nc-theme-cards">
                      {themePreviews.map((preview) => (
                        <button
                          key={preview.id}
                          type="button"
                          className="nc-theme-card"
                          data-active={normalizeTheme(draft.theme) === preview.id}
                          onClick={() => update("theme", preview.id)}
                        >
                          <div
                            className="nc-theme-swatch"
                            style={{ background: preview.swatch[0], borderColor: preview.swatch[2] + "33" }}
                          >
                            <span style={{ background: preview.swatch[1], flex: 2 }} />
                            <span style={{ background: preview.swatch[2], flex: 3, opacity: 0.7 }} />
                            <span style={{ background: preview.swatch[0], flex: 4 }} />
                          </div>
                          <div className="nc-theme-name">{preview.name}</div>
                          <div className="text-[10px] text-fg-muted">{preview.description}</div>
                          {normalizeTheme(draft.theme) === preview.id && (
                            <span className="nc-theme-check">
                              <Check size={11} strokeWidth={3} />
                            </span>
                          )}
                        </button>
                      ))}
                    </div>
                  </div>

                  <div className="nc-setting-group">
                    <div className="nc-setting-group-title">Accent</div>
                    <div className="flex items-center justify-between gap-4">
                      <div className="nc-accent-row">
                        {accents.map((color) => (
                          <button
                            key={color}
                            type="button"
                            className="nc-accent-dot"
                            style={{ background: color }}
                            data-active={draft.accent.toLowerCase() === color}
                            aria-label={`Use ${color} accent`}
                            onClick={() => update("accent", color)}
                          />
                        ))}
                      </div>
                      <label className="flex items-center gap-2 text-[11px] text-fg-muted">
                        Custom
                        <input
                          type="color"
                          value={/^#[0-9a-f]{6}$/i.test(draft.accent) ? draft.accent : "#8b5cf6"}
                          onChange={(event) => update("accent", event.target.value)}
                          className="h-7 w-9 cursor-pointer rounded border border-border bg-transparent p-0.5"
                          aria-label="Custom accent color"
                        />
                      </label>
                    </div>
                  </div>

                  <div className="nc-setting-group">
                    <div className="nc-setting-group-title">Comfort</div>
                    <div className="nc-setting-row">
                      <div>
                        <div className="nc-setting-label">Reduced motion</div>
                        <div className="nc-setting-hint">Remove transitions and animated panels.</div>
                      </div>
                      <div className="nc-setting-control">
                        <Switch
                          checked={draft.reduced_motion}
                          onChange={(value) => update("reduced_motion", value)}
                          label="Reduced motion"
                        />
                      </div>
                    </div>
                    <div className="nc-setting-row">
                      <div>
                        <div className="nc-setting-label">High contrast</div>
                        <div className="nc-setting-hint">Stronger borders and brighter secondary text.</div>
                      </div>
                      <div className="nc-setting-control">
                        <Switch
                          checked={draft.high_contrast}
                          onChange={(value) => update("high_contrast", value)}
                          label="High contrast"
                        />
                      </div>
                    </div>
                  </div>

                  <div className="nc-setting-group">
                    <div className="nc-setting-group-title">Preview</div>
                    <div className="rounded-lg border border-border bg-surface p-4">
                      <div
                        className="nc-prose"
                        style={{
                          fontSize: `${draft.editor_font_size}px`,
                          lineHeight: draft.editor_line_height,
                          maxWidth: "100%",
                        }}
                      >
                        <h2 style={{ marginTop: 0 }}>Quiet tools, clear thoughts</h2>
                        <p>
                          A note is a place to think. Write in Markdown, link with{" "}
                          <code>[[brackets]]</code>, and let the structure emerge.
                        </p>
                        <p style={{ marginBottom: 0 }}>
                          <a className="nc-wikilink" href="#">A linked idea</a>
                          <span className="ml-2 text-[10px] text-fg-faint">#writing</span>
                        </p>
                      </div>
                    </div>
                  </div>
                </>
              )}

              {section === "editor" && (
                <>
                  <div className="nc-setting-group">
                    <div className="nc-setting-group-title">Typography</div>
                    <div className="nc-setting-row">
                      <div>
                        <div className="nc-setting-label">Font size</div>
                        <div className="nc-setting-hint">Applies to editing and reading.</div>
                      </div>
                      <div className="nc-setting-control">
                        <input
                          type="range"
                          min={12}
                          max={20}
                          step={0.5}
                          value={draft.editor_font_size}
                          onChange={(event) => update("editor_font_size", Number(event.target.value))}
                          className="nc-range"
                        />
                        <input
                          type="number"
                          min={12}
                          max={20}
                          step={0.5}
                          value={draft.editor_font_size}
                          onChange={(event) =>
                            update("editor_font_size", clamp(Number(event.target.value) || 15, 12, 20))
                          }
                          className="nc-number"
                          aria-label="Font size in pixels"
                        />
                      </div>
                    </div>
                    <div className="nc-setting-row">
                      <div>
                        <div className="nc-setting-label">Line height</div>
                        <div className="nc-setting-hint">Breathing room between lines.</div>
                      </div>
                      <div className="nc-setting-control">
                        <input
                          type="range"
                          min={1.4}
                          max={2.1}
                          step={0.05}
                          value={draft.editor_line_height}
                          onChange={(event) => update("editor_line_height", Number(event.target.value))}
                          className="nc-range"
                        />
                        <span className="w-[42px] text-right text-[11px] tabular-nums text-fg-secondary">
                          {draft.editor_line_height.toFixed(2)}
                        </span>
                      </div>
                    </div>
                    <div className="nc-setting-row">
                      <div>
                        <div className="nc-setting-label">Reading width</div>
                        <div className="nc-setting-hint">Maximum measure for prose in pixels.</div>
                      </div>
                      <div className="nc-setting-control">
                        <input
                          type="range"
                          min={560}
                          max={1000}
                          step={20}
                          value={draft.reading_width}
                          onChange={(event) => update("reading_width", Number(event.target.value))}
                          className="nc-range"
                        />
                        <span className="w-[52px] text-right text-[11px] tabular-nums text-fg-secondary">
                          {draft.reading_width}px
                        </span>
                      </div>
                    </div>
                  </div>

                  <div className="nc-setting-group">
                    <div className="nc-setting-group-title">Saving</div>
                    <div className="nc-setting-row">
                      <div>
                        <div className="nc-setting-label">Autosave delay</div>
                        <div className="nc-setting-hint">
                          Notes save automatically after you stop typing.
                        </div>
                      </div>
                      <div className="nc-setting-control">
                        <select
                          value={draft.autosave_ms}
                          onChange={(event) => update("autosave_ms", Number(event.target.value))}
                          className="nc-select"
                        >
                          <option value={400}>0.4 seconds</option>
                          <option value={800}>0.8 seconds</option>
                          <option value={1200}>1.2 seconds</option>
                          <option value={2000}>2 seconds</option>
                        </select>
                      </div>
                    </div>
                    <div className="nc-setting-row">
                      <div>
                        <div className="nc-setting-label">Editor modes</div>
                        <div className="nc-setting-hint">
                          Source, live preview, reading and split are available from the editor
                          toolbar.
                        </div>
                      </div>
                      <div className="nc-setting-control text-[10.5px] text-fg-faint">
                        <Kbd>Ctrl</Kbd> <Kbd>S</Kbd> saves immediately
                      </div>
                    </div>
                  </div>
                </>
              )}

              {section === "files" && (
                <>
                  <div className="nc-setting-group">
                    <div className="nc-setting-group-title">Daily notes</div>
                    <div className="nc-setting-row">
                      <div>
                        <div className="nc-setting-label">Daily note folder</div>
                        <div className="nc-setting-hint">Vault-relative folder used for YYYY-MM-DD notes.</div>
                      </div>
                      <div className="nc-setting-control">
                        <input
                          className="nc-input !h-8 !w-40"
                          value={draft.daily_note_folder}
                          onChange={(event) => update("daily_note_folder", event.target.value.replace(/[/\\]+/g, "/").replace(/^[/\\]+|[/\\]+$/g, ""))}
                          aria-label="Daily note folder"
                        />
                      </div>
                    </div>
                    <div className="nc-setting-row">
                      <div>
                        <div className="nc-setting-label">Daily note template</div>
                        <div className="nc-setting-hint">Optional Markdown template path, such as templates/daily.md.</div>
                      </div>
                      <div className="nc-setting-control">
                        <select className="nc-select max-w-48" value={draft.daily_note_template ?? ""} onChange={(event) => update("daily_note_template", event.target.value || null)} aria-label="Daily note template">
                          <option value="">Default heading</option>
                          {allTemplates.map((template) => <option key={template.path} value={template.path}>{template.path}</option>)}
                        </select>
                      </div>
                    </div>
                    <div className="nc-setting-group-title">Quick capture</div>
                    <div className="nc-setting-row">
                      <div>
                        <div className="nc-setting-label">Capture note</div>
                        <div className="nc-setting-hint">Quick capture (Ctrl ⇧ N) appends bullets to this note.</div>
                      </div>
                      <div className="nc-setting-control">
                        <input
                          className="nc-input !h-8 !w-40"
                          value={draft.capture_note}
                          onChange={(event) => update("capture_note", event.target.value)}
                          aria-label="Capture note path"
                        />
                      </div>
                    </div>
                    <div className="nc-setting-group-title">Explorer</div>
                    <div className="nc-setting-row">
                      <div>
                        <div className="nc-setting-label">Sort files by</div>
                        <div className="nc-setting-hint">Applies to the file tree.</div>
                      </div>
                      <div className="nc-setting-control">
                        <select
                          className="nc-select max-w-40"
                          value={draft.explorer_sort}
                          onChange={(event) => update("explorer_sort", event.target.value as Settings["explorer_sort"])}
                          aria-label="Explorer sort order"
                        >
                          <option value="name">Name</option>
                          <option value="modified">Modified</option>
                          <option value="size">Size</option>
                        </select>
                      </div>
                    </div>
                    <div className="nc-setting-row">
                      <div>
                        <div className="nc-setting-label">Folders first</div>
                        <div className="nc-setting-hint">Group folders above files when sorting.</div>
                      </div>
                      <div className="nc-setting-control">
                        <Switch
                          checked={draft.explorer_group_dirs}
                          onChange={(value) => update("explorer_group_dirs", value)}
                          label="Folders first"
                        />
                      </div>
                    </div>
                    <div className="nc-setting-group-title">Vault files</div>
                    <div className="nc-setting-row">
                      <div>
                        <div className="nc-setting-label">Show hidden files</div>
                        <div className="nc-setting-hint">
                          Include files and folders that start with a dot in the explorer.
                        </div>
                      </div>
                      <div className="nc-setting-control">
                        <Switch
                          checked={draft.hidden_files}
                          onChange={(value) => update("hidden_files", value)}
                          label="Show hidden files"
                        />
                      </div>
                    </div>
                    <div className="py-3">
                      <div className="nc-setting-label mb-1.5">Excluded folders</div>
                      <div className="nc-setting-hint mb-2">
                        One folder name per line. These are skipped by the explorer, search and
                        graph. Changes apply after reopening the vault.
                      </div>
                      <textarea
                        value={excludedText}
                        onChange={(event) => setExcludedText(event.target.value)}
                        rows={4}
                        placeholder={".trash\nnode_modules\nattachments/private"}
                        className="nc-input h-auto resize-y py-2 font-mono text-[11px]"
                      />
                    </div>
                  </div>

                  <div className="nc-setting-group">
                    <div className="nc-setting-group-title">Sync folder (optional)</div>
                    <div className="nc-setting-row">
                      <div>
                        <div className="nc-setting-label">Mirror this vault to a folder</div>
                        <div className="nc-setting-hint">
                          A local folder you control — a mounted drive or a Dropbox/Syncthing
                          folder. Conflicts are reported, never overwritten.
                        </div>
                      </div>
                      <div className="nc-setting-control">
                        <div className="flex items-center gap-1.5">
                          <input
                            className="nc-input !h-8 !w-48 font-mono !text-[10.5px]"
                            value={draft.sync_folder ?? ""}
                            placeholder="Not configured"
                            readOnly
                            aria-label="Sync folder path"
                          />
                          <button
                            type="button"
                            className="nc-btn nc-btn-outline !h-8 !px-2 text-[10px]"
                            onClick={async () => {
                              try {
                                const { open: pickDirectory } = await import("@tauri-apps/plugin-dialog");
                                const picked = await pickDirectory({ directory: true, multiple: false });
                                const folder = Array.isArray(picked) ? picked[0] : picked;
                                if (!folder) return;
                                const next = { ...draft, sync_folder: folder };
                                setDraft(next);
                                await persistSettings(next);
                                await refreshSync();
                                toast.success("Sync folder configured", { description: folder });
                              } catch (error) {
                                toast.error("Could not choose a folder", { description: String(error) });
                              }
                            }}
                          >
                            Choose…
                          </button>
                        </div>
                      </div>
                    </div>
                    {syncStatuses && (
                      <div className="nc-setting-row">
                        <div>
                          <div className="nc-setting-label">
                            {syncStatuses.filter((s) => s.status === "in_sync").length} in sync
                            {syncStatuses.filter((s) => s.status === "local_only" || s.status === "local_newer").length > 0 &&
                              ` · ${syncStatuses.filter((s) => s.status === "local_only" || s.status === "local_newer").length} to push`}
                            {syncStatuses.filter((s) => s.status === "remote_only" || s.status === "remote_newer").length > 0 &&
                              ` · ${syncStatuses.filter((s) => s.status === "remote_only" || s.status === "remote_newer").length} to pull`}
                            {syncStatuses.filter((s) => s.status === "conflict").length > 0 &&
                              ` · ${syncStatuses.filter((s) => s.status === "conflict").length} conflicts`}
                          </div>
                          <div className="nc-setting-hint">Markdown files compared with the sync folder.</div>
                        </div>
                        <div className="nc-setting-control flex items-center gap-1.5">
                          <button
                            type="button"
                            className="nc-btn nc-btn-ghost !h-7 !px-2 text-[10px]"
                            disabled={syncBusy !== null}
                            onClick={async () => {
                              setSyncBusy("refresh");
                              await refreshSync();
                              setSyncBusy(null);
                            }}
                          >
                            <RefreshCw size={11} className={syncBusy === "refresh" ? "animate-spin" : undefined} /> Refresh
                          </button>
                          <button
                            type="button"
                            className="nc-btn nc-btn-outline !h-7 !px-2 text-[10px]"
                            disabled={syncBusy !== null}
                            onClick={async () => {
                              setSyncBusy("push");
                              try {
                                const result = await api.syncPush();
                                const conflicts = result.filter((s) => s.status === "conflict").length;
                                toast.success("Push complete", {
                                  description: conflicts ? `${conflicts} conflict(s) left untouched.` : "Vault copied to the sync folder.",
                                });
                                setSyncStatuses(result);
                              } catch (error) {
                                toast.error("Push failed", { description: String(error) });
                              } finally {
                                setSyncBusy(null);
                              }
                            }}
                          >
                            <Upload size={11} /> Push
                          </button>
                          <button
                            type="button"
                            className="nc-btn nc-btn-outline !h-7 !px-2 text-[10px]"
                            disabled={syncBusy !== null}
                            onClick={async () => {
                              setSyncBusy("pull");
                              try {
                                const result = await api.syncPull();
                                const conflicts = result.filter((s) => s.status === "conflict").length;
                                toast.success("Pull complete", {
                                  description: conflicts ? `${conflicts} conflict(s) left untouched.` : "Sync folder merged into the vault.",
                                });
                                setSyncStatuses(result);
                              } catch (error) {
                                toast.error("Pull failed", { description: String(error) });
                              } finally {
                                setSyncBusy(null);
                              }
                            }}
                          >
                            <Download size={11} /> Pull
                          </button>
                        </div>
                      </div>
                    )}
                  </div>

                  <div className="nc-setting-group">
                    <div className="nc-setting-group-title">History</div>
                    <div className="nc-setting-row">
                      <div>
                        <div className="nc-setting-label">Snapshot interval</div>
                        <div className="nc-setting-hint">
                          Minutes between automatic snapshots of edited notes.
                        </div>
                      </div>
                      <div className="nc-setting-control">
                        <input
                          type="number"
                          min={1}
                          max={120}
                          value={draft.snapshot_interval_min}
                          onChange={(event) =>
                            update(
                              "snapshot_interval_min",
                              clamp(Number(event.target.value) || 5, 1, 120)
                            )
                          }
                          className="nc-number"
                          aria-label="Snapshot interval in minutes"
                        />
                      </div>
                    </div>
                    <div className="nc-setting-row">
                      <div>
                        <div className="nc-setting-label">Snapshots kept per note</div>
                        <div className="nc-setting-hint">Older versions are pruned automatically.</div>
                      </div>
                      <div className="nc-setting-control">
                        <input
                          type="number"
                          min={1}
                          max={500}
                          value={draft.snapshot_retention}
                          onChange={(event) =>
                            update(
                              "snapshot_retention",
                              clamp(Number(event.target.value) || 50, 1, 500)
                            )
                          }
                          className="nc-number"
                          aria-label="Snapshot retention"
                        />
                      </div>
                    </div>
                  </div>
                </>
              )}

              {section === "plugins" && (
                <>
                  <div className="nc-setting-group">
                    <div className="nc-setting-group-title">Installed plugins</div>
                    {pluginList.length === 0 ? (
                      <div className="nc-setting-hint">No plugins installed yet — pick one from the catalog below.</div>
                    ) : (
                      pluginList.map((plugin) => (
                        <div key={plugin.id} className="nc-setting-row">
                          <div>
                            <div className="nc-setting-label">
                              {plugin.name} <span className="text-fg-faint">v{plugin.version}</span>
                            </div>
                            <div className="nc-setting-hint">{plugin.description}</div>
                            {plugin.permissions.length > 0 && (
                              <div className="mt-1.5 flex flex-wrap gap-1">
                                {plugin.permissions.map((permission) => (
                                  <span key={permission} className="nc-chip !py-0 !text-[9px]">{permission}</span>
                                ))}
                              </div>
                            )}
                          </div>
                          <div className="nc-setting-control flex items-center gap-2">
                            <Switch
                              checked={plugin.enabled}
                              onChange={(value) => {
                                void api
                                  .enablePlugin(plugin.id, value)
                                  .then(loadPlugins)
                                  .catch((error) =>
                                    toast.error("Could not change plugin state", { description: String(error) })
                                  );
                              }}
                              label={`Enable ${plugin.name}`}
                            />
                            <button
                              type="button"
                              className="nc-btn nc-btn-ghost !h-7 !px-2 text-[10px]"
                              onClick={() => {
                                void api
                                  .uninstallPlugin(plugin.id)
                                  .then(() => {
                                    loadPlugins();
                                    toast.success("Plugin removed", { description: plugin.name });
                                  })
                                  .catch((error) =>
                                    toast.error("Could not remove plugin", { description: String(error) })
                                  );
                              }}
                            >
                              Remove
                            </button>
                          </div>
                        </div>
                      ))
                    )}
                  </div>

                  <div className="nc-setting-group">
                    <div className="nc-setting-group-title">Marketplace</div>
                    {catalog.length === 0 ? (
                      <div className="nc-setting-hint">The bundled catalog is unavailable in this build.</div>
                    ) : (
                      catalog.map((entry) => (
                        <div key={entry.id} className="nc-setting-row">
                          <div>
                            <div className="nc-setting-label">
                              {entry.name} <span className="text-fg-faint">v{entry.version}</span>
                            </div>
                            <div className="nc-setting-hint">{entry.description}</div>
                          </div>
                          <div className="nc-setting-control">
                            <button
                              type="button"
                              className="nc-btn nc-btn-outline !h-7 !px-2 text-[10px]"
                              disabled={entry.installed}
                              onClick={() => {
                                void api
                                  .installCatalogPlugin(entry.id)
                                  .then(() => {
                                    loadPlugins();
                                    toast.success("Plugin installed", { description: entry.name });
                                  })
                                  .catch((error) =>
                                    toast.error("Could not install plugin", { description: String(error) })
                                  );
                              }}
                            >
                              {entry.installed ? (
                                <>
                                  <Check size={11} /> Installed
                                </>
                              ) : (
                                "Install"
                              )}
                            </button>
                          </div>
                        </div>
                      ))
                    )}
                    <div className="nc-setting-hint mt-2">
                      Plugins declare their permissions, stay disabled until you enable them, and can
                      be removed at any time.
                    </div>
                  </div>
                </>
              )}

              {section === "about" && (
                <>
                  <div className="nc-setting-group">
                    <div className="nc-setting-group-title">NoteCraft</div>
                    <p className="text-[12px] leading-relaxed text-fg-secondary">
                      A local-first Markdown workspace. Your notes are ordinary files in an
                      ordinary folder — no account, no cloud, no lock-in.
                    </p>
                    <div className="mt-3 flex flex-wrap gap-1.5">
                      <span className="nc-chip">Version 0.1.0</span>
                      <span className="nc-chip">Tauri 2</span>
                      <span className="nc-chip">MIT licensed</span>
                    </div>
                  </div>

                  <div className="nc-setting-group">
                    <div className="nc-setting-group-title">Keyboard</div>
                    <div className="grid grid-cols-2 gap-x-6">
                      {[
                        ["Search & commands", "Ctrl K"],
                        ["Quick switch note", "Ctrl P"],
                        ["New note", "Ctrl N"],
                        ["Save now", "Ctrl S"],
                        ["Close note", "Ctrl W"],
                        ["File explorer", "Ctrl B"],
                        ["Context panel", "Ctrl \\"],
                        ["Focus mode", "Ctrl ⇧ F"],
                        ["Toggle theme", "Ctrl ⇧ D"],
                        ["Settings", "Ctrl ,"],
                      ].map(([label, shortcut]) => (
                        <div key={label} className="nc-shortcut-row">
                          <span className="text-fg-secondary">{label}</span>
                          <Kbd>{shortcut}</Kbd>
                        </div>
                      ))}
                    </div>
                  </div>

                  <div className="nc-setting-group">
                    <div className="nc-setting-group-title">Settings file</div>
                    <div className="flex flex-wrap items-center gap-2">
                      <button type="button" className="nc-btn nc-btn-outline" onClick={exportSettings}>
                        <Download size={13} /> Export settings
                      </button>
                      <button
                        type="button"
                        className="nc-btn nc-btn-outline"
                        onClick={() => fileInputRef.current?.click()}
                      >
                        <Upload size={13} /> Import settings
                      </button>
                      <input
                        ref={fileInputRef}
                        type="file"
                        accept="application/json,.json"
                        className="hidden"
                        onChange={(event) => {
                          const file = event.target.files?.[0];
                          if (file) void importSettings(file);
                          event.target.value = "";
                        }}
                      />
                      <button
                        type="button"
                        className="nc-btn nc-btn-ghost"
                        onClick={() => {
                          if (!settings) return;
                          setDraft({ ...settings });
                          setExcludedText(settings.excluded_folders.join("\n"));
                          setImportError(null);
                        }}
                      >
                        <RotateCcw size={13} /> Reset changes
                      </button>
                    </div>
                    {importError && (
                      <div className="mt-2 rounded-md border border-danger/25 bg-danger/5 px-2.5 py-2 text-[11px] text-danger">
                        Import failed: {importError}
                      </div>
                    )}
                    <div className="nc-setting-hint mt-2">
                      Imports are validated field by field. Nothing is written until you press
                      Save.
                    </div>
                  </div>
                </>
              )}
            </div>
          </div>

          <div className="nc-modal-foot">
            <span className="mr-auto flex items-center gap-1.5 text-[10.5px] text-fg-faint">
              {normalizeTheme(draft.theme) === "dark" ? <Moon size={11} /> : <Sun size={11} />}
              Changes preview live
            </span>
            <button type="button" className="nc-btn nc-btn-outline" onClick={close}>
              Cancel
            </button>
            <button type="button" className="nc-btn nc-btn-primary" onClick={() => void save()} disabled={saving}>
              {saving ? <Loader2 size={13} className="animate-spin" /> : <Check size={13} />}
              Save changes
            </button>
          </div>
        </>
      )}
    </Modal>
  );
}
