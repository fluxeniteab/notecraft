<script lang="ts">
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import {
    ChevronDown,
    Command,
    Lock,
    Minus,
    Moon,
    PanelLeft,
    PanelRight,
    Search,
    Settings,
    Square,
    Sun,
    X,
  } from "lucide-svelte";
  import {
    activeTab,
    closeTab,
    cycleTab,
    focusMode,
    graphOpen,
    leftVisible,
    paletteOpen,
    profile,
    recentVaults,
    rightTab,
    rightVisible,
    saveSession,
    settings,
    settingsOpen,
    settingsSection,
    shortcutsOpen,
    aboutOpen,
    updateTab,
    vaultRoot,
    pushToast,
  } from "../lib/state";
  import {
    closeVault,
    createDailyNote,
    createDocument,
    createFolder,
    createNamedDocument,
    exportActive,
    lockProfile,
    openVaultAt,
    pickVault,
    saveActive,
    snapshotActive,
    toggleTheme,
  } from "../lib/actions";
  import { api } from "../lib/ipc";
  import { baseName } from "../lib/files";
  import { capabilitiesFor } from "../lib/documentTypes";
  import { insertLink, insertText, openFind, redoEdit, selectAllText, setHeading, toggleLinePrefix, undoEdit, wrapSelection } from "../lib/editor";
  import { isTauri } from "../lib/theme";
  import Menu from "./Menu.svelte";
  import type { MenuItem } from "../lib/menu";

  let menu: { x: number; y: number; items: MenuItem[] } | null = null;
  let maximized = false;
  const tauri = isTauri();

  const names = ["File", "Edit", "Format", "View", "Insert", "Navigate", "Tools", "Plugins", "Help"];

  $: tab = $activeTab;

  onMount(() => {
    if (!tauri) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void (async () => {
      try {
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        const win = getCurrentWindow();
        maximized = await win.isMaximized();
        unlisten = await win.onResized(async () => {
          const next = await win.isMaximized().catch(() => false);
          if (!disposed) maximized = next;
        });
      } catch {
        /* browser preview */
      }
    })();
    return () => {
      disposed = true;
      unlisten?.();
    };
  });

  async function windowAction(action: "minimize" | "maximize" | "close"): Promise<void> {
    if (!tauri) return;
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      const win = getCurrentWindow();
      if (action === "minimize") await win.minimize();
      else if (action === "maximize") await win.toggleMaximize();
      else await win.close();
    } catch (error) {
      pushToast("error", "Window action failed", String(error));
    }
  }

  function setMode(mode: "source" | "live" | "reading" | "split"): void {
    if (tab) updateTab(tab.id, { mode });
  }

  function openMenu(name: string, event: MouseEvent): void {
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    menu = { x: rect.left, y: rect.bottom + 5, items: menuFor(name) };
  }

  function menuFor(name: string): MenuItem[] {
    const hasTab = !!get(activeTab);
    const isProse = !!get(activeTab) && capabilitiesFor(get(activeTab)!.path).hasLinks;
    switch (name) {
      case "File":
        return [
          { header: "New" },
          { label: "Markdown note", shortcut: "Ctrl N", action: () => void createDocument("markdown") },
          { label: "MDX document", action: () => void createDocument("mdx") },
          { label: "JSON document", action: () => void createDocument("json") },
          { label: "File with a name…", action: () => void createNamedDocument("markdown") },
          { label: "Folder…", action: () => void createFolder() },
          { separator: true },
          ...(get(recentVaults).length
            ? [
                { header: "Recent vaults" } as MenuItem,
                ...get(recentVaults).slice(0, 4).map((path) => ({
                  label: baseName(path),
                  action: () => void openVaultAt(path),
                })),
                { separator: true } as MenuItem,
              ]
            : []),
          { label: "Open vault…", shortcut: "Ctrl O", action: () => void pickVault("open") },
          { label: "Create vault…", action: () => void pickVault("create") },
          { separator: true },
          { label: "Save", shortcut: "Ctrl S", disabled: !hasTab, action: () => void saveActive() },
          { label: "Export as HTML…", disabled: !hasTab, action: () => void exportActive("html") },
          { label: "Close tab", shortcut: "Ctrl W", disabled: !hasTab, action: () => void closeTabSafe() },
          { separator: true },
          { label: "Close vault", disabled: !get(vaultRoot), action: () => void closeVault() },
          { label: "Lock profile", shortcut: "Ctrl Shift L", action: () => void lockProfile() },
        ];

      case "Edit":
        return [
          { label: "Undo", shortcut: "Ctrl Z", action: undoEdit },
          { label: "Redo", shortcut: "Ctrl Shift Z", action: redoEdit },
          { separator: true },
          { label: "Find in file…", shortcut: "Ctrl F", action: openFind },
          { label: "Select all", shortcut: "Ctrl A", action: selectAllText },
        ];

      case "Format":
        return [
          { header: "Headings" },
          { label: "Heading 1", disabled: !isProse, action: () => setHeading(1) },
          { label: "Heading 2", disabled: !isProse, action: () => setHeading(2) },
          { label: "Heading 3", disabled: !isProse, action: () => setHeading(3) },
          { separator: true },
          { label: "Bold", disabled: !isProse, action: () => wrapSelection("**") },
          { label: "Italic", disabled: !isProse, action: () => wrapSelection("*") },
          { label: "Strikethrough", disabled: !isProse, action: () => wrapSelection("~~") },
          { label: "Inline code", disabled: !isProse, action: () => wrapSelection("`") },
          { label: "Link…", disabled: !isProse, action: insertLink },
          { separator: true },
          { label: "Bulleted list", disabled: !isProse, action: () => toggleLinePrefix("- ") },
          { label: "Numbered list", disabled: !isProse, action: () => toggleLinePrefix("1. ") },
          { label: "Quote", disabled: !isProse, action: () => toggleLinePrefix("> ") },
          { label: "Code block", disabled: !isProse, action: () => insertText("\n```\n\n```\n") },
        ];

      case "View":
        return [
          {
            label: "Toggle sidebar",
            shortcut: "Ctrl B",
            action: () => {
              leftVisible.update((value) => !value);
              saveSession();
            },
          },
          {
            label: "Toggle context panel",
            shortcut: "Ctrl \\",
            action: () => {
              rightVisible.update((value) => !value);
              saveSession();
            },
          },
          {
            label: "Focus mode",
            shortcut: "Ctrl Shift F",
            action: () => {
              focusMode.update((value) => !value);
              saveSession();
            },
          },
          {
            label: "Graph view",
            action: () => graphOpen.set(true),
          },
          { separator: true },
          { label: "Source view", disabled: !hasTab, action: () => setMode("source") },
          { label: "Live preview", disabled: !hasTab || !isProse, action: () => setMode("live") },
          { label: "Reading view", disabled: !hasTab, action: () => setMode("reading") },
          { label: "Split view", disabled: !hasTab, action: () => setMode("split") },
          { separator: true },
          { label: "Toggle light / dark theme", shortcut: "Ctrl Shift D", action: () => void toggleTheme() },
        ];

      case "Insert":
        return [
          { label: "Wiki link [[…]]", disabled: !isProse, action: () => wrapSelection("[[", "]]") },
          { label: "Markdown link", disabled: !isProse, action: insertLink },
          { separator: true },
          { label: "Table", disabled: !isProse, action: () => insertText("\n| Column | Column |\n| --- | --- |\n|  |  |\n") },
          { label: "Horizontal rule", disabled: !isProse, action: () => insertText("\n---\n") },
          { label: "Callout", disabled: !isProse, action: () => insertText("\n> [!note]\n> ") },
          { separator: true },
          { label: "Current date", disabled: !hasTab, action: () => insertText(new Date().toLocaleDateString()) },
          { label: "Daily note", shortcut: "Ctrl Alt N", action: () => void createDailyNote() },
        ];

      case "Navigate":
        return [
          { label: "Command palette", shortcut: "Ctrl K", action: () => paletteOpen.set(true) },
          { label: "Go to file…", shortcut: "Ctrl P", action: () => document.dispatchEvent(new CustomEvent("notecraft:open-switcher")) },
          {
            label: "Search all documents",
            action: () => document.dispatchEvent(new CustomEvent("notecraft:focus-search")),
          },
          { separator: true },
          { label: "Next tab", shortcut: "Ctrl Tab", action: () => cycleTab(1) },
          { label: "Previous tab", shortcut: "Ctrl Shift Tab", action: () => cycleTab(-1) },
          { separator: true },
          {
            label: "Outline panel",
            action: () => {
              rightVisible.set(true);
              rightTab.set("outline");
            },
          },
          {
            label: "Backlinks panel",
            action: () => {
              rightVisible.set(true);
              rightTab.set("backlinks");
            },
          },
          {
            label: "Version history",
            action: () => {
              rightVisible.set(true);
              rightTab.set("history");
            },
          },
        ];

      case "Tools":
        return [
          { label: "Settings…", shortcut: "Ctrl ,", action: () => settingsOpen.set(true) },
          { label: "Quick capture…", shortcut: "Ctrl Shift N", action: () => document.dispatchEvent(new CustomEvent("notecraft:capture")) },
          { label: "Create snapshot", disabled: !hasTab, action: () => void snapshotActive() },
          { separator: true },
          { label: "Export HTML…", disabled: !hasTab, action: () => void exportActive("html") },
          { label: "Print / Save as PDF…", disabled: !hasTab, action: () => void exportActive("pdf") },
          { separator: true },
          {
            label: "Rebuild search index",
            disabled: !get(vaultRoot),
            action: () =>
              void api
                .rebuildIndex()
                .then(() => pushToast("success", "Search index rebuilt"))
                .catch((error) => pushToast("error", "Index rebuild failed", String(error))),
          },
        ];

      case "Plugins":
        return [
          {
            label: "Manage plugins…",
            action: () => {
              settingsSection.set("plugins");
              settingsOpen.set(true);
            },
          },
        ];

      case "Help":
        return [
          { label: "Keyboard shortcuts", action: () => shortcutsOpen.set(true) },
          { label: "About NoteCraft", action: () => aboutOpen.set(true) },
        ];

      default:
        return [];
    }
  }

  function closeTabSafe(): void {
    if (tab) void closeTab(tab.id);
  }

  function openVaultMenu(event: MouseEvent): void {
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    menu = {
      x: rect.left,
      y: rect.bottom + 5,
      items: [
        ...(get(profile) ? [{ header: get(profile)!.username } as MenuItem, { label: "Lock profile", action: () => void lockProfile() } as MenuItem, { separator: true } as MenuItem] : []),
        ...(get(recentVaults).length
          ? [
              { header: "Recent vaults" } as MenuItem,
              ...get(recentVaults).slice(0, 6).map((path) => ({
                label: baseName(path),
                action: () => void openVaultAt(path),
              })),
              { separator: true } as MenuItem,
            ]
          : []),
        { label: "Open folder…", action: () => void pickVault("open") },
        { label: "Create vault…", action: () => void pickVault("create") },
        ...(get(vaultRoot) ? [{ separator: true } as MenuItem, { label: "Close vault", action: () => void closeVault() } as MenuItem] : []),
      ],
    };
  }

  function commandButton(): void {
    paletteOpen.set(true);
  }
</script>

<header class="titlebar" data-tauri-drag-region>
  <div class="brand" data-tauri-drag-region>
    <div class="brand-mark">N</div>
    <span>NoteCraft</span>
  </div>

  <div class="brand-divider"></div>

  <button type="button" class="vault-button" title="Vault options" on:click={openVaultMenu}>
    <span class="vault-name">{get(vaultRoot) ? baseName(get(vaultRoot)!) : "No vault"}</span>
    <ChevronDown size={11} />
  </button>

  <nav class="menubar" aria-label="Application menu">
    {#each names as name}
      <button type="button" class="menubar-item" on:click={(event) => openMenu(name, event)}>
        {name}
      </button>
    {/each}
  </nav>

  <div class="titlebar-spacer"></div>

  <div class="titlebar-actions">
    <button type="button" class="icon-btn" title="Command palette  Ctrl+K" aria-label="Command palette" on:click={commandButton}>
      <Command size={14} />
    </button>
    <button
      type="button"
      class="icon-btn"
      title="Search all documents"
      aria-label="Search all documents"
      on:click={() => document.dispatchEvent(new CustomEvent("notecraft:focus-search"))}
    >
      <Search size={14} />
    </button>
    <button
      type="button"
      class="icon-btn"
      class:active={$leftVisible}
      title="Toggle sidebar  Ctrl+B"
      aria-label="Toggle sidebar"
      on:click={() => {
        leftVisible.update((value) => !value);
        saveSession();
      }}
    >
      <PanelLeft size={14} />
    </button>
    <button
      type="button"
      class="icon-btn"
      class:active={$rightVisible}
      title="Toggle context panel  Ctrl+\"
      aria-label="Toggle context panel"
      on:click={() => {
        rightVisible.update((value) => !value);
        saveSession();
      }}
    >
      <PanelRight size={14} />
    </button>
    <button
      type="button"
      class="icon-btn"
      title={$settingsOpen ? "Close settings" : "Settings  Ctrl+,"}
      aria-label="Settings"
      on:click={() => settingsOpen.set(true)}
    >
      <Settings size={14} />
    </button>
    <button type="button" class="icon-btn" title="Toggle theme" aria-label="Toggle theme" on:click={() => void toggleTheme()}>
      {#if $settings.theme === "dark"}<Sun size={14} />{:else}<Moon size={14} />{/if}
    </button>
    <button type="button" class="icon-btn" title="Lock profile" aria-label="Lock profile" on:click={() => void lockProfile()}>
      <Lock size={13} />
    </button>
  </div>

  {#if tauri}
    <div class="window-controls">
      <button type="button" aria-label="Minimize" on:click={() => void windowAction("minimize")}><Minus size={14} /></button>
      <button type="button" aria-label={maximized ? "Restore" : "Maximize"} on:click={() => void windowAction("maximize")}>
        <Square size={11} />
      </button>
      <button type="button" class="close" aria-label="Close" on:click={() => void windowAction("close")}><X size={14} /></button>
    </div>
  {/if}
</header>

{#if menu}
  <Menu x={menu.x} y={menu.y} items={menu.items} onclose={() => (menu = null)} />
{/if}
