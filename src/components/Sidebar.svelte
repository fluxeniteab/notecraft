<script lang="ts">
  import { get } from "svelte/store";
  import {
    ChevronLeft,
    Clock3,
    FilePlus2,
    Files,
    FolderPlus,
    Moon,
    RefreshCw,
    Search,
    Settings2,
    Star,
    Sun,
  } from "lucide-svelte";
  import {
    allFiles,
    expanded,
    favoritePaths,
    leftVisible,
    openTab,
    recentNotes,
    saveSession,
    searchQuery,
    searchResults,
    searching,
    selectedPath,
    settings,
    settingsOpen,
    sidebarView,
    toggleFavorite,
    vaultRoot,
    pushToast,
  } from "../lib/state";
  import { persistSettings, toggleTheme } from "../lib/actions";
  import {
    createDocument,
    createFolder,
    createNamedDocument,
    deletePath,
    duplicatePath,
    movePath,
    pickVault,
    renamePath,
    runSearch,
  } from "../lib/actions";
  import { DOCUMENT_KINDS, capabilitiesOf } from "../lib/documentTypes";
  import { baseName, normalizeRel } from "../lib/files";
  import { buildTree, filterTree, type TreeNode } from "../lib/tree";
  import FileRow from "./FileRow.svelte";
  import Menu from "./Menu.svelte";
  import type { MenuItem } from "../lib/menu";

  let filter = "";
  let newMenu: { x: number; y: number; items: MenuItem[] } | null = null;
  let contextMenu: { x: number; y: number; items: MenuItem[] } | null = null;

  $: root = $vaultRoot;
  $: sort = $settings.explorer_sort;
  $: grouped = $settings.explorer_group_dirs;
  $: tree = buildTree($allFiles, sort, grouped);
  $: filtered = filterTree(tree, filter);
  $: rows = visibleRows(filtered, $expanded, filter.trim().length > 0);
  $: pinned = $favoritePaths
    .map((path) => $allFiles.find((file) => file.path === path && !file.is_dir))
    .filter((file): file is NonNullable<typeof file> => !!file)
    .slice(0, 5);
  $: recents = $recentNotes
    .map((path) => $allFiles.find((file) => file.path === path && !file.is_dir))
    .filter((file): file is NonNullable<typeof file> => !!file)
    .slice(0, 5);

  function visibleRows(
    nodes: TreeNode[],
    open: Set<string>,
    autoExpand: boolean,
    depth = 0,
    out: Array<{ node: TreeNode; depth: number }> = []
  ): Array<{ node: TreeNode; depth: number }> {
    for (const node of nodes) {
      out.push({ node, depth });
      if (node.isDir && (autoExpand || open.has(node.path))) {
        visibleRows(node.children, open, autoExpand, depth + 1, out);
      }
    }
    return out;
  }

  function toggleDir(path: string): void {
    expanded.update((set) => {
      const next = new Set(set);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });
  }

  function handleOpen(path: string): void {
    selectedPath.set(normalizeRel(path));
    void openTab(path);
  }

  function openNewMenu(event: MouseEvent): void {
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    newMenu = {
      x: rect.left,
      y: rect.bottom + 4,
      items: [
        { header: "New document" },
        ...DOCUMENT_KINDS.map((kind) => ({
          label: capabilitiesOf(kind).label,
          shortcut: `.${capabilitiesOf(kind).extension}`,
          action: () => void createDocument(kind),
        })),
        { separator: true },
        { label: "Name it first…", action: () => void createNamedDocument("markdown") },
        { label: "New folder", action: () => void createFolder() },
      ],
    };
  }

  function openContext(event: MouseEvent, node: TreeNode): void {
    event.preventDefault();
    selectedPath.set(node.path);
    const isPinned = get(favoritePaths).includes(node.path);
    const items: MenuItem[] = node.isDir
      ? [
          { label: "New file here…", action: () => void createNamedDocument("markdown") },
          { label: "New folder here…", action: () => void createFolder() },
          { separator: true },
          { label: "Rename folder…", action: () => void renamePath(node.path) },
          { label: "Move to trash", danger: true, action: () => void deletePath(node.path) },
          { separator: true },
          {
            label: "Copy relative path",
            action: () => void copyPath(node.path),
          },
        ]
      : [
          { label: "Open", shortcut: "Enter", action: () => handleOpen(node.path) },
          { separator: true },
          { label: "Rename…", action: () => void renamePath(node.path) },
          { label: "Duplicate", action: () => void duplicatePath(node.path) },
          {
            label: isPinned ? "Unpin from favorites" : "Pin to favorites",
            action: () => toggleFavorite(node.path),
          },
          {
            label: "Copy relative path",
            action: () => void copyPath(node.path),
          },
          { separator: true },
          { label: "Move to trash", danger: true, action: () => void deletePath(node.path) },
        ];
    contextMenu = { x: event.clientX, y: event.clientY, items };
  }

  async function copyPath(path: string): Promise<void> {
    try {
      await navigator.clipboard.writeText(path);
      pushToast("success", "Path copied", path);
    } catch {
      pushToast("error", "Clipboard is not available");
    }
  }

  function submitSearch(): void {
    if (filter.trim()) void runSearch(filter);
  }

  async function cycleSort(): Promise<void> {
    const order = ["name", "modified", "size"] as const;
    const index = order.indexOf(get(settings).explorer_sort);
    await persistSettings({ explorer_sort: order[(index + 1) % order.length] });
  }

  function collapse(): void {
    leftVisible.set(false);
    saveSession();
  }
</script>

<aside class="sidebar">
  <div class="sidebar-header">
    <div class="sidebar-title">
      <span class="profile-dot">{root ? baseName(root).slice(0, 1).toUpperCase() : "N"}</span>
      <span title={root ?? "No vault open"}>{root ? baseName(root) : "No vault"}</span>
    </div>
    <button type="button" class="icon-btn sm" title="New document" on:click={openNewMenu} aria-label="New document">
      <FilePlus2 size={14} />
    </button>
    <button type="button" class="icon-btn sm" title="New folder" on:click={() => void createFolder()} aria-label="New folder">
      <FolderPlus size={14} />
    </button>
    <button type="button" class="icon-btn sm" title="Collapse sidebar  Ctrl+B" on:click={collapse} aria-label="Collapse sidebar">
      <ChevronLeft size={14} />
    </button>
  </div>

  <div class="sidebar-search">
    <div class="search-input">
      <Search size={13} />
      <input
        id="sidebar-search"
        bind:value={filter}
        placeholder="Search files…"
        spellcheck="false"
        on:keydown={(event) => {
          if (event.key === "Enter") {
            event.preventDefault();
            submitSearch();
          }
          if (event.key === "Escape") filter = "";
        }}
      />
      {#if filter}
        <button
          type="button"
          class="icon-btn sm"
          style="position:absolute; right:2px; width:22px; height:22px"
          aria-label="Clear search"
          on:click={() => (filter = "")}
        >×</button>
      {/if}
    </div>
  </div>

  <div class="sidebar-nav">
    <button type="button" class="nav-row" class:active={$sidebarView === "files"} on:click={() => sidebarView.set("files")}>
      <Files size={14} />
      Files
      <span class="count">{$allFiles.filter((file) => !file.is_dir).length}</span>
    </button>
    <button
      type="button"
      class="nav-row"
      class:active={$sidebarView === "search"}
      on:click={() => {
        sidebarView.set("search");
        if (filter.trim()) void runSearch(filter);
      }}
    >
      <Search size={14} />
      Search
      <span class="count">{$allFiles.filter((file) => !file.is_dir).length > 0 ? "full text" : ""}</span>
    </button>
  </div>

  <div class="sidebar-scroll">
    {#if !root}
      <div class="tree-empty">
        No vault is open. Open a folder to start writing — Markdown, MDX, JSON and code files all work.
      </div>
    {:else if $sidebarView === "search"}
      <div class="section-head">
        Results
        {#if $searching}<RefreshCw size={11} class="spin" />{/if}
        <span class="spacer"></span>
        {#if $searchQuery}<span style="text-transform:none; letter-spacing:0; font-weight:400">“{$searchQuery}”</span>{/if}
      </div>
      {#if !$searchQuery}
        <div class="panel-empty">Type a query above and press Enter to search every document's content.</div>
      {:else if !$searching && $searchResults.length === 0}
        <div class="panel-empty">No documents match that query.</div>
      {:else}
        {#each $searchResults as hit}
          <button type="button" class="panel-row" on:click={() => handleOpen(hit.path)} title={hit.path}>
            <div style="min-width:0; flex:1">
              <div style="color:var(--nc-fg); overflow:hidden; text-overflow:ellipsis; white-space:nowrap">{hit.title || baseName(hit.path)}</div>
              <div class="meta" style="overflow:hidden; text-overflow:ellipsis; white-space:nowrap">{hit.snippet}</div>
            </div>
          </button>
        {/each}
      {/if}
    {:else}
      {#if pinned.length}
        <div class="section-head"><Star size={11} /> Pinned</div>
        {#each pinned as file}
          <button type="button" class="pin-row" title={file.path} on:click={() => handleOpen(file.path)}>
            <Star size={12} style="color:var(--nc-fg-faint)" />
            <span>{file.name}</span>
          </button>
        {/each}
      {/if}

      <div class="section-head">
        Explorer
        <span class="spacer"></span>
        <button
          type="button"
          class="icon-btn sm"
          title="Sort: {$settings.explorer_sort}. Click to cycle."
          on:click={() => void cycleSort()}
          style="width:18px; height:18px"
        >
          <RefreshCw size={11} />
        </button>
      </div>

      {#if rows.length === 0}
        <div class="tree-empty">This folder is empty. Create a file to get started.</div>
      {:else}
        <div class="tree">
          {#each rows as row (row.node.path)}
            <FileRow
              node={row.node}
              depth={row.depth}
              expanded={$expanded}
              selected={$selectedPath}
              ontoggle={toggleDir}
              onopen={handleOpen}
              oncontext={openContext}
              onmove={(from, to) => void movePath(from, to)}
            />
          {/each}
        </div>
      {/if}

      {#if recents.length}
        <div class="section-head"><Clock3 size={11} /> Recent</div>
        {#each recents as file}
          <button type="button" class="pin-row" title={file.path} on:click={() => handleOpen(file.path)}>
            <Clock3 size={12} style="color:var(--nc-fg-faint)" />
            <span>{file.name}</span>
          </button>
        {/each}
      {/if}
    {/if}
  </div>

  <div class="sidebar-footer">
    <button type="button" class="icon-btn" title="Open a vault" aria-label="Open a vault" on:click={() => void pickVault("open")}>
      <FolderPlus size={14} />
    </button>
    <div style="flex:1"></div>
    <button
      type="button"
      class="icon-btn"
      title={$settings.theme === "dark" ? "Light theme" : "Dark theme"}
      aria-label="Toggle theme"
      on:click={() => void toggleTheme()}
    >
      {#if $settings.theme === "dark"}<Sun size={14} />{:else}<Moon size={14} />{/if}
    </button>
    <button type="button" class="icon-btn" title="Settings  Ctrl+," aria-label="Settings" on:click={() => settingsOpen.set(true)}>
      <Settings2 size={14} />
    </button>
    <button
      type="button"
      class="icon-btn"
      title="Create a document"
      aria-label="Create a document"
      on:click={() => void createDocument("markdown")}
    >
      <FilePlus2 size={14} />
    </button>
  </div>
</aside>

{#if newMenu}
  <Menu x={newMenu.x} y={newMenu.y} items={newMenu.items} onclose={() => (newMenu = null)} />
{/if}

{#if contextMenu}
  <Menu x={contextMenu.x} y={contextMenu.y} items={contextMenu.items} onclose={() => (contextMenu = null)} />
{/if}

<style>
  .profile-dot {
    display: grid;
    place-items: center;
    width: 17px;
    height: 17px;
    border-radius: 4px;
    background: var(--nc-accent-soft);
    color: var(--nc-accent);
    font-size: 10px;
    font-weight: 600;
    flex: 0 0 auto;
  }
</style>
