<script lang="ts">
  import { onMount, tick } from "svelte";
  import { get } from "svelte/store";
  import { FilePlus2, FolderOpen, Maximize2, Minimize2 } from "lucide-svelte";
  import {
    aboutOpen,
    activeTabId,
    allFiles,
    automationsOpen,
    booting,
    canvasOpen,
    captureOpen,
    closeTab,
    collectionsOpen,
    cycleTab,
    focusMode,
    graphOpen,
    leftVisible,
    leftWidth,
    modal,
    openTab,
    paletteOpen,
    profile,
    pushToast,
    recentNotes,
    recentVaults,
    rightVisible,
    rightWidth,
    saveSession,
    settings,
    settingsOpen,
    shortcutsOpen,
    switcherOpen,
    tabs,
    updateTab,
    vaultRoot,
    flushAllTabs,
  } from "./lib/state";
  import {
    applySettingsToDom,
    bootstrap,
    createDailyNote,
    createDocument,
    lockProfile,
    openVaultAt,
    pickVault,
    saveActive,
    toggleTheme,
  } from "./lib/actions";
  import { api } from "./lib/ipc";
  import { checkForUpdates } from "./lib/updater";
  import { baseName } from "./lib/files";
  import { isTauri } from "./lib/theme";
  import { openFind } from "./lib/editor";
  import TitleBar from "./components/TitleBar.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import TabBar from "./components/TabBar.svelte";
  import EditorHost from "./components/EditorHost.svelte";
  import RightPanel from "./components/RightPanel.svelte";
  import StatusBar from "./components/StatusBar.svelte";
  import CommandPalette from "./components/CommandPalette.svelte";
  import QuickSwitcher from "./components/QuickSwitcher.svelte";
  import SettingsDialog from "./components/SettingsDialog.svelte";
  import LockScreen from "./components/LockScreen.svelte";
  import GraphView from "./components/GraphView.svelte";
  import CollectionsView from "./components/CollectionsView.svelte";
  import AutomationsView from "./components/AutomationsView.svelte";
  import CanvasView from "./components/CanvasView.svelte";
  import QuickCapture from "./components/QuickCapture.svelte";
  import DialogHost from "./components/DialogHost.svelte";
  import Toasts from "./components/Toasts.svelte";
  import AboutDialog from "./components/AboutDialog.svelte";
  import ShortcutsDialog from "./components/ShortcutsDialog.svelte";

  let cleanup: Array<() => void> = [];
  let fsTimer = 0;
  let resizing = false;
  let winW = typeof window !== "undefined" ? window.innerWidth : 1280;
  $: compact = winW < 980;

  $: recentFiles = $recentNotes
    .map((path) => $allFiles.find((file) => file.path === path && !file.is_dir))
    .filter((file): file is NonNullable<typeof file> => !!file)
    .slice(0, 6);

  onMount(() => {
    const unsubscribe = settings.subscribe(() => applySettingsToDom());
    cleanup.push(unsubscribe);

    void (async () => {
      await bootstrap();
      booting.set(false);
      if (isTauri()) void wireWatchers();
      if (get(profile)) void checkForUpdates(true);
      document.documentElement.classList.toggle("reduced-motion", get(settings).reduced_motion);
      // Persist which tab is active only after the saved session is restored,
      // so startup cannot overwrite a real session with an empty one.
      cleanup.push(activeTabId.subscribe(() => saveSession()));
    })();

    const onSearchFocus = async () => {
      if (!get(vaultRoot)) {
        pushToast("info", "Open a vault to search its documents");
        return;
      }
      leftVisible.set(true);
      await tick();
      const input = document.getElementById("sidebar-search") as HTMLInputElement | null;
      input?.focus();
      input?.select();
    };
    const onOpenSwitcher = () => switcherOpen.set(true);
    const onCapture = () => {
      if (get(vaultRoot)) captureOpen.set(true);
      else pushToast("info", "Open a vault first");
    };
    document.addEventListener("notecraft:focus-search", onSearchFocus);
    document.addEventListener("notecraft:open-switcher", onOpenSwitcher);
    document.addEventListener("notecraft:capture", onCapture);
    cleanup.push(() => document.removeEventListener("notecraft:focus-search", onSearchFocus));
    cleanup.push(() => document.removeEventListener("notecraft:open-switcher", onOpenSwitcher));
    cleanup.push(() => document.removeEventListener("notecraft:capture", onCapture));

    return () => {
      cleanup.forEach((fn) => fn());
      cleanup = [];
    };
  });

  async function wireWatchers(): Promise<void> {
    try {
      const stopChange = await api.onVaultFsChange((paths) => scheduleExternal(paths));
      const stopWrite = await api.onVaultFileWritten((path) => scheduleExternal([path]));
      cleanup.push(stopChange, stopWrite);
    } catch (error) {
      console.error("Could not watch the vault", error);
    }

    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      const win = getCurrentWindow();
      const unlisten = await win.onCloseRequested(async (event) => {
        event.preventDefault();
        await flushAllTabs();
        const pending = get(tabs).some((tab) => tab.dirty || tab.status === "saving" || tab.status === "error");
        if (pending) {
          pushToast("error", "Some files are still unsaved", "Check the save indicator and close again.");
          return;
        }
        try {
          await win.destroy();
        } catch {
          await win.close();
        }
      });
      cleanup.push(unlisten);
    } catch {
      /* browser preview */
    }
  }

  function scheduleExternal(paths: string[]): void {
    window.clearTimeout(fsTimer);
    fsTimer = window.setTimeout(() => {
      void refreshFiles();
      for (const path of paths) void reloadExternal(path);
    }, 220);
  }

  async function refreshFiles(): Promise<void> {
    try {
      allFiles.set(await api.listAllFiles());
    } catch {
      /* the vault may have been removed */
    }
  }

  async function reloadExternal(path: string): Promise<void> {
    const tab = get(tabs).find((item) => item.path === path);
    if (!tab || tab.loading || tab.dirty) return;
    try {
      const content = await api.readFile(path);
      if (content !== tab.content) {
        updateTab(tab.id, {
          content,
          base: content,
          status: "idle",
          loadError: null,
          stats: tab.stats ? { ...tab.stats, chars: content.length, lines: content.split("\n").length } : tab.stats,
        });
      }
    } catch {
      updateTab(tab.id, { loadError: "This file no longer exists on disk." });
    }
  }

  function onKeyDown(event: KeyboardEvent): void {
    const mod = event.ctrlKey || event.metaKey;
    const key = event.key.toLowerCase();
    const overlayOpen =
      get(paletteOpen) ||
      get(switcherOpen) ||
      get(settingsOpen) ||
      get(graphOpen) ||
      get(captureOpen) ||
      get(shortcutsOpen) ||
      get(aboutOpen) ||
      !!get(modal);

    if (overlayOpen) return;

    if (!mod) {
      if (event.key === "Escape" && get(focusMode)) {
        event.preventDefault();
        focusMode.set(false);
        saveSession();
      }
      return;
    }

    switch (key) {
      case "k":
        event.preventDefault();
        switcherOpen.set(false);
        paletteOpen.update((value) => !value);
        return;
      case "p":
        event.preventDefault();
        paletteOpen.set(false);
        switcherOpen.update((value) => !value);
        return;
      case "s":
        event.preventDefault();
        void saveActive();
        return;
      case "n":
        event.preventDefault();
        if (event.shiftKey) {
          if (get(vaultRoot)) captureOpen.set(true);
        } else if (event.altKey) {
          void createDailyNote();
        } else if (get(vaultRoot)) {
          void createDocument("markdown");
        }
        return;
      case "w": {
        event.preventDefault();
        const id = get(activeTabId);
        if (id) void closeTab(id);
        return;
      }
      case "b":
        event.preventDefault();
        leftVisible.update((value) => !value);
        saveSession();
        return;
      case "\\":
        event.preventDefault();
        rightVisible.update((value) => !value);
        saveSession();
        return;
      case ",":
        event.preventDefault();
        settingsOpen.set(true);
        return;
      case "o":
        event.preventDefault();
        void pickVault("open");
        return;
      case "f":
        event.preventDefault();
        if (event.shiftKey) {
          focusMode.update((value) => !value);
          saveSession();
        } else {
          openFind();
        }
        return;
      case "d":
        if (event.shiftKey) {
          event.preventDefault();
          void toggleTheme();
        }
        return;
      case "l":
        if (event.shiftKey) {
          event.preventDefault();
          void lockProfile();
        }
        return;
      case "tab":
        event.preventDefault();
        cycleTab(event.shiftKey ? -1 : 1);
        return;
      default:
        return;
    }
  }

  function startResize(side: "left" | "right", event: PointerEvent): void {
    event.preventDefault();
    resizing = true;
    const startX = event.clientX;
    const startWidth = side === "left" ? get(leftWidth) : get(rightWidth);
    const move = (moveEvent: PointerEvent) => {
      const delta = moveEvent.clientX - startX;
      if (side === "left") {
        leftWidth.set(Math.max(230, Math.min(440, startWidth + delta)));
      } else {
        rightWidth.set(Math.max(250, Math.min(470, startWidth - delta)));
      }
    };
    const up = () => {
      resizing = false;
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
      saveSession();
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  }

  function exitFocus(): void {
    focusMode.set(false);
    saveSession();
  }

  $: showShell = !$focusMode;
</script>

<svelte:window on:keydown={onKeyDown} on:resize={() => (winW = window.innerWidth)} />

{#if $booting}
  <div class="boot">
    <div class="brand-mark">N</div>
    <span>Opening NoteCraft…</span>
  </div>
{:else if !$profile}
  <LockScreen />
{:else}
  <div class="app" class:resizing>
    {#if showShell}
      <TitleBar />
    {/if}

    <div class="app-body">
      {#if showShell && $leftVisible}
        <div style="width:{$leftWidth}px; display:flex; flex:0 0 auto; min-height:0">
          <Sidebar />
        </div>
        <div
          class="resize-handle"
          role="separator"
          aria-label="Resize sidebar"
          on:pointerdown={(event) => startResize("left", event)}
        ></div>
      {/if}

      <main class="main">
        {#if showShell && $tabs.length > 0}
          <TabBar />
        {/if}

        {#if $tabs.length === 0}
          <div class="empty">
            <div class="empty-mark">N</div>
            <h2>{$vaultRoot ? "A quiet place to think" : "Open a vault to begin"}</h2>
            <p>
              {#if $vaultRoot}
                Create a Markdown, MDX, JSON or source file. Everything is saved as a plain file in
                <strong>{baseName($vaultRoot)}</strong>.
              {:else}
                Choose a folder on this machine. NoteCraft reads and writes ordinary files and never
                uploads them anywhere.
              {/if}
            </p>
            <div class="empty-actions">
              {#if $vaultRoot}
                <button type="button" class="btn primary" on:click={() => void createDocument("markdown")}>
                  <FilePlus2 size={13} /> New note
                </button>
                <button type="button" class="btn" on:click={() => paletteOpen.set(true)}>
                  <Maximize2 size={13} /> Command palette
                </button>
              {:else}
                <button type="button" class="btn primary" on:click={() => void pickVault("open")}>
                  <FolderOpen size={13} /> Open vault
                </button>
                <button type="button" class="btn" on:click={() => void pickVault("create")}>Create vault</button>
              {/if}
            </div>

            {#if $vaultRoot && recentFiles.length}
              <div class="empty-recents">
                <div class="head">Recent</div>
                {#each recentFiles as file}
                  <button type="button" class="panel-row" style="padding-left:8px" on:click={() => void openTab(file.path)}>
                    <span class="truncate">{file.name}</span>
                    <span class="spacer"></span>
                    <span class="meta truncate" style="max-width:50%">{file.path}</span>
                  </button>
                {/each}
              </div>
            {:else if !$vaultRoot && $recentVaults.length}
              <div class="empty-recents">
                <div class="head">Recent vaults</div>
                {#each $recentVaults.slice(0, 5) as path}
                  <button type="button" class="panel-row" style="padding-left:8px" on:click={() => void openVaultAt(path)}>
                    <FolderOpen size={12} style="color:var(--nc-fg-faint)" />
                    <span class="truncate">{baseName(path)}</span>
                    <span class="spacer"></span>
                    <span class="meta truncate" style="max-width:55%">{path}</span>
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        {:else}
          <EditorHost />
        {/if}

        {#if showShell}
          <StatusBar />
        {/if}
      </main>

      {#if showShell && $rightVisible && !compact}
        <div
          class="resize-handle"
          role="separator"
          aria-label="Resize context panel"
          on:pointerdown={(event) => startResize("right", event)}
        ></div>
        <div style="width:{$rightWidth}px; display:flex; flex:0 0 auto; min-height:0">
          <RightPanel />
        </div>
      {/if}
    </div>

    {#if $focusMode}
      <button type="button" class="focus-exit" on:click={exitFocus}>
        <Minimize2 size={12} /> Exit focus mode <span class="kbd">Esc</span>
      </button>
    {/if}
  </div>

  {#if $paletteOpen}
    <CommandPalette />
  {/if}
  {#if $switcherOpen}
    <QuickSwitcher />
  {/if}
  {#if $settingsOpen}
    <SettingsDialog />
  {/if}
  {#if $graphOpen}
    <GraphView />
  {/if}
  {#if $collectionsOpen}
    <CollectionsView />
  {/if}
  {#if $automationsOpen}
    <AutomationsView />
  {/if}
  {#if $canvasOpen}
    <CanvasView />
  {/if}
  {#if $captureOpen}
    <QuickCapture />
  {/if}
  {#if $shortcutsOpen}
    <ShortcutsDialog />
  {/if}
  {#if $aboutOpen}
    <AboutDialog />
  {/if}
  <DialogHost />
  <Toasts />
{/if}

<style>
  .app.resizing {
    cursor: col-resize;
    user-select: none;
  }
  .app.resizing :global(*) {
    cursor: col-resize !important;
  }
</style>
