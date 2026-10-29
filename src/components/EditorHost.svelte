<script lang="ts">
  import {
    AlertTriangle,
    BookOpen,
    Check,
    Code2,
    Columns2,
    FileText,
    Loader2,
    MoreHorizontal,
    RotateCw,
    Sparkles,
  } from "lucide-svelte";
  import {
    activeTab,
    askConfirm,
    closeTab,
    pushToast,
    saveTab,
    statsFor,
    updateTab,
    type Tab,
  } from "../lib/state";
  import { capabilitiesFor } from "../lib/documentTypes";
  import { api } from "../lib/ipc";
  import { exportActive, saveActive, snapshotActive } from "../lib/actions";
  import CodeEditor from "./CodeEditor.svelte";
  import DocView from "./DocView.svelte";
  import Menu from "./Menu.svelte";
  import type { MenuItem } from "../lib/menu";

  let menu: { x: number; y: number; items: MenuItem[] } | null = null;
  let reloading = false;

  $: tab = $activeTab;
  $: caps = tab ? capabilitiesFor(tab.path) : null;
  $: prose = caps?.preview === "prose";

  const proseModes = [
    { mode: "source" as const, label: "Source", icon: Code2, hint: "Raw text" },
    { mode: "live" as const, label: "Live", icon: Sparkles, hint: "Edit with formatting" },
    { mode: "reading" as const, label: "Read", icon: BookOpen, hint: "Rendered document" },
    { mode: "split" as const, label: "Split", icon: Columns2, hint: "Editor and preview" },
  ];
  const codeModes = [
    { mode: "source" as const, label: "Source", icon: Code2, hint: "Editor" },
    { mode: "reading" as const, label: "View", icon: BookOpen, hint: "Structured view" },
    { mode: "split" as const, label: "Split", icon: Columns2, hint: "Editor and view" },
  ];

  function setMode(mode: Tab["mode"]): void {
    if (!tab) return;
    updateTab(tab.id, { mode });
  }

  async function reloadFromDisk(): Promise<void> {
    if (!tab) return;
    if (tab.dirty) {
      const ok = await askConfirm({
        title: "Reload from disk?",
        description: "Unsaved changes in this tab will be discarded and the file's current contents will be loaded.",
        confirmLabel: "Reload",
        danger: true,
      });
      if (!ok) return;
    }
    reloading = true;
    try {
      const content = await api.readFile(tab.path);
      updateTab(tab.id, {
        content,
        base: content,
        dirty: false,
        status: "idle",
        error: null,
        loadError: null,
        stats: statsFor(content, tab.path),
      });
      pushToast("success", "Reloaded from disk");
    } catch (problem) {
      pushToast("error", "Could not reload the file", String(problem));
    } finally {
      reloading = false;
    }
  }

  async function copyText(text: string, label: string): Promise<void> {
    try {
      await navigator.clipboard.writeText(text);
      pushToast("success", label);
    } catch {
      pushToast("error", "Clipboard is not available");
    }
  }

  function openMenu(event: MouseEvent): void {
    if (!tab) return;
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const items: MenuItem[] = [
      { label: "Save now", shortcut: "Ctrl S", action: () => void saveActive() },
      { label: "Create snapshot", action: () => void snapshotActive() },
      { separator: true },
      { label: "Copy relative path", action: () => void copyText(tab.path, "Path copied") },
      {
        label: "Copy rendered HTML",
        action: () =>
          void api
            .exportHtml(tab.path)
            .then((html) => copyText(html, "HTML copied to clipboard"))
            .catch((problem) => pushToast("error", "Could not render the document", String(problem))),
      },
      { separator: true },
      { label: "Export Markdown…", action: () => void exportActive("markdown") },
      { label: "Export HTML…", action: () => void exportActive("html") },
      { label: "Print / Save as PDF…", action: () => void exportActive("pdf") },
      { separator: true },
      { label: "Close tab", shortcut: "Ctrl W", action: () => void closeTab(tab.id) },
    ];
    menu = { x: rect.right - 220, y: rect.bottom + 5, items };
  }

  function statusLabel(state: Tab["status"]): string {
    if (state === "dirty") return "Unsaved";
    if (state === "saving") return "Saving";
    if (state === "saved") return "Saved";
    if (state === "error") return "Save failed";
    return "";
  }

  $: conflict = tab?.status === "error";
</script>

{#if tab}
  <section class="editor-area" style="flex-direction:column">
    <header class="editor-header">
      <span class="path" title={tab.path}>
        <FileText size={12} />
        <span>{tab.path}</span>
      </span>
      <span class="chip">
        {caps?.label}
        {#if caps && !caps.hasLinks}<span style="opacity:.55">·</span><span style="font-family:var(--nc-font-mono); font-size:9.5px">.{caps.extension}</span>{/if}
      </span>

      {#if statusLabel(tab.status)}
        <span class="save-chip" data-state={tab.status}>
          {#if tab.status === "saving"}
            <Loader2 size={10} class="spin" />
          {:else if tab.status === "saved"}
            <Check size={11} />
          {:else if tab.status === "error"}
            <AlertTriangle size={10} />
          {:else}
            <span style="width:5px;height:5px;border-radius:50%;background:currentColor"></span>
          {/if}
          {statusLabel(tab.status)}
        </span>
      {/if}

      <div style="flex:1"></div>

      <div class="segmented" role="group" aria-label="View mode">
        {#each prose ? proseModes : codeModes as option}
          <button
            type="button"
            class:on={tab.mode === option.mode}
            title={option.hint}
            on:click={() => setMode(option.mode)}
          >
            <svelte:component this={option.icon} size={13} />
            <span class="mode-label">{option.label}</span>
          </button>
        {/each}
      </div>

      <button type="button" class="icon-btn sm" title="More actions" aria-label="More actions" on:click={openMenu}>
        <MoreHorizontal size={15} />
      </button>
    </header>

    {#if conflict}
      <div class="conflict-bar">
        <AlertTriangle size={13} />
        <span style="min-width:0; flex-shrink:1">
          {tab.error ?? "This file could not be saved."}
        </span>
        <div class="spacer"></div>
        <button type="button" class="btn sm" on:click={() => void reloadFromDisk()} disabled={reloading}>
          {#if reloading}<Loader2 size={11} class="spin" />{/if} Reload from disk
        </button>
        <button type="button" class="btn sm primary" on:click={() => void saveTab(tab.id, true)}>
          Overwrite
        </button>
      </div>
    {/if}

    {#if tab.loading}
      <div class="editor-loading">
        <Loader2 size={16} class="spin" />
        Opening {tab.path}…
      </div>
    {:else if tab.loadError}
      <div class="editor-error">
        <AlertTriangle size={20} />
        <strong>Could not open this file</strong>
        <div class="hint">{tab.loadError}</div>
        <div class="empty-actions">
          <button type="button" class="btn sm" on:click={() => void reloadFromDisk()}>
            <RotateCw size={12} /> Try again
          </button>
          <button type="button" class="btn sm" on:click={() => void closeTab(tab.id)}>Close tab</button>
        </div>
      </div>
    {:else if tab.mode === "reading"}
      <DocView {tab} />
    {:else if tab.mode === "split"}
      <div class="split">
        <div class="editor-pane">
          <div class="editor-scroll">
            <CodeEditor {tab} active={true} />
          </div>
        </div>
        <div class="split-divider"></div>
        <div class="editor-pane" style="background: var(--nc-surface)">
          <DocView {tab} />
        </div>
      </div>
    {:else}
      <div class="editor-pane">
        <div class="editor-scroll">
          <CodeEditor {tab} active={true} />
        </div>
      </div>
    {/if}
  </section>
{/if}

{#if menu}
  <Menu x={menu.x} y={menu.y} items={menu.items} minWidth={220} onclose={() => (menu = null)} />
{/if}

<style>
  .mode-label {
    font-size: 10.5px;
  }
  @media (max-width: 1100px) {
    .mode-label {
      display: none;
    }
  }
</style>
