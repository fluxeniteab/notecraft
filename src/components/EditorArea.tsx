import { useMemo, useState } from "react";
import { toast } from "sonner";
import {
  AlertTriangle,
  BookOpen,
  Camera,
  Check,
  ChevronRight,
  Code2,
  Columns2,
  Copy,
  FileDown,
  FilePlus2,
  FileText,
  FolderOpen,
  FolderPlus,
  Printer,
  Keyboard,
  Loader2,
  Maximize2,
  Minimize2,
  MoreHorizontal,
  Save,
  Sparkles,
} from "lucide-react";
import { useAppStore } from "../store/appStore";
import { api } from "../lib/tauri";
import { baseName, dirName, formatRelativeTime, stem } from "../lib/files";
import { capabilitiesFor } from "../lib/documentTypes";
import { flushTab } from "../lib/saveRegistry";
import { pickVault } from "../lib/vaultActions";
import { createNote as createNoteAction } from "../lib/noteActions";
import { exportNote, type ExportFormat } from "../lib/export";
import EditorPane from "./EditorPane";
import { ContextMenu, IconButton, Kbd, Segmented, type MenuEntry } from "./ui";

const proseModes = [
  { value: "source" as const, label: <Code2 size={13} />, title: "Source — raw text" },
  { value: "live" as const, label: <Sparkles size={13} />, title: "Live — edit with rendered formatting" },
  { value: "reading" as const, label: <BookOpen size={13} />, title: "Reading — rendered note" },
  { value: "split" as const, label: <Columns2 size={13} />, title: "Split — editor and preview" },
];

const codeModes = [
  { value: "source" as const, label: <Code2 size={13} />, title: "Editor" },
  { value: "reading" as const, label: <BookOpen size={13} />, title: "Structured / rendered view" },
  { value: "split" as const, label: <Columns2 size={13} />, title: "Split — editor and view" },
];

function SaveChip({ state }: { state: "idle" | "dirty" | "saving" | "saved" | "error" }) {
  if (state === "idle") return null;
  const map = {
    dirty: { label: "Unsaved", icon: <span className="h-1.5 w-1.5 rounded-full bg-current" /> },
    saving: { label: "Saving", icon: <Loader2 size={10} className="animate-spin" /> },
    saved: { label: "Saved", icon: <Check size={11} /> },
    error: { label: "Save failed", icon: <AlertTriangle size={11} /> },
  } as const;
  const entry = map[state];
  return (
    <span className="nc-save-chip" data-state={state}>
      {entry.icon}
      {entry.label}
    </span>
  );
}

export default function EditorArea({
  onExternalReload,
}: {
  onExternalReload?: (path: string, reload: (() => void) | null) => void;
}) {
  const tabs = useAppStore((s) => s.tabs);
  const activeTabId = useAppStore((s) => s.activeTabId);
  const vaultRoot = useAppStore((s) => s.vaultRoot);
  const saveStatus = useAppStore((s) => s.saveStatus);
  const tabStats = useAppStore((s) => s.tabStats);
  const focusMode = useAppStore((s) => s.focusMode);
  const recentNotes = useAppStore((s) => s.recentNotes);
  const allFiles = useAppStore((s) => s.allFiles);
  const setTabMode = useAppStore((s) => s.setTabMode);
  const setCommandPaletteOpen = useAppStore((s) => s.setCommandPaletteOpen);
  const setQuickSwitcherOpen = useAppStore((s) => s.setQuickSwitcherOpen);
  const setFocusMode = useAppStore((s) => s.setFocusMode);
  const openTab = useAppStore((s) => s.openTab);
  const [menu, setMenu] = useState<{ x: number; y: number } | null>(null);

  const active = tabs.find((tab) => tab.id === activeTabId) ?? null;
  // "Live" formatting only makes sense for prose; JSON and code get a smaller
  // set of view modes instead of a control that does nothing.
  const modeOptions = active && capabilitiesFor(active.path).preview === "prose" ? proseModes : codeModes;
  const stats = active ? tabStats[active.id] : undefined;
  const status = active ? (saveStatus[active.id] ?? "idle") : "idle";

  const recent = useMemo(
    () =>
      recentNotes
        .map((path) => allFiles.find((file) => file.path === path))
        .filter((file): file is NonNullable<typeof file> => !!file && !file.is_dir)
        .slice(0, 4),
    [allFiles, recentNotes]
  );

  const createNote = async () => {
    await createNoteAction();
  };

  const saveNow = async () => {
    if (!active) return;
    await flushTab(active.id);
    const state = useAppStore.getState().saveStatus[active.id];
    if (state === "error") toast.error("The note could not be saved");
    else toast.success("Note saved", { description: active.path });
  };

  const snapshot = async () => {
    if (!active) return;
    try {
      await flushTab(active.id);
      await api.snapshotNow(active.path, "manual");
      toast.success("Snapshot created", { description: active.title });
    } catch (error) {
      toast.error("Could not create a snapshot", { description: String(error) });
    }
  };

  const runExport = async (format: ExportFormat) => {
    if (!active) return;
    try {
      await flushTab(active.id);
      const path = await exportNote(active.path, format);
      if (!path) return;
      if (format === "pdf") {
        toast.success("Opened for printing", {
          description: "Choose \"Save as PDF\" in the browser's print dialog.",
        });
      } else {
        toast.success("Note exported", { description: path });
      }
    } catch (error) {
      toast.error("Export failed", { description: String(error) });
    }
  };

  const copy = async (text: string, label: string) => {
    try {
      await navigator.clipboard.writeText(text);
      toast.success(label);
    } catch {
      toast.error("Clipboard is not available");
    }
  };

  const menuItems: MenuEntry[] = active
    ? [
        { label: "Save now", icon: <Save size={13} />, shortcut: "Ctrl S", onSelect: () => void saveNow() },
        { label: "Create snapshot", icon: <Camera size={13} />, onSelect: () => void snapshot() },
        { separator: true },
        {
          label: "Copy note path",
          icon: <Copy size={13} />,
          onSelect: () => void copy(active.path, "Path copied"),
        },
        {
          label: "Copy rendered HTML",
          icon: <Code2 size={13} />,
          onSelect: () =>
            void api
              .exportHtml(active.path)
              .then((html) => copy(html, "HTML copied to clipboard"))
              .catch((error) => toast.error("Could not render the note", { description: String(error) })),
        },
        { separator: true },
        {
          label: "Export Markdown…",
          icon: <FileDown size={13} />,
          onSelect: () => void runExport("markdown"),
        },
        {
          label: "Export HTML…",
          icon: <FileText size={13} />,
          onSelect: () => void runExport("html"),
        },
        {
          label: "Print / Save as PDF…",
          icon: <Printer size={13} />,
          onSelect: () => void runExport("pdf"),
        },
        {
          label: focusMode ? "Leave focus mode" : "Enter focus mode",
          icon: focusMode ? <Minimize2 size={13} /> : <Maximize2 size={13} />,
          shortcut: "Ctrl ⇧ F",
          onSelect: () => setFocusMode(!focusMode),
        },
      ]
    : [];

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      {active && (
        <div className="nc-editor-header">
          <div className="nc-breadcrumb min-w-0">
            <FileText size={13} className="shrink-0 text-fg-faint" />
            {dirName(active.path).split("/").filter(Boolean).map((part, index, parts) => (
              <span key={`${part}-${index}`} className="flex min-w-0 items-center gap-1">
                <span className="truncate">{part}</span>
                {index < parts.length - 1 && <ChevronRight size={10} className="shrink-0 opacity-50" />}
              </span>
            ))}
            {dirName(active.path) && <ChevronRight size={10} className="shrink-0 opacity-50" />}
            <span className="nc-breadcrumb-current">{baseName(active.path)}</span>
          </div>

          <SaveChip state={status} />

          <div className="flex-1" />

          {stats && active && capabilitiesFor(active.path).stats === "prose" && (
            <span className="hidden shrink-0 text-[10.5px] tabular-nums text-fg-faint md:inline">
              {stats.words.toLocaleString()} words
            </span>
          )}

          <Segmented
            value={active.mode}
            options={modeOptions}
            onChange={(mode) => setTabMode(active.id, mode)}
          />

          <div className="ml-1 flex items-center gap-0.5">
            <IconButton label="Save now" shortcut="Ctrl S" small onClick={() => void saveNow()}>
              <Save size={14} />
            </IconButton>
            <IconButton label="More actions" small onClick={(event) => {
              const rect = event.currentTarget.getBoundingClientRect();
              setMenu({ x: rect.right - 184, y: rect.bottom + 6 });
            }}>
              <MoreHorizontal size={15} />
            </IconButton>
          </div>
        </div>
      )}

      <div className="nc-editor-stage">
        {!vaultRoot ? (
          <div className="nc-page-start">
            <h1 className="nc-page-title">Welcome</h1>
            <p className="nc-page-sub">Open a folder of Markdown files, or start a new vault.</p>
            <div className="nc-page-actions">
              <button type="button" className="nc-btn nc-btn-primary" onClick={() => void pickVault("create")}>
                <FolderPlus size={14} /> Create vault
              </button>
              <button type="button" className="nc-btn nc-btn-outline" onClick={() => void pickVault("open")}>
                <FolderOpen size={14} /> Open folder
              </button>
            </div>
            <div className="nc-page-section">Get started</div>
            <div className="nc-page-list">
              <button type="button" className="nc-page-row" onClick={() => void createNote()}>
                <span className="nc-page-row-icon"><FilePlus2 size={12} /></span>
                Create your first note
                <span className="nc-page-row-meta"><Kbd>Ctrl</Kbd> <Kbd>N</Kbd></span>
              </button>
              <button type="button" className="nc-page-row" onClick={() => setQuickSwitcherOpen(true)}>
                <span className="nc-page-row-icon"><FileText size={12} /></span>
                Jump to any note
                <span className="nc-page-row-meta"><Kbd>Ctrl</Kbd> <Kbd>P</Kbd></span>
              </button>
              <button type="button" className="nc-page-row" onClick={() => setCommandPaletteOpen(true)}>
                <span className="nc-page-row-icon"><Keyboard size={12} /></span>
                Run a command
                <span className="nc-page-row-meta"><Kbd>Ctrl</Kbd> <Kbd>K</Kbd></span>
              </button>
            </div>
          </div>
        ) : active ? (
          <>
            {tabs.map((tab) => (
              <EditorPane
                key={tab.id}
                tab={tab}
                active={tab.id === activeTabId}
                onExternalReload={onExternalReload}
              />
            ))}
          </>
        ) : (
          <div className="nc-page-start">
            <h1 className="nc-page-title">{baseName(vaultRoot)}</h1>
            <p className="nc-page-sub">
              {recent.length > 0 ? "Pick up where you left off, or start something new." : "This vault is empty. Create your first note."}
            </p>
            <div className="nc-page-actions">
              <button type="button" className="nc-btn nc-btn-primary" onClick={() => void createNote()}>
                <FilePlus2 size={14} /> New note
              </button>
              <button type="button" className="nc-btn nc-btn-outline" onClick={() => setQuickSwitcherOpen(true)}>
                <FileText size={14} /> Open note
              </button>
              <button type="button" className="nc-btn nc-btn-ghost" onClick={() => setCommandPaletteOpen(true)}>
                <Keyboard size={14} /> Commands
              </button>
            </div>

            <div className="nc-page-section">{recent.length > 0 ? "Recent" : "Get started"}</div>
            <div className="nc-page-list">
              {recent.length > 0 ? (
                recent.map((file) => (
                  <button
                    key={file.path}
                    type="button"
                    className="nc-page-row"
                    onClick={() => openTab(file.path)}
                  >
                    <span className="nc-page-row-icon"><FileText size={12} /></span>
                    <span className="truncate">{stem(file.path)}</span>
                    <span className="truncate text-fg-faint">{dirName(file.path)}</span>
                    <span className="nc-page-row-meta">{formatRelativeTime(file.modified)}</span>
                  </button>
                ))
              ) : (
                <button type="button" className="nc-page-row" onClick={() => void createNote()}>
                  <span className="nc-page-row-icon"><FilePlus2 size={12} /></span>
                  Create your first note
                  <span className="nc-page-row-meta"><Kbd>Ctrl</Kbd> <Kbd>N</Kbd></span>
                </button>
              )}
            </div>

            <div className="nc-page-section">Shortcuts</div>
            <div className="grid max-w-md grid-cols-2 gap-x-6">
              <div className="nc-shortcut-row">
                <span><Kbd>Ctrl</Kbd> <Kbd>K</Kbd> Commands</span>
              </div>
              <div className="nc-shortcut-row">
                <span><Kbd>Ctrl</Kbd> <Kbd>P</Kbd> Quick switch</span>
              </div>
              <div className="nc-shortcut-row">
                <span><Kbd>Ctrl</Kbd> <Kbd>Shift</Kbd> <Kbd>F</Kbd> Search</span>
              </div>
              <div className="nc-shortcut-row">
                <span><Kbd>Ctrl</Kbd> <Kbd>\</Kbd> Context panel</span>
              </div>
            </div>
          </div>
        )}

      </div>

      {menu && <ContextMenu x={menu.x} y={menu.y} items={menuItems} onClose={() => setMenu(null)} />}
    </div>
  );
}
