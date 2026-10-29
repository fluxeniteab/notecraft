<script lang="ts">
  import { onMount } from "svelte";
  import {
    CalendarDays,
    Columns3,
    LayoutList,
    Loader2,
    Pencil,
    Plus,
    Table2,
    Trash2,
    X,
  } from "lucide-svelte";
  import type { CollectionConfig } from "../lib/types";
  import { collectionsOpen, openTab, pushToast, newId, vaultRoot } from "../lib/state";
  import { api } from "../lib/ipc";
  import { baseName, dirName, formatRelativeTime } from "../lib/files";

  type ViewKind = "table" | "list" | "kanban" | "calendar";

  interface CollectionNote {
    path: string;
    title: string;
    properties: Record<string, string>;
    tags: string[];
    modified: number;
    due?: string | null;
  }

  const views: Array<{ id: ViewKind; label: string; icon: typeof Table2 }> = [
    { id: "table", label: "Table", icon: Table2 },
    { id: "list", label: "List", icon: LayoutList },
    { id: "kanban", label: "Board", icon: Columns3 },
    { id: "calendar", label: "Calendar", icon: CalendarDays },
  ];

  let collections: CollectionConfig[] = [];
  let notes: CollectionNote[] | null = null;
  let selected: string | null = null;
  let loading = true;
  let busy = false;
  let editing = false;
  let draft = { name: "", folder: "", tag: "", property: "", sort: "modified", view: "table" as ViewKind };

  $: active = collections.find((item) => item.id === selected) ?? null;
  $: view = (active?.config.view ?? "table") as ViewKind;
  $: keys = notes ? propertyKeys(notes) : [];

  onMount(() => {
    void load();
    const onEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") collectionsOpen.set(false);
    };
    window.addEventListener("keydown", onEscape);
    return () => window.removeEventListener("keydown", onEscape);
  });

  async function load(): Promise<void> {
    loading = true;
    try {
      collections = await api.listCollections();
      selected = selected ?? collections[0]?.id ?? null;
      if (selected) await loadNotes(selected);
    } catch (problem) {
      pushToast("error", "Could not load collections", String(problem));
      collections = [];
    } finally {
      loading = false;
    }
  }

  async function loadNotes(id: string): Promise<void> {
    try {
      notes = await api.collectionNotes(id);
    } catch (problem) {
      notes = [];
      pushToast("error", "Could not load this collection", String(problem));
    }
  }

  function propertyKeys(items: CollectionNote[]): string[] {
    const set = new Set<string>();
    items.forEach((note) => Object.keys(note.properties).forEach((key) => set.add(key)));
    return [...set].sort();
  }

  function select(id: string): void {
    selected = id;
    editing = false;
    notes = null;
    void loadNotes(id);
  }

  function beginCreate(): void {
    selected = null;
    notes = null;
    editing = true;
    draft = { name: "", folder: "", tag: "", property: "", sort: "modified", view: "table" };
  }

  function beginEdit(): void {
    if (!active) return;
    editing = true;
    draft = {
      name: active.name,
      folder: active.config.folder ?? "",
      tag: active.config.tag ?? "",
      property: active.config.property ?? "",
      sort: active.config.sort ?? "modified",
      view: (active.config.view ?? "table") as ViewKind,
    };
  }

  async function save(): Promise<void> {
    const name = draft.name.trim();
    if (!name) {
      pushToast("error", "A collection needs a name");
      return;
    }
    busy = true;
    const id = active?.id ?? newId();
    const config: CollectionConfig["config"] = {
      view: draft.view,
      folder: draft.folder.trim() || undefined,
      tag: draft.tag.trim() || undefined,
      property: draft.property.trim() || undefined,
      sort: draft.sort || undefined,
    };
    try {
      await api.saveCollection(id, name, config);
      await load();
      selected = id;
      editing = false;
      await loadNotes(id);
      pushToast("success", "Collection saved", name);
    } catch (problem) {
      pushToast("error", "Could not save the collection", String(problem));
    } finally {
      busy = false;
    }
  }

  async function remove(): Promise<void> {
    if (!active) return;
    try {
      await api.deleteCollection(active.id);
      collections = collections.filter((item) => item.id !== active.id);
      selected = collections[0]?.id ?? null;
      notes = null;
      if (selected) await loadNotes(selected);
      editing = false;
      pushToast("success", "Collection deleted", active.name);
    } catch (problem) {
      pushToast("error", "Could not delete the collection", String(problem));
    }
  }

  function setView(next: ViewKind): void {
    if (!active) return;
    const id = active.id;
    collections = collections.map((item) =>
      item.id === id ? { ...item, config: { ...item.config, view: next } } : item
    );
    void api
      .saveCollection(id, active.name, { ...active.config, view: next })
      .catch((problem) => pushToast("error", "Could not remember the view", String(problem)));
  }

  function dueKey(): string | null {
    if (keys.includes("due")) return "due";
    if (keys.includes("date")) return "date";
    return null;
  }

  function dueOf(note: CollectionNote): string | null {
    const key = dueKey();
    return note.due ?? (key ? note.properties[key] ?? null : null);
  }

  const today = new Date().toISOString().slice(0, 10);

  $: board = (() => {
    const columns = [
      { id: "overdue", label: "Overdue", items: [] as CollectionNote[] },
      { id: "today", label: "Today", items: [] as CollectionNote[] },
      { id: "later", label: "Later", items: [] as CollectionNote[] },
      { id: "none", label: "No date", items: [] as CollectionNote[] },
    ];
    for (const note of notes ?? []) {
      const due = dueOf(note);
      if (!due) columns[3].items.push(note);
      else if (due < today) columns[0].items.push(note);
      else if (due === today) columns[1].items.push(note);
      else columns[2].items.push(note);
    }
    return columns;
  })();

  $: calendar = (() => {
    const groups = new Map<string, CollectionNote[]>();
    for (const note of notes ?? []) {
      const due = dueOf(note);
      const label = due ? due.slice(0, 7) : "No date";
      groups.set(label, [...(groups.get(label) ?? []), note]);
    }
    return [...groups.entries()].sort((a, b) => (a[0] === "No date" ? 1 : b[0] === "No date" ? -1 : a[0].localeCompare(b[0])));
  })();

  function dueLabel(note: CollectionNote): string | null {
    return note.due ?? note.properties.due ?? note.properties.date ?? null;
  }

  function monthLabel(month: string): string {
    const date = new Date(`${month}-01T00:00:00`);
    if (Number.isNaN(date.getTime())) return month;
    return date.toLocaleDateString(undefined, { month: "long", year: "numeric" });
  }

  function close(): void {
    collectionsOpen.set(false);
  }
</script>

<div class="overlay center" role="presentation" on:click={(event) => event.target === event.currentTarget && close()}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    tabindex="-1"
    class="dialog wide"
    role="dialog"
    aria-label="Collections"
    on:click|stopPropagation
    style="height:min(82vh, 740px); display:flex; flex-direction:column"
  >
    <div class="dialog-header" style="align-items:center">
      <div class="titles">
        <h3>Collections</h3>
        <p>Filter the vault by folder, tag or property. Notes stay the source of truth.</p>
      </div>
      <button type="button" class="btn sm" on:click={beginCreate}><Plus size={12} /> New</button>
      <button type="button" class="icon-btn" title="Close" aria-label="Close collections" on:click={close}>
        <X size={15} />
      </button>
    </div>

    <div class="collections">
      <nav class="collection-rail" aria-label="Saved collections">
        {#if collections.length === 0 && !loading}
          <p class="rail-empty">No collections yet.</p>
        {/if}
        {#each collections as item (item.id)}
          <button
            type="button"
            class="rail-item"
            class:on={selected === item.id}
            on:click={() => select(item.id)}
          >
            <span class="truncate">{item.name}</span>
          </button>
        {/each}
      </nav>

      <div class="collection-main">
        {#if loading}
          <div class="editor-loading"><Loader2 size={15} class="spin" /> Loading collections…</div>
        {:else if editing}
          <div class="collection-form">
            <label class="field">
              <span>Name</span>
              <input bind:value={draft.name} placeholder="Reading list" spellcheck="false" />
            </label>
            <div class="field-row">
              <label class="field">
                <span>Folder</span>
                <input bind:value={draft.folder} placeholder="notes/ideas" spellcheck="false" />
              </label>
              <label class="field">
                <span>Tag</span>
                <input bind:value={draft.tag} placeholder="project" spellcheck="false" />
              </label>
            </div>
            <div class="field-row">
              <label class="field">
                <span>Property</span>
                <input bind:value={draft.property} placeholder="status" spellcheck="false" />
              </label>
              <label class="field">
                <span>Sort</span>
                <select bind:value={draft.sort}>
                  <option value="modified">Modified</option>
                  <option value="title">Title</option>
                  <option value="due">Due date</option>
                </select>
              </label>
            </div>
            <label class="field">
              <span>View</span>
              <div class="segmented" role="group" aria-label="Default view">
                {#each views as option}
                  <button type="button" class:on={draft.view === option.id} on:click={() => (draft.view = option.id)}>
                    <svelte:component this={option.icon} size={12} /> {option.label}
                  </button>
                {/each}
              </div>
            </label>
            <div class="form-actions">
              {#if active}
                <button type="button" class="btn sm danger" on:click={remove}><Trash2 size={12} /> Delete</button>
              {/if}
              <span class="spacer"></span>
              <button type="button" class="btn sm" on:click={() => (editing = false)}>Cancel</button>
              <button type="button" class="btn sm primary" disabled={busy} on:click={save}>
                {#if busy}<Loader2 size={11} class="spin" />{/if} Save collection
              </button>
            </div>
          </div>
        {:else if !active}
          <div class="empty" style="background:transparent">
            <div class="empty-mark">·</div>
            <h2>No collection selected</h2>
            <p>Create one to group notes by folder, tag or property without moving a single file.</p>
            <div class="empty-actions">
              <button type="button" class="btn primary" on:click={beginCreate}><Plus size={12} /> New collection</button>
            </div>
          </div>
        {:else}
          <div class="collection-bar">
            <div class="segmented" role="group" aria-label="View">
              {#each views as option}
                <button type="button" class:on={view === option.id} on:click={() => setView(option.id)}>
                  <svelte:component this={option.icon} size={12} /> {option.label}
                </button>
              {/each}
            </div>
            <span class="meta">
              {notes?.length ?? 0} notes
              {#if active.config.folder}· in {active.config.folder}{/if}
              {#if active.config.tag}· tag #{active.config.tag}{/if}
            </span>
            <span class="spacer"></span>
            <button type="button" class="icon-btn sm" title="Edit collection" aria-label="Edit collection" on:click={beginEdit}>
              <Pencil size={12} />
            </button>
          </div>

          {#if notes === null}
            <div class="editor-loading"><Loader2 size={14} class="spin" /> Reading notes…</div>
          {:else if notes.length === 0}
            <div class="empty" style="background:transparent">
              <div class="empty-mark">·</div>
              <h2>Nothing matches</h2>
              <p>No note in {$vaultRoot ? baseName($vaultRoot) : "this vault"} matches this collection's filter yet.</p>
            </div>
          {:else if view === "table"}
            <div class="collection-scroll">
              <table class="collection-table">
                <thead>
                  <tr>
                    <th>Title</th>
                    <th>Folder</th>
                    <th>Tags</th>
                    {#each keys.slice(0, 4) as key}<th>{key}</th>{/each}
                    <th>Modified</th>
                  </tr>
                </thead>
                <tbody>
                  {#each notes as note (note.path)}
                    <tr on:click={() => void openTab(note.path)}>
                      <td class="title-cell">{note.title || baseName(note.path)}</td>
                      <td class="muted">{dirName(note.path) || "—"}</td>
                      <td class="muted">{note.tags.map((tag) => `#${tag}`).join(" ") || "—"}</td>
                      {#each keys.slice(0, 4) as key}<td class="muted">{note.properties[key] ?? "—"}</td>{/each}
                      <td class="muted">{formatRelativeTime(note.modified)}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          {:else if view === "list"}
            <div class="collection-scroll">
              {#each notes as note (note.path)}
                <button type="button" class="collection-row" on:click={() => void openTab(note.path)}>
                  <span class="dot"></span>
                  <span class="truncate title-cell">{note.title || baseName(note.path)}</span>
                  <span class="spacer"></span>
                  {#if dueLabel(note)}<span class="tag-pill">{dueLabel(note)}</span>{/if}
                  {#each note.tags.slice(0, 2) as tag}<span class="tag-pill">#{tag}</span>{/each}
                  <span class="meta">{formatRelativeTime(note.modified)}</span>
                </button>
              {/each}
            </div>
          {:else if view === "kanban"}
            <div class="collection-board">
              {#each board as column (column.id)}
                <div class="board-column">
                  <div class="board-head">{column.label}<span class="count">{column.items.length}</span></div>
                  {#each column.items as note (note.path)}
                    <button type="button" class="board-card" on:click={() => void openTab(note.path)}>
                      <span class="truncate title-cell">{note.title || baseName(note.path)}</span>
                      <span class="meta truncate">{dirName(note.path) || "vault root"}</span>
                    </button>
                  {/each}
                </div>
              {/each}
            </div>
          {:else}
            <div class="collection-scroll">
              {#each calendar as [month, items] (month)}
                <div class="calendar-group">
                  <div class="head">{month === "No date" ? "No date" : monthLabel(month)}<span class="count">{items.length}</span></div>
                  {#each items as note (note.path)}
                    <button type="button" class="panel-row" on:click={() => void openTab(note.path)}>
                      <span class="truncate">{note.title || baseName(note.path)}</span>
                      <span class="spacer"></span>
                      <span class="meta">{dueLabel(note) ?? "—"}</span>
                    </button>
                  {/each}
                </div>
              {/each}
            </div>
          {/if}
        {/if}
      </div>
    </div>
  </div>
</div>


<style>
  .collections {
    display: flex;
    flex: 1;
    min-height: 0;
    border-top: 1px solid var(--nc-border-soft);
  }
  .collection-rail {
    width: 176px;
    flex: none;
    overflow-y: auto;
    padding: 8px 6px;
    border-right: 1px solid var(--nc-border-soft);
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .rail-empty {
    margin: 0;
    padding: 6px 8px;
    font-size: 11px;
    color: var(--nc-fg-faint);
  }
  .rail-item {
    display: flex;
    padding: 5px 8px;
    border-radius: var(--nc-r-sm);
    font-size: 11.5px;
    color: var(--nc-fg-secondary);
    text-align: left;
  }
  .rail-item:hover {
    background: var(--nc-hover);
    color: var(--nc-fg);
  }
  .rail-item.on {
    background: var(--nc-raised);
    color: var(--nc-fg);
  }
  .collection-main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .collection-bar {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--nc-border-soft);
  }
  .collection-scroll {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }
  .collection-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 11.5px;
  }
  .collection-table th {
    position: sticky;
    top: 0;
    z-index: 1;
    background: var(--nc-elev);
    padding: 7px 12px;
    font-size: 10px;
    font-weight: 500;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--nc-fg-faint);
    text-align: left;
    border-bottom: 1px solid var(--nc-border-soft);
    white-space: nowrap;
  }
  .collection-table td {
    padding: 6px 12px;
    border-bottom: 1px solid var(--nc-border-soft);
    max-width: 220px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .collection-table tbody tr {
    cursor: pointer;
  }
  .collection-table tbody tr:hover {
    background: var(--nc-hover);
  }
  .title-cell {
    color: var(--nc-fg);
  }
  .muted {
    color: var(--nc-fg-muted);
  }
  .collection-row {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 7px 12px;
    border-bottom: 1px solid var(--nc-border-soft);
    text-align: left;
    font-size: 11.5px;
  }
  .collection-row:hover {
    background: var(--nc-hover);
  }
  .dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--nc-border-strong);
    flex: none;
  }
  .tag-pill {
    flex: none;
    padding: 0 6px;
    height: 17px;
    display: inline-flex;
    align-items: center;
    border-radius: 999px;
    border: 1px solid var(--nc-border);
    font-size: 10px;
    color: var(--nc-fg-muted);
  }
  .collection-board {
    flex: 1;
    min-height: 0;
    overflow: auto;
    display: grid;
    grid-auto-flow: column;
    grid-auto-columns: minmax(180px, 1fr);
    gap: 8px;
    padding: 10px;
  }
  .board-column {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-height: 0;
    padding: 8px;
    border: 1px solid var(--nc-border-soft);
    border-radius: var(--nc-r-md);
    background: var(--nc-bg);
  }
  .board-head {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 10.5px;
    font-weight: 500;
    letter-spacing: 0.03em;
    text-transform: uppercase;
    color: var(--nc-fg-faint);
    padding-bottom: 2px;
  }
  .board-card {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 7px 8px;
    border: 1px solid var(--nc-border);
    border-radius: var(--nc-r-sm);
    background: var(--nc-panel);
    text-align: left;
    font-size: 11.5px;
  }
  .board-card:hover {
    border-color: var(--nc-border-strong);
  }
  .calendar-group {
    padding: 4px 0 10px;
  }
  .calendar-group .head {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 12px;
    font-size: 10.5px;
    letter-spacing: 0.03em;
    text-transform: uppercase;
    color: var(--nc-fg-faint);
  }
  .collection-form {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 14px 16px;
    max-width: 520px;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .field span {
    font-size: 10.5px;
    color: var(--nc-fg-muted);
  }
  .field input,
  .field select {
    height: 30px;
    padding: 0 8px;
    border: 1px solid var(--nc-border);
    border-radius: var(--nc-r-sm);
    background: var(--nc-bg);
    color: var(--nc-fg);
    font: inherit;
    font-size: 12px;
  }
  .field input:focus-visible,
  .field select:focus-visible {
    border-color: var(--nc-accent);
    outline: none;
  }
  .field-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }
  .form-actions {
    display: flex;
    align-items: center;
    gap: 6px;
    padding-top: 4px;
  }
</style>
