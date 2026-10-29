<script lang="ts">
  import { onMount } from "svelte";
  import { Search } from "lucide-svelte";
  import {
    activeTab,
    automationsOpen,
    canvasOpen,
    captureOpen,
    collectionsOpen,
    focusMode,
    graphOpen,
    leftVisible,
    paletteOpen,
    rightTab,
    rightVisible,
    saveSession,
    settingsOpen,
    shortcutsOpen,
    switcherOpen,
    updateTab,
  } from "../lib/state";
  import {
    closeVault,
    createDailyNote,
    createDocument,
    createFolder,
    createNamedDocument,
    exportActive,
    lockProfile,
    pickVault,
    saveActive,
    snapshotActive,
    toggleTheme,
  } from "../lib/actions";

  interface Command {
    group: string;
    label: string;
    hint?: string;
    run: () => void | Promise<void>;
  }

  let query = "";
  let selected = 0;
  let input: HTMLInputElement;

  const commands: Command[] = [
    { group: "Create", label: "New Markdown note", hint: ".md", run: () => createDocument("markdown") },
    { group: "Create", label: "New MDX document", hint: ".mdx", run: () => createDocument("mdx") },
    { group: "Create", label: "New JSON document", hint: ".json", run: () => createDocument("json") },
    { group: "Create", label: "New plain text file", hint: ".txt", run: () => createDocument("plain-text") },
    { group: "Create", label: "New file with a name…", hint: "choose extension", run: () => createNamedDocument("markdown") },
    { group: "Create", label: "New folder…", run: () => createFolder() },
    { group: "Create", label: "Quick capture…", hint: "Ctrl Shift N", run: () => captureOpen.set(true) },

    { group: "Navigate", label: "Go to file…", hint: "Ctrl P", run: () => switcherOpen.set(true) },
    { group: "Navigate", label: "Search all documents…", hint: "full text", run: () => { paletteOpen.set(false); document.dispatchEvent(new CustomEvent("notecraft:focus-search")); } },
    { group: "Navigate", label: "Open today's daily note", run: () => createDailyNote() },
    { group: "Navigate", label: "Open graph view", run: () => graphOpen.set(true) },
    { group: "Navigate", label: "Open collections", hint: "table · board · calendar", run: () => collectionsOpen.set(true) },
    { group: "Navigate", label: "Open automations", hint: "one-click rules", run: () => automationsOpen.set(true) },
    { group: "Navigate", label: "Open canvas", hint: "spatial notes", run: () => canvasOpen.set(true) },
    {
      group: "Navigate",
      label: "Open the assistant",
      hint: "AI panel",
      run: () => {
        rightVisible.set(true);
        rightTab.set("assistant");
        saveSession();
      },
    },

    { group: "Document", label: "Save current file", hint: "Ctrl S", run: () => saveActive() },
    { group: "Document", label: "Create a version snapshot", run: () => snapshotActive() },
    { group: "Document", label: "Export as HTML…", run: () => exportActive("html") },
    { group: "Document", label: "Print / Save as PDF…", run: () => exportActive("pdf") },
    {
      group: "Document",
      label: "Switch to reading view",
      run: () => {
        const tab = $activeTab;
        if (tab) updateTab(tab.id, { mode: "reading" });
      },
    },
    {
      group: "Document",
      label: "Switch to split view",
      run: () => {
        const tab = $activeTab;
        if (tab) updateTab(tab.id, { mode: "split" });
      },
    },

    { group: "Workspace", label: "Open a vault…", hint: "Ctrl O", run: () => pickVault("open") },
    { group: "Workspace", label: "Create a vault…", run: () => pickVault("create") },
    { group: "Workspace", label: "Close current vault", run: () => closeVault() },
    {
      group: "Workspace",
      label: "Toggle sidebar",
      hint: "Ctrl B",
      run: () => {
        leftVisible.update((value) => !value);
        saveSession();
      },
    },
    {
      group: "Workspace",
      label: "Toggle context panel",
      hint: "Ctrl \\",
      run: () => {
        rightVisible.update((value) => !value);
        saveSession();
      },
    },
    {
      group: "Workspace",
      label: "Toggle focus mode",
      hint: "Ctrl Shift F",
      run: () => {
        focusMode.update((value) => !value);
        saveSession();
      },
    },
    { group: "Workspace", label: "Toggle light / dark theme", hint: "Ctrl Shift D", run: () => toggleTheme() },
    { group: "Workspace", label: "Settings…", hint: "Ctrl ,", run: () => settingsOpen.set(true) },
    { group: "Workspace", label: "Keyboard shortcuts", run: () => shortcutsOpen.set(true) },
    { group: "Workspace", label: "Lock this profile", hint: "Ctrl Shift L", run: () => lockProfile() },
  ];

  $: filtered = query.trim()
    ? commands.filter((command) => command.label.toLowerCase().includes(query.trim().toLowerCase()) || command.group.toLowerCase().includes(query.trim().toLowerCase()))
    : commands;
  $: if (selected >= filtered.length) selected = Math.max(0, filtered.length - 1);

  let lastQuery = "";
  $: if (query !== lastQuery) {
    lastQuery = query;
    selected = 0;
  }

  onMount(() => {
    input?.focus();
    const onEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") paletteOpen.set(false);
    };
    window.addEventListener("keydown", onEscape);
    return () => window.removeEventListener("keydown", onEscape);
  });

  function close(): void {
    paletteOpen.set(false);
  }

  function run(command: Command): void {
    close();
    void command.run();
  }

  function onKey(event: KeyboardEvent): void {
    if (event.key === "Escape") {
      event.preventDefault();
      close();
    } else if (event.key === "ArrowDown") {
      event.preventDefault();
      selected = Math.min(filtered.length - 1, selected + 1);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      selected = Math.max(0, selected - 1);
    } else if (event.key === "Enter") {
      event.preventDefault();
      const command = filtered[selected];
      if (command) run(command);
    }
  }

  function onBackdrop(event: MouseEvent): void {
    if (event.target === event.currentTarget) close();
  }

  $: grouped = filtered.reduce<Array<{ group: string; items: Array<Command & { index: number }> }>>((acc, command, index) => {
    const bucket = acc.find((entry) => entry.group === command.group);
    if (bucket) bucket.items.push({ ...command, index });
    else acc.push({ group: command.group, items: [{ ...command, index }] });
    return acc;
  }, []);
</script>

<div class="overlay" role="presentation" on:click={onBackdrop}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div tabindex="-1" class="palette" role="dialog" aria-label="Command palette" on:click|stopPropagation>
    <div class="palette-input">
      <Search size={15} />
      <input
        bind:this={input}
        bind:value={query}
        placeholder="Type a command…"
        spellcheck="false"
        aria-label="Command"
        on:keydown={onKey}
      />
      <span class="kbd">Esc</span>
    </div>
    <div class="palette-list">
      {#each grouped as bucket (bucket.group)}
        <div class="palette-group">{bucket.group}</div>
        {#each bucket.items as command (command.index)}
          <button
            type="button"
            class="palette-item"
            class:selected={command.index === selected}
            on:click={() => run(command)}
            on:mouseenter={() => (selected = command.index)}
          >
            <span>{command.label}</span>
            <span class="spacer"></span>
            {#if command.hint}<span class="hint">{command.hint}</span>{/if}
          </button>
        {/each}
      {/each}
      {#if filtered.length === 0}
        <div class="panel-empty" style="padding:16px">No command matches “{query}”.</div>
      {/if}
    </div>
    <div class="palette-footer">
      <span class="k"><span class="kbd">↑</span><span class="kbd">↓</span> navigate</span>
      <span class="k"><span class="kbd">↵</span> run</span>
      <span class="k"><span class="kbd">Esc</span> dismiss</span>
    </div>
  </div>
</div>
