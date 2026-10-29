<script lang="ts">
  import { onMount } from "svelte";
  import { X } from "lucide-svelte";
  import { shortcutsOpen } from "../lib/state";

  const groups = [
    {
      name: "Files",
      rows: [
        ["Command palette", "Ctrl K"],
        ["Go to file", "Ctrl P"],
        ["New Markdown note", "Ctrl N"],
        ["Save", "Ctrl S"],
        ["Close tab", "Ctrl W"],
        ["Open vault", "Ctrl O"],
      ],
    },
    {
      name: "Writing",
      rows: [
        ["Find in file", "Ctrl F"],
        ["Next / previous tab", "Ctrl Tab"],
        ["Bold", "Format menu"],
        ["Link", "Insert menu"],
        ["Daily note", "Ctrl Alt N"],
        ["Quick capture", "Ctrl Shift N"],
      ],
    },
    {
      name: "Workspace",
      rows: [
        ["Toggle sidebar", "Ctrl B"],
        ["Toggle context panel", "Ctrl \\"],
        ["Focus mode", "Ctrl Shift F"],
        ["Toggle theme", "Ctrl Shift D"],
        ["Settings", "Ctrl ,"],
        ["Lock profile", "Ctrl Shift L"],
      ],
    },
  ];

  onMount(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") shortcutsOpen.set(false);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });
</script>

<div class="overlay center" role="presentation" on:click={(event) => event.target === event.currentTarget && shortcutsOpen.set(false)}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div tabindex="-1" class="dialog" role="dialog" aria-label="Keyboard shortcuts" on:click|stopPropagation style="width:min(620px, calc(100vw - 40px))">
    <div class="dialog-header">
      <div class="titles">
        <h3>Keyboard shortcuts</h3>
        <p>Everything important is reachable without the mouse.</p>
      </div>
      <button type="button" class="icon-btn" aria-label="Close" on:click={() => shortcutsOpen.set(false)}><X size={15} /></button>
    </div>
    <div class="dialog-body">
      <div style="display:grid; grid-template-columns:1fr 1fr; gap:26px">
        {#each groups as group}
          <div>
            <div class="field-label" style="margin-bottom:7px">{group.name}</div>
            <div class="shortcut-grid">
              {#each group.rows as [label, keys]}
                <div class="row"><span>{label}</span><span>{keys}</span></div>
              {/each}
            </div>
          </div>
        {/each}
      </div>
      <p class="setting-note" style="margin-top:16px">
        Menus also show their shortcuts. Bare-key shortcuts are deliberately not global so typing never triggers commands.
      </p>
    </div>
    <div class="dialog-footer">
      <span class="left"><span class="kbd">Esc</span> closes any overlay</span>
      <button type="button" class="btn primary" on:click={() => shortcutsOpen.set(false)}>Done</button>
    </div>
  </div>
</div>
