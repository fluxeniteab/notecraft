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
  FilePlus2,
  FileText,
  FolderOpen,
  FolderPlus,
  Keyboard,
  Loader2,
  Maximize2,
  Minimize2,
  MoreHorizontal,
  Save,
  Sparkles,
  Clock3,
} from "lucide-react";
import { useAppStore } from "../store/appStore";
import { api } from "../lib/tauri";
import { baseName, dirName, formatRelativeTime, stem } from "../lib/files";
import { flushTab } from "../lib/saveRegistry";
import { pickVault } from "../lib/vaultActions";
import { createNote as createNoteAction } from "../lib/noteActions";
import { exportNotePdf } from "../lib/export";
import EditorPane from "./EditorPane";
import { ContextMenu, IconButton, Kbd, Segmented, type MenuEntry } from "./ui";

const modeOptions = [
  { value: "source" as const, label: <Code2 size={13} />, title: "Source — raw Markdown" },
  { value: "live" as const, label: <Sparkles size={13} />, title: "Live — edit with rendered formatting" },
  { value: "reading" as const, label: <BookOpen size={13} />, title: "Reading — rendered note" },
  { value: "split" as const, label: <Columns2 size={13} />, title: "Split — editor and preview" },
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
        {
          label: "Export PDF…",
          icon: <FileText size={13} />,
          onSelect: () => {
            void exportNotePdf(active.path)
              .then(() => toast.success("Print dialog opened", { description: "Choose \"Save as PDF\" to keep a copy." }))
              .catch((error) => toast.error("Could not open the print view", { description: String(error) }));
          },
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

          {stats && (
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
          <div className="nc-welcome">
            <div className="nc-welcome-inner">
              <div className="nc-welcome-mark">
                <Sparkles size={24} />
              </div>
              <h1 className="nc-welcome-title">
                Your notes, <em>beautifully</em> local.
              </h1>
              <p className="nc-welcome-sub">
                NoteCraft is a calm, local-first writing space. Open a folder of Markdown
                files or start a new vault — everything stays on your machine.
              </p>
              <div className="nc-welcome-actions">
                <button type="button" className="nc-btn nc-btn-primary" onClick={() => void pickVault("open")}>
                  <FolderOpen size={14} /> Open a vault
                </button>
                <button type="button" className="nc-btn nc-btn-outline" onClick={() => void pickVault("create")}>
                  <FolderPlus size={14} /> Create a vault
                </button>
              </div>
              <div className="nc-welcome-grid">
                <div className="nc-welcome-card">
                  <div className="nc-welcome-card-label">Local-first</div>
                  <div className="nc-welcome-card-title">Plain Markdown on disk</div>
                  <div className="nc-welcome-card-sub">Portable, greppable, yours forever.</div>
                </div>
                <div className="nc-welcome-card">
                  <div className="nc-welcome-card-label">Connections</div>
                  <div className="nc-welcome-card-title">Links, tags and graph</div>
                  <div className="nc-welcome-card-sub">Find ideas by how they relate.</div>
                </div>
              </div>
              <div className="mt-6 max-w-sm">
                <div className="nc-shortcut-row">
                  <span><Kbd>Ctrl</Kbd> <Kbd>K</Kbd> Command palette</span>
                  <span className="text-fg-faint">Search notes and actions</span>
                </div>
                <div className="nc-shortcut-row">
                  <span><Kbd>Ctrl</Kbd> <Kbd>P</Kbd> Quick switcher</span>
                  <span className="text-fg-faint">Jump to any note</span>
                </div>
              </div>
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
          <div className="nc-welcome">
            <div className="nc-welcome-inner">
              <div className="flex items-center gap-2 text-[10px] font-semibold uppercase tracking-[0.12em] text-fg-faint">
                <Clock3 size={12} /> Pick up where you left off
              </div>
              <h1 className="nc-welcome-title mt-3 text-[clamp(26px,3.4vw,36px)]">
                A quiet place for <em>clear thinking</em>.
              </h1>
              <p className="nc-welcome-sub">
                {baseName(vaultRoot)} is open. Open a note, follow a link, or start something new.
              </p>

              <div className="nc-welcome-actions">
                <button type="button" className="nc-btn nc-btn-primary" onClick={() => void createNote()}>
                  <FilePlus2 size={14} /> New note
                </button>
                <button type="button" className="nc-btn nc-btn-outline" onClick={() => setQuickSwitcherOpen(true)}>
                  <FolderOpen size={14} /> Open note <Kbd>Ctrl P</Kbd>
                </button>
                <button type="button" className="nc-btn nc-btn-ghost" onClick={() => setCommandPaletteOpen(true)}>
                  <Keyboard size={14} /> Commands
                </button>
              </div>

              {recent.length > 0 ? (
                <>
                  <div className="nc-section-label mb-2">Recent notes</div>
                  <div className="nc-welcome-grid">
                    {recent.map((file) => (
                      <button
                        key={file.path}
                        type="button"
                        className="nc-welcome-card"
                        onClick={() => openTab(file.path)}
                      >
                        <div className="nc-welcome-card-label">
                          {dirName(file.path) || "Vault"}
                        </div>
                        <div className="nc-welcome-card-title">{stem(file.path)}</div>
                        <div className="nc-welcome-card-sub">
                          {formatRelativeTime(file.modified)} · {file.size.toLocaleString()} bytes
                        </div>
                      </button>
                    ))}
                  </div>
                </>
              ) : (
                <div className="nc-welcome-grid">
                  <div className="nc-welcome-card">
                    <div className="nc-welcome-card-label">Start here</div>
                    <div className="nc-welcome-card-title">Create your first note</div>
                    <div className="nc-welcome-card-sub">Markdown, instantly saved.</div>
                  </div>
                  <div className="nc-welcome-card">
                    <div className="nc-welcome-card-label">Tip</div>
                    <div className="nc-welcome-card-title">Link with [[double brackets]]</div>
                    <div className="nc-welcome-card-sub">Connections build the graph automatically.</div>
                  </div>
                </div>
              )}

              <div className="mt-6 grid max-w-md grid-cols-2 gap-x-6">
                <div className="nc-shortcut-row">
                  <span><Kbd>Ctrl</Kbd> <Kbd>N</Kbd> New note</span>
                </div>
                <div className="nc-shortcut-row">
                  <span><Kbd>Ctrl</Kbd> <Kbd>K</Kbd> Commands</span>
                </div>
                <div className="nc-shortcut-row">
                  <span><Kbd>Ctrl</Kbd> <Kbd>P</Kbd> Quick switch</span>
                </div>
                <div className="nc-shortcut-row">
                  <span><Kbd>Ctrl</Kbd> <Kbd>\</Kbd> Context panel</span>
                </div>
              </div>
            </div>
          </div>
        )}

        {!active && !vaultRoot && (
          <div className="pointer-events-none absolute inset-x-0 bottom-6 flex justify-center text-[10.5px] text-fg-faint">
            All notes stay local. No account, no sync, no telemetry.
          </div>
        )}
      </div>

      {menu && <ContextMenu x={menu.x} y={menu.y} items={menuItems} onClose={() => setMenu(null)} />}
    </div>
  );
}
