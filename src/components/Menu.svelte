<script lang="ts">
  import { onMount } from "svelte";
  import type { MenuItem } from "../lib/menu";

  export let x = 0;
  export let y = 0;
  export let items: MenuItem[] = [];
  export let minWidth = 210;
  export let onclose: () => void = () => {};

  let el: HTMLDivElement | null = null;
  let left = x;
  let top = y;

  onMount(() => {
    if (el) {
      const rect = el.getBoundingClientRect();
      left = Math.max(8, Math.min(x, window.innerWidth - rect.width - 8));
      top = Math.max(8, Math.min(y, window.innerHeight - rect.height - 10));
    }
    const onPointer = (event: PointerEvent) => {
      if (el && !el.contains(event.target as Node)) onclose();
    };
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        onclose();
      }
    };
    window.addEventListener("pointerdown", onPointer);
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("pointerdown", onPointer);
      window.removeEventListener("keydown", onKey);
    };
  });

  function run(item: MenuItem): void {
    if (item.disabled || !item.action) return;
    onclose();
    item.action();
  }
</script>

<div class="menu" bind:this={el} style="left:{left}px; top:{top}px; min-width:{minWidth}px" role="menu">
  {#each items as item}
    {#if item.separator}
      <div class="menu-sep"></div>
    {:else if item.header}
      <div class="menu-header">{item.header}</div>
    {:else}
      <button
        type="button"
        class="menu-item"
        disabled={item.disabled}
        role="menuitem"
        on:click={() => run(item)}
      >
        <span class="label" style={item.danger ? "color: var(--nc-danger)" : ""}>{item.label}</span>
        {#if item.shortcut}<span class="shortcut">{item.shortcut}</span>{/if}
      </button>
    {/if}
  {/each}
</div>
