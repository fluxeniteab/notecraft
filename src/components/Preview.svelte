<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import type { Tab } from "../lib/state";
  import { allFiles, openTab, pushToast } from "../lib/state";
  import { api } from "../lib/ipc";
  import { resolveNotePath } from "../lib/files";
  import { hasDocumentExtension } from "../lib/documentTypes";
  import { isTauri } from "../lib/theme";

  export let tab: Tab;

  let html = "";
  let error = "";
  let loading = true;
  let rendered = "";
  let timer = 0;

  $: schedule(tab.path, tab.content);

  function schedule(path: string, content: string): void {
    const key = `${path}\u0000${content}`;
    if (key === rendered) return;
    rendered = key;
    window.clearTimeout(timer);
    timer = window.setTimeout(() => void render(path, content), 220);
  }

  async function render(path: string, content: string): Promise<void> {
    loading = true;
    try {
      html = await api.renderDocument(content, path);
      error = "";
    } catch (problem) {
      error = String(problem);
      html = "";
    } finally {
      loading = false;
    }
  }

  function onPreviewClick(event: MouseEvent): void {
    const anchor = (event.target as HTMLElement | null)?.closest("a");
    if (!anchor) return;
    const href = anchor.getAttribute("href");
    if (!href) return;
    event.preventDefault();

    if (/^(https?:|mailto:)/i.test(href)) {
      if (isTauri()) void openUrl(href).catch(() => {});
      else window.open(href, "_blank", "noopener");
      return;
    }
    if (href.startsWith("#")) {
      const id = decodeURIComponent(href.slice(1));
      document.getElementById(id)?.scrollIntoView({ behavior: "smooth", block: "start" });
      return;
    }

    const clean = decodeURIComponent(href.split("#")[0]);
    const direct = $allFiles.find((file) => !file.is_dir && file.path.toLowerCase() === clean.toLowerCase());
    if (direct) {
      void openTab(direct.path);
      return;
    }
    if (hasDocumentExtension(clean)) {
      pushToast("info", "That file is not in this vault", clean);
      return;
    }
    const resolved = resolveNotePath(clean, $allFiles, tab.path);
    if (resolved) void openTab(resolved);
    else pushToast("info", "Link target not found", clean);
  }
</script>

<div class="preview-scroll">
  <div class="preview-inner">
    {#if error}
      <div class="json-invalid">
        <strong>This document could not be rendered.</strong>
        <div style="margin-top:6px">{error}</div>
        <div style="margin-top:8px; color: var(--nc-fg-muted)">The source is untouched. Switch to Source to fix the syntax.</div>
      </div>
    {:else if loading && !html}
      <div class="editor-loading">Rendering…</div>
    {:else}
      <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_noninteractive_element_interactions -->
      <article class="prose" on:click={onPreviewClick}>
        <!-- HTML is produced and escaped by the Rust renderer; raw script tags are not emitted. -->
        {@html html}
      </article>
    {/if}
  </div>
</div>
