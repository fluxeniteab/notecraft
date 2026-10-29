<script lang="ts">
  import { Plus, X } from "lucide-svelte";
  import { activeTabId, closeTab, tabs } from "../lib/state";
  import { createDocument, createNamedDocument } from "../lib/actions";
  import { DOCUMENT_KINDS, capabilitiesFor, capabilitiesOf } from "../lib/documentTypes";
  import Menu from "./Menu.svelte";
  import type { MenuItem } from "../lib/menu";

  let dragIndex: number | null = null;
  let newMenu: { x: number; y: number; items: MenuItem[] } | null = null;

  function activate(id: string): void {
    activeTabId.set(id);
  }

  function reorder(from: number, to: number): void {
    if (from === to) return;
    tabs.update((list) => {
      const next = [...list];
      const [moved] = next.splice(from, 1);
      if (moved) next.splice(to, 0, moved);
      return next;
    });
  }

  function openNewMenu(event: MouseEvent): void {
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    newMenu = {
      x: rect.left - 170,
      y: rect.bottom + 5,
      items: [
        { header: "New document" },
        ...DOCUMENT_KINDS.map((kind) => ({
          label: capabilitiesOf(kind).label,
          shortcut: `.${capabilitiesOf(kind).extension}`,
          action: () => void createDocument(kind),
        })),
        { separator: true },
        { label: "Choose name and type…", action: () => void createNamedDocument("markdown") },
      ],
    };
  }

</script>

<div class="tabstrip" role="tablist" aria-label="Open documents">
  {#each $tabs as tab, index (tab.id)}
    <div
      class="tab"
      class:active={tab.id === $activeTabId}
      role="tab"
      tabindex="0"
      aria-selected={tab.id === $activeTabId}
      title={tab.path}
      draggable="true"
      on:click={() => activate(tab.id)}
      on:keydown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          activate(tab.id);
        }
      }}
      on:auxclick={(event) => {
        if (event.button === 1) {
          event.preventDefault();
          void closeTab(tab.id);
        }
      }}
      on:dragstart={() => (dragIndex = index)}
      on:dragover|preventDefault={() => {}}
      on:drop|preventDefault={() => {
        if (dragIndex !== null) reorder(dragIndex, index);
        dragIndex = null;
      }}
    >
      <span class="tab-ext">{capabilitiesFor(tab.path).extension}</span>
      <span class="title">{tab.title}</span>
      {#if tab.dirty}<span class="dot" title="Unsaved changes"></span>{/if}
      <button
        type="button"
        class="close"
        aria-label={`Close ${tab.title}`}
        on:click|stopPropagation={() => void closeTab(tab.id)}
      >
        <X size={11} />
      </button>
    </div>
  {/each}

  <button type="button" class="tab-add" aria-label="New document" title="New document" on:click={openNewMenu}>
    <Plus size={14} />
  </button>
</div>

{#if newMenu}
  <Menu x={newMenu.x} y={newMenu.y} items={newMenu.items} onclose={() => (newMenu = null)} />
{/if}

