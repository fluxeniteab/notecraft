<script lang="ts">
  import { onMount } from "svelte";
  import { X } from "lucide-svelte";
  import { aboutOpen, settings } from "../lib/state";

  onMount(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") aboutOpen.set(false);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });
</script>

<div class="overlay center" role="presentation" on:click={(event) => event.target === event.currentTarget && aboutOpen.set(false)}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div tabindex="-1" class="dialog" role="dialog" aria-label="About NoteCraft" on:click|stopPropagation style="width:min(480px, calc(100vw - 40px))">
    <div class="dialog-header">
      <div class="brand-mark" style="width:34px;height:34px;font-size:22px;border-radius:9px">N</div>
      <div class="titles">
        <h3>NoteCraft</h3>
        <p>Version 0.4.0 · local-first knowledge management</p>
      </div>
      <button type="button" class="icon-btn" aria-label="Close" on:click={() => aboutOpen.set(false)}><X size={15} /></button>
    </div>
    <div class="dialog-body">
      <p style="margin:0; color:var(--nc-fg-secondary); font-size:12px; line-height:1.75">
        A quiet desktop writing app for Markdown, MDX, JSON and source files. Notes live as ordinary
        files in folders you choose; profiles are encrypted locally; nothing is uploaded.
      </p>
      <div class="divider"></div>
      <div class="prop-row" style="padding-left:0; padding-right:0">
        <span class="prop-key">Interface</span><span class="prop-value">Svelte 5 · TypeScript</span>
      </div>
      <div class="prop-row" style="padding-left:0; padding-right:0">
        <span class="prop-key">Runtime</span><span class="prop-value">Tauri 2 · Rust</span>
      </div>
      <div class="prop-row" style="padding-left:0; padding-right:0">
        <span class="prop-key">Editor</span><span class="prop-value">CodeMirror 6</span>
      </div>
      <div class="prop-row" style="padding-left:0; padding-right:0">
        <span class="prop-key">Data</span>
        <span class="prop-value" style="font-size:10.5px">{$settings.data_dir ?? "Default app data folder"}</span>
      </div>
      <div class="prop-row" style="padding-left:0; padding-right:0">
        <span class="prop-key">License</span><span class="prop-value">GPL-3.0-only</span>
      </div>
    </div>
    <div class="dialog-footer">
      <span class="left">© 2026 Fluxenite</span>
      <button type="button" class="btn primary" on:click={() => aboutOpen.set(false)}>Close</button>
    </div>
  </div>
</div>
