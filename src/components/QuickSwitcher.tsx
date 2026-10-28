import { useEffect, useMemo, useRef, useState } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { Command } from "cmdk";
import { FilePlus2, FileText, FolderOpen, Loader2, Search } from "lucide-react";
import { useAppStore } from "../store/appStore";
import { api } from "../lib/tauri";
import { dirName, formatRelativeTime, isMarkdown, stem, uniqueName } from "../lib/files";
import { createNote } from "../lib/noteActions";
import { Kbd } from "./ui";

export default function QuickSwitcher() {
  const open = useAppStore((s) => s.quickSwitcherOpen);
  const setOpen = useAppStore((s) => s.setQuickSwitcherOpen);
  const allFiles = useAppStore((s) => s.allFiles);
  const setAllFiles = useAppStore((s) => s.setAllFiles);
  const recentNotes = useAppStore((s) => s.recentNotes);
  const [query, setQuery] = useState("");
  const [loading, setLoading] = useState(false);
  const loadedRef = useRef(false);

  useEffect(() => {
    if (open) setQuery("");
  }, [open]);

  useEffect(() => {
    if (!open || loadedRef.current || allFiles.length) return;
    loadedRef.current = true;
    setLoading(true);
    api
      .listAllFiles()
      .then(setAllFiles)
      .catch(() => {
        loadedRef.current = false;
      })
      .finally(() => setLoading(false));
  }, [open, allFiles.length, setAllFiles]);

  const notes = useMemo(() => allFiles.filter((file) => !file.is_dir && isMarkdown(file.path)), [allFiles]);

  const results = useMemo(() => {
    const text = query.trim().toLowerCase();
    if (!text) {
      const recent = recentNotes
        .map((path) => notes.find((note) => note.path === path))
        .filter((note): note is NonNullable<typeof note> => !!note);
      const recentPaths = new Set(recent.map((note) => note.path));
      const rest = notes.filter((note) => !recentPaths.has(note.path)).slice(0, 40);
      return [...recent, ...rest];
    }
    const scored = notes
      .map((note) => {
        const name = stem(note.path).toLowerCase();
        const path = note.path.toLowerCase();
        let score = -1;
        if (name === text) score = 0;
        else if (name.startsWith(text)) score = 1;
        else if (name.includes(text)) score = 2;
        else if (path.includes(text)) score = 3;
        return { note, score };
      })
      .filter((entry) => entry.score >= 0)
      .sort((a, b) => a.score - b.score || a.note.path.length - b.note.path.length);
    return scored.slice(0, 50).map((entry) => entry.note);
  }, [notes, query, recentNotes]);

  const exact = results.some(
    (note) => stem(note.path).toLowerCase() === query.trim().toLowerCase()
  );
  const canCreate = query.trim().length > 0 && !exact;
  const createName = query.trim() ? uniqueName(query, allFiles, "") : "";

  const close = () => setOpen(false);

  return (
    <Dialog.Root open={open} onOpenChange={(next) => !next && close()}>
      <Dialog.Portal>
        <Dialog.Overlay className="nc-overlay" />
        <Dialog.Content
          className="nc-palette-wrap"
          aria-describedby={undefined}
          onClick={(event) => {
            if (event.target === event.currentTarget) close();
          }}
        >
          <Dialog.Title className="sr-only">Quick switcher</Dialog.Title>
          <Command
            className="nc-palette"
            style={{ width: "min(560px, calc(100vw - 40px))" }}
            label="Quick switcher"
            shouldFilter={false}
            loop
          >
            <div className="nc-palette-input-row">
              <Search size={16} />
              <Command.Input
                autoFocus
                value={query}
                onValueChange={setQuery}
                placeholder="Find a note by name…"
                className="nc-palette-input"
              />
              {loading && <Loader2 size={13} className="animate-spin text-fg-faint" />}
              <Kbd>Esc</Kbd>
            </div>

            <Command.List className="nc-palette-list nc-scroll">
              {canCreate && (
                <Command.Group heading="Create">
                  <Command.Item
                    value={`create-${query}`}
                    className="nc-palette-item"
                    onSelect={() => {
                      close();
                      void createNote("", query.trim());
                    }}
                  >
                    <FilePlus2 size={14} className="nc-palette-icon" />
                    <span className="nc-palette-item-body">
                      <span className="nc-palette-item-title">
                        Create “{query.trim()}”
                      </span>
                      <span className="nc-palette-item-sub">
                        New note {createName ? `· ${createName}` : "at the vault root"}
                      </span>
                    </span>
                  </Command.Item>
                </Command.Group>
              )}

              <Command.Empty>
                <div className="px-3 py-8 text-center text-[11.5px] text-fg-muted">
                  {notes.length === 0
                    ? "No Markdown notes in this vault yet."
                    : `No notes match “${query}”.`}
                </div>
              </Command.Empty>

              {results.length > 0 && (
                <Command.Group heading={query.trim() ? "Notes" : "Recent notes"}>
                  {results.map((note) => (
                    <Command.Item
                      key={note.path}
                      value={note.path}
                      className="nc-palette-item"
                      onSelect={() => {
                        close();
                        useAppStore.getState().openTab(note.path);
                      }}
                    >
                      <FileText size={14} className="nc-palette-icon" />
                      <span className="nc-palette-item-body">
                        <span className="nc-palette-item-title">{stem(note.path)}</span>
                        <span className="nc-palette-item-sub">
                          {dirName(note.path) || "Vault root"}
                        </span>
                      </span>
                      <span className="text-[9.5px] text-fg-faint">
                        {formatRelativeTime(note.modified)}
                      </span>
                    </Command.Item>
                  ))}
                </Command.Group>
              )}
            </Command.List>

            <div className="nc-palette-foot">
              <span><Kbd>↑</Kbd><Kbd>↓</Kbd> navigate</span>
              <span><Kbd>↵</Kbd> open</span>
              <span><Kbd>Esc</Kbd> close</span>
              <div className="flex-1" />
              <span className="inline-flex items-center gap-1">
                <FolderOpen size={10} /> {notes.length.toLocaleString()} notes
              </span>
            </div>
          </Command>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
