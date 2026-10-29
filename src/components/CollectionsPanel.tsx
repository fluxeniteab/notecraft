import { useCallback, useEffect, useMemo, useState } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { toast } from "sonner";
import {
  CalendarDays,
  Columns3,
  LayoutGrid,
  List as ListIcon,
  Loader2,
  Pencil,
  Plus,
  Table2,
  Trash2,
} from "lucide-react";
import { useAppStore } from "../store/appStore";
import { api } from "../lib/tauri";
import { dirName, formatRelativeTime } from "../lib/files";
import type { CollectionConfig } from "../lib/types";
import { Kbd, Modal, DialogCloseButton } from "./ui";

type ViewKind = NonNullable<CollectionConfig["config"]["view"]>;

interface CollectionNote {
  path: string;
  title: string;
  properties: Record<string, string>;
  tags: string[];
  modified: number;
  due?: string | null;
}

const viewOptions: Array<{ id: ViewKind; label: string; icon: React.ReactNode }> = [
  { id: "table", label: "Table", icon: <Table2 size={13} /> },
  { id: "list", label: "List", icon: <ListIcon size={13} /> },
  { id: "kanban", label: "Kanban", icon: <Columns3 size={13} /> },
  { id: "calendar", label: "Calendar", icon: <CalendarDays size={13} /> },
];

const propertyKeys = (notes: CollectionNote[]): string[] => {
  const keys = new Set<string>();
  notes.forEach((note) => Object.keys(note.properties).forEach((key) => keys.add(key)));
  return [...keys].sort();
};

/**
 * Dynamic collections: filter the vault by folder/tag/property and render the
 * matching notes as a table, list, kanban board or calendar. Note content in
 * the vault stays the source of truth — the view never stores note data.
 */
export default function CollectionsPanel() {
  const openTab = useAppStore((s) => s.openTab);
  const vaultRoot = useAppStore((s) => s.vaultRoot);
  const [collections, setCollections] = useState<CollectionConfig[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [notes, setNotes] = useState<CollectionNote[] | null>(null);
  const [editing, setEditing] = useState<CollectionConfig | null>(null);
  const [creating, setCreating] = useState(false);
  const [loading, setLoading] = useState(false);
  const [folders, setFolders] = useState<string[]>([]);

  useEffect(() => {
    if (!vaultRoot) return;
    api
      .listCollections()
      .then((items) => {
        setCollections(items);
        setSelected((current) => current ?? items[0]?.id ?? null);
      })
      .catch(() => setCollections([]));
    api
      .listAllFiles()
      .then((files) => {
        const set = new Set<string>();
        files.forEach((file) => {
          const dir = dirName(file.path);
          if (dir) set.add(dir);
        });
        setFolders([...set].sort());
      })
      .catch(() => setFolders([]));
  }, [vaultRoot]);

  const refreshNotes = useCallback(async (id: string) => {
    setLoading(true);
    try {
      setNotes(await api.collectionNotes(id));
    } catch (error) {
      setNotes([]);
      toast.error("Could not load collection", { description: String(error) });
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (selected) void refreshNotes(selected);
    else setNotes(null);
  }, [selected, refreshNotes]);

  const active = collections.find((item) => item.id === selected) ?? null;
  const view: ViewKind = active?.config.view ?? "table";
  const keys = useMemo(() => (notes ? propertyKeys(notes) : []), [notes]);
  const dueKey = keys.includes("due") ? "due" : keys.includes("date") ? "date" : null;

  const kanbanColumns = useMemo(() => {
    if (!notes || !dueKey) return [];
    const today = new Date().toISOString().slice(0, 10);
    const columns: Array<{ id: string; label: string; items: CollectionNote[] }> = [
      { id: "overdue", label: "Overdue", items: [] },
      { id: "today", label: "Today", items: [] },
      { id: "later", label: "Later", items: [] },
      { id: "none", label: "No date", items: [] },
    ];
    notes.forEach((note) => {
      const due = note.due ?? note.properties[dueKey] ?? null;
      if (!due) columns[3].items.push(note);
      else if (due < today) columns[0].items.push(note);
      else if (due === today) columns[1].items.push(note);
      else columns[2].items.push(note);
    });
    return columns;
  }, [dueKey, notes]);

  const calendarWeeks = useMemo(() => {
    if (!notes || !dueKey) return [];
    const dated = notes
      .map((note) => ({ note, due: note.due ?? note.properties[dueKey] ?? null }))
      .filter((entry): entry is { note: CollectionNote; due: string } => !!entry.due);
    if (!dated.length) return [];
    const dates = dated.map((entry) => entry.due).sort();
    const start = new Date(`${dates[0]}T00:00:00`);
    const end = new Date(`${dates[dates.length - 1]}T00:00:00`);
    // Clamp the rendered range so a bad date cannot hang the panel.
    if (end.getTime() - start.getTime() > 1000 * 60 * 60 * 24 * 370) {
      return [];
    }
    const weeks: Array<Array<{ date: string; items: CollectionNote[] } | null>> = [];
    let week: Array<{ date: string; items: CollectionNote[] } | null> = [];
    const cursor = new Date(start);
    const toIso = (date: Date) =>
      `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(
        date.getDate()
      ).padStart(2, "0")}`;
    // Pad the first week to its weekday boundary.
    for (let i = 0; i < cursor.getDay(); i++) week.push(null);
    for (;;) {
      const iso = toIso(cursor);
      week.push({ date: iso, items: [] });
      if (cursor >= end) break;
      cursor.setDate(cursor.getDate() + 1);
      if (week.length === 7) {
        weeks.push(week);
        week = [];
      }
    }
    if (week.length) weeks.push(week);
    const byDate = new Map<string, CollectionNote[]>();
    dated.forEach(({ due, note }) => {
      if (!byDate.has(due)) byDate.set(due, []);
      byDate.get(due)!.push(note);
    });
    weeks.flat().forEach((cell) => {
      if (cell) cell.items = byDate.get(cell.date) ?? [];
    });
    return weeks;
  }, [dueKey, notes]);

  const saveNew = async (name: string, folder: string, tag: string, viewKind: ViewKind) => {
    const id = crypto.randomUUID();
    const config: CollectionConfig["config"] = {
      view: viewKind,
      folder: folder.trim() || undefined,
      tag: tag.trim().replace(/^#/, "") || undefined,
    };
    await api.saveCollection(id, name.trim(), config);
    setCollections(await api.listCollections());
    setSelected(id);
  };

  const saveEdit = async (values: {
    name: string;
    folder: string;
    tag: string;
    property: string;
    sort: string;
  }) => {
    if (!editing) return;
    const config: CollectionConfig["config"] = {
      ...editing.config,
      folder: values.folder.trim() || undefined,
      tag: values.tag.trim().replace(/^#/, "") || undefined,
      property: values.property.trim() || undefined,
      sort: values.sort,
    };
    await api.saveCollection(editing.id, values.name.trim(), config);
    setCollections(await api.listCollections());
    setEditing(null);
    toast.success("Collection saved");
  };

  if (!vaultRoot) return <div className="nc-panel-empty">Open a vault to use collections.</div>;

  return (
    <div>
      <div className="mb-2 flex items-center gap-1.5">
        <select
          className="nc-select !h-7 min-w-0 flex-1 text-[10px]"
          value={selected ?? ""}
          onChange={(event) => setSelected(event.target.value || null)}
          aria-label="Select collection"
        >
          <option value="">Choose collection…</option>
          {collections.map((item) => (
            <option key={item.id} value={item.id}>
              {item.name}
            </option>
          ))}
        </select>
        <button
          type="button"
          className="nc-icon-btn nc-sm"
          aria-label="New collection"
          onClick={() => setCreating(true)}
        >
          <Plus size={13} />
        </button>
        <button
          type="button"
          className="nc-icon-btn nc-sm"
          aria-label="Edit collection"
          disabled={!active}
          onClick={() => active && setEditing({ ...active })}
        >
          <Pencil size={13} />
        </button>
        <button
          type="button"
          className="nc-icon-btn nc-sm"
          aria-label="Delete collection"
          disabled={!active}
          onClick={async () => {
            if (!active) return;
            try {
              await api.deleteCollection(active.id);
              setCollections((current) => current.filter((item) => item.id !== active.id));
              setSelected(null);
              toast.success("Collection deleted");
            } catch (error) {
              toast.error("Could not delete collection", { description: String(error) });
            }
          }}
        >
          <Trash2 size={13} />
        </button>
      </div>

      {!active && (
        <div className="nc-panel-empty">
          Create a collection to view notes by folder, tag or property as a table, list, kanban or
          calendar.
        </div>
      )}

      {active && (
        <>
          <div className="mb-2 flex items-center gap-1">
            {viewOptions.map((option) => (
              <button
                key={option.id}
                type="button"
                className="nc-icon-btn nc-sm"
                aria-label={`${option.label} view`}
                aria-pressed={view === option.id}
                data-active-view={view === option.id || undefined}
                title={option.label}
                onClick={async () => {
                  const config = { ...active.config, view: option.id };
                  setCollections((current) =>
                    current.map((item) => (item.id === active.id ? { ...item, config } : item))
                  );
                  await api.saveCollection(active.id, active.name, config).catch(() => {});
                }}
                style={{
                  color: view === option.id ? "var(--nc-accent)" : undefined,
                }}
              >
                {option.icon}
              </button>
            ))}
            <div className="flex-1" />
            {notes && (
              <span className="text-[9.5px] tabular-nums text-fg-faint">{notes.length} notes</span>
            )}
          </div>

          {loading && (
            <div className="flex items-center gap-2 px-1 py-3 text-[11px] text-fg-muted">
              <Loader2 size={12} className="animate-spin" /> Loading…
            </div>
          )}

          {!loading && notes && notes.length === 0 && (
            <div className="nc-panel-empty">No notes match this collection yet.</div>
          )}

          {!loading && notes && notes.length > 0 && view === "table" && (
            <div className="nc-scroll overflow-x-auto">
              <table className="nc-collection-table">
                <thead>
                  <tr>
                    <th>Note</th>
                    {keys.slice(0, 4).map((key) => (
                      <th key={key}>{key}</th>
                    ))}
                    <th>Modified</th>
                  </tr>
                </thead>
                <tbody>
                  {notes.map((note) => (
                    <tr key={note.path} onClick={() => openTab(note.path)}>
                      <td title={note.path}>{note.title}</td>
                      {keys.slice(0, 4).map((key) => (
                        <td key={key}>{note.properties[key] ?? ""}</td>
                      ))}
                      <td>{note.modified ? formatRelativeTime(note.modified) : "—"}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {!loading && notes && notes.length > 0 && view === "list" && (
            <div>
              {notes.map((note) => (
                <button
                  key={note.path}
                  type="button"
                  className="nc-panel-link"
                  onClick={() => openTab(note.path)}
                >
                  <LayoutGrid size={12} className="shrink-0 text-fg-faint" />
                  <span className="min-w-0 flex-1 truncate">{note.title}</span>
                  <span className="max-w-[35%] truncate text-[9.5px] text-fg-faint">
                    {dirName(note.path)}
                  </span>
                </button>
              ))}
            </div>
          )}

          {!loading && notes && notes.length > 0 && view === "kanban" && (
            <div className="flex gap-2 overflow-x-auto pb-2">
              {(kanbanColumns.length
                ? kanbanColumns
                : [{ id: "all", label: "All notes", items: notes }]
              ).map((column) => (
                <div key={column.id} className="nc-kanban-col">
                  <div className="nc-kanban-col-title">
                    {column.label} <span className="nc-count">{column.items.length}</span>
                  </div>
                  {column.items.map((note) => (
                    <button
                      key={note.path}
                      type="button"
                      className="nc-kanban-card"
                      onClick={() => openTab(note.path)}
                    >
                      <span className="block truncate text-[11px] text-fg-secondary">{note.title}</span>
                      {dueKey && (note.due ?? note.properties[dueKey]) && (
                        <span className="mt-0.5 block text-[9.5px] text-fg-faint">
                          {note.due ?? note.properties[dueKey]}
                        </span>
                      )}
                    </button>
                  ))}
                  {column.items.length === 0 && (
                    <div className="px-1 py-2 text-[10px] text-fg-faint">Empty</div>
                  )}
                </div>
              ))}
            </div>
          )}

          {!loading && notes && notes.length > 0 && view === "calendar" && (
            <div>
              {!dueKey && (
                <div className="nc-panel-empty mb-1.5">
                  No <Kbd>due</Kbd> or <Kbd>date</Kbd> property found — add one to frontmatter to
                  place notes on a calendar.
                </div>
              )}
              {dueKey && calendarWeeks.length === 0 && (
                <div className="nc-panel-empty">No dated notes to show.</div>
              )}
              {calendarWeeks.length > 0 && (
                <div className="nc-calendar-grid" role="grid" aria-label="Collection calendar">
                  {["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"].map((day) => (
                    <div key={day} className="nc-calendar-head" role="columnheader">
                      {day}
                    </div>
                  ))}
                  {calendarWeeks.flat().map((cell, index) =>
                    cell ? (
                      <div
                        key={cell.date}
                        className="nc-calendar-cell"
                        role="gridcell"
                        aria-label={cell.date}
                      >
                        <span className="nc-calendar-date">{Number(cell.date.slice(8, 10))}</span>
                        {cell.items.slice(0, 2).map((note) => (
                          <button
                            key={note.path}
                            type="button"
                            className="nc-calendar-item"
                            title={note.title}
                            onClick={() => openTab(note.path)}
                          >
                            {note.title}
                          </button>
                        ))}
                        {cell.items.length > 2 && (
                          <span className="nc-calendar-more">+{cell.items.length - 2}</span>
                        )}
                      </div>
                    ) : (
                      <div key={`pad-${index}`} className="nc-calendar-cell" role="gridcell" />
                    )
                  )}
                </div>
              )}
            </div>
          )}
        </>
      )}

      <NewCollectionDialog
        open={creating}
        folders={folders}
        onClose={() => setCreating(false)}
        onCreate={saveNew}
      />

      <EditCollectionDialog
        collection={editing}
        folders={folders}
        onClose={() => setEditing(null)}
        onSave={saveEdit}
      />
    </div>
  );
}

function NewCollectionDialog({
  open,
  folders,
  onClose,
  onCreate,
}: {
  open: boolean;
  folders: string[];
  onClose: () => void;
  onCreate: (name: string, folder: string, tag: string, view: ViewKind) => Promise<void>;
}) {
  const [name, setName] = useState("");
  const [folder, setFolder] = useState("");
  const [tag, setTag] = useState("");
  const [view, setView] = useState<ViewKind>("table");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (open) {
      setName("");
      setFolder("");
      setTag("");
      setView("table");
      setBusy(false);
    }
  }, [open]);

  const submit = async () => {
    if (!name.trim() || busy) return;
    setBusy(true);
    try {
      await onCreate(name, folder, tag, view);
      onClose();
    } catch (error) {
      toast.error("Could not create collection", { description: String(error) });
      setBusy(false);
    }
  };

  return (
    <Modal open={open} onOpenChange={(next) => !next && onClose()} className="!w-[460px]" labelledBy="nc-new-collection-title">
      <div className="nc-modal-head">
        <Dialog.Title id="nc-new-collection-title" className="nc-modal-title">
          New collection
        </Dialog.Title>
        <div className="flex-1" />
        <DialogCloseButton onClose={onClose} />
      </div>
      <div className="nc-modal-body space-y-3">
        <label className="block">
          <span className="mb-1.5 block text-[11px] font-medium text-fg-secondary">Name</span>
          <input
            autoFocus
            className="nc-input"
            value={name}
            placeholder="Meeting notes"
            onChange={(event) => setName(event.target.value)}
          />
        </label>
        <label className="block">
          <span className="mb-1.5 block text-[11px] font-medium text-fg-secondary">Folder (optional)</span>
          <input
            className="nc-input"
            list="nc-collection-folders"
            value={folder}
            placeholder="projects"
            onChange={(event) => setFolder(event.target.value)}
          />
          <datalist id="nc-collection-folders">
            {folders.map((item) => (
              <option key={item} value={item} />
            ))}
          </datalist>
        </label>
        <label className="block">
          <span className="mb-1.5 block text-[11px] font-medium text-fg-secondary">Tag (optional)</span>
          <input
            className="nc-input"
            value={tag}
            placeholder="project"
            onChange={(event) => setTag(event.target.value)}
          />
        </label>
        <div>
          <span className="mb-1.5 block text-[11px] font-medium text-fg-secondary">View</span>
          <div className="flex gap-1.5">
            {viewOptions.map((option) => (
              <button
                key={option.id}
                type="button"
                className="nc-btn nc-btn-ghost !h-7 !px-2 text-[10px]"
                aria-pressed={view === option.id}
                onClick={() => setView(option.id)}
              >
                {option.icon} {option.label}
              </button>
            ))}
          </div>
        </div>
      </div>
      <div className="nc-modal-foot">
        <button type="button" className="nc-btn nc-btn-outline" onClick={onClose}>
          Cancel
        </button>
        <button type="button" className="nc-btn nc-btn-primary" onClick={() => void submit()} disabled={!name.trim() || busy}>
          Create
        </button>
      </div>
    </Modal>
  );
}

function EditCollectionDialog({
  collection,
  folders,
  onClose,
  onSave,
}: {
  collection: CollectionConfig | null;
  folders: string[];
  onClose: () => void;
  onSave: (values: {
    name: string;
    folder: string;
    tag: string;
    property: string;
    sort: string;
  }) => Promise<void>;
}) {
  const [name, setName] = useState("");
  const [folder, setFolder] = useState("");
  const [tag, setTag] = useState("");
  const [property, setProperty] = useState("");
  const [sort, setSort] = useState("name");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (collection) {
      setName(collection.name);
      setFolder(collection.config.folder ?? "");
      setTag(collection.config.tag ?? "");
      setProperty(collection.config.property ?? "");
      setSort(collection.config.sort ?? "name");
      setBusy(false);
    }
  }, [collection]);

  if (!collection) return null;

  return (
    <Modal open onOpenChange={(next) => !next && onClose()} className="!w-[460px]" labelledBy="nc-edit-collection-title">
      <div className="nc-modal-head">
        <Dialog.Title id="nc-edit-collection-title" className="nc-modal-title">
          Edit collection
        </Dialog.Title>
        <div className="flex-1" />
        <DialogCloseButton onClose={onClose} />
      </div>
      <div className="nc-modal-body space-y-3">
        <label className="block">
          <span className="mb-1.5 block text-[11px] font-medium text-fg-secondary">Name</span>
          <input className="nc-input" value={name} onChange={(event) => setName(event.target.value)} />
        </label>
        <label className="block">
          <span className="mb-1.5 block text-[11px] font-medium text-fg-secondary">Folder</span>
          <input
            className="nc-input"
            list="nc-collection-folders-edit"
            value={folder}
            onChange={(event) => setFolder(event.target.value)}
          />
          <datalist id="nc-collection-folders-edit">
            {folders.map((item) => (
              <option key={item} value={item} />
            ))}
          </datalist>
        </label>
        <label className="block">
          <span className="mb-1.5 block text-[11px] font-medium text-fg-secondary">Tag</span>
          <input className="nc-input" value={tag} onChange={(event) => setTag(event.target.value)} />
        </label>
        <label className="block">
          <span className="mb-1.5 block text-[11px] font-medium text-fg-secondary">
            Required property (optional)
          </span>
          <input
            className="nc-input"
            value={property}
            placeholder="status"
            onChange={(event) => setProperty(event.target.value)}
          />
        </label>
        <label className="block">
          <span className="mb-1.5 block text-[11px] font-medium text-fg-secondary">Sort</span>
          <select className="nc-select" value={sort} onChange={(event) => setSort(event.target.value)}>
            <option value="name">Title</option>
            <option value="modified">Modified</option>
            <option value="due">Due date</option>
          </select>
        </label>
      </div>
      <div className="nc-modal-foot">
        <button type="button" className="nc-btn nc-btn-outline" onClick={onClose}>
          Cancel
        </button>
        <button
          type="button"
          className="nc-btn nc-btn-primary"
          disabled={!name.trim() || busy}
          onClick={async () => {
            if (!name.trim() || busy) return;
            setBusy(true);
            try {
              await onSave({ name, folder, tag, property, sort });
            } finally {
              setBusy(false);
            }
          }}
        >
          Save
        </button>
      </div>
    </Modal>
  );
}
