<script lang="ts">
  import { AlertTriangle, Check, CloudOff, FileText } from "lucide-svelte";
  import { activeTab, allFiles, vaultRoot } from "../lib/state";
  import { baseName } from "../lib/files";
  import { capabilitiesFor } from "../lib/documentTypes";

  $: tab = $activeTab;
  $: caps = tab ? capabilitiesFor(tab.path) : null;
  $: stats = tab?.stats ?? null;
  $: status = tab?.status ?? "idle";
  $: eol = tab && tab.content.includes("\r\n") ? "CRLF" : "LF";
  $: documents = $allFiles.filter((file) => !file.is_dir && capabilitiesFor(file.path).indexable).length;
  $: statusLabel =
    status === "dirty"
      ? "Unsaved changes"
      : status === "saving"
        ? "Saving…"
        : status === "error"
          ? "Save failed"
          : status === "saved"
            ? "Saved"
            : "";
  $: valid = stats?.validation ?? null;
</script>

<footer class="statusbar">
  <span class="status-item" title={$vaultRoot ?? "No vault open"}>
    <span class="status-dot" class:off={!$vaultRoot}></span>
    {$vaultRoot ? baseName($vaultRoot) : "No vault"}
  </span>

  {#if $vaultRoot}
    <span class="status-item status-docs">
      <FileText size={10} />
      {documents.toLocaleString()} {documents === 1 ? "document" : "documents"}
    </span>
  {/if}

  {#if tab}
    <span class="status-item status-path" title={tab.path} style="max-width: 36vw; overflow:hidden">
      <span style="overflow:hidden; text-overflow:ellipsis; white-space:nowrap">{tab.path}</span>
      {#if statusLabel}
        <span style="opacity:.4">·</span>
        <span
          style="color:{status === 'error'
            ? 'var(--nc-danger)'
            : status === 'dirty'
              ? 'var(--nc-warning)'
              : 'var(--nc-fg-faint)'}"
        >{statusLabel}</span>
      {/if}
    </span>
  {/if}

  <span class="status-spacer"></span>

  {#if caps?.validator === "json" && valid}
    <span
      class="status-item"
      style="color:{valid.valid ? 'var(--nc-fg-faint)' : 'var(--nc-danger)'}"
      title={valid.valid
        ? "Valid JSON"
        : `Invalid JSON${valid.line ? ` at line ${valid.line}, column ${valid.column}` : ""}: ${valid.message ?? ""}`}
    >
      {#if valid.valid}<Check size={10} /> Valid JSON{:else}<AlertTriangle size={10} /> Invalid JSON{/if}
    </span>
  {:else if caps?.validator === "mdx"}
    <span class="status-item" title="MDX is parsed by the Rust pipeline; source is preserved exactly as typed.">
      MDX · source preserved
    </span>
  {/if}

  {#if tab && caps?.stats !== "data"}
    <span class="status-item status-cursor" style="font-variant-numeric:tabular-nums">
      Ln {tab.cursor.line}, Col {tab.cursor.col}
    </span>
  {/if}

  {#if stats}
    {#if caps?.stats === "prose"}
      <span class="status-item status-stats" style="font-variant-numeric:tabular-nums">
        {stats.words.toLocaleString()} words · {stats.chars.toLocaleString()} chars
      </span>
    {:else if caps?.stats === "data"}
      <span class="status-item status-stats" style="font-variant-numeric:tabular-nums">
        {stats.lines.toLocaleString()} lines · {stats.chars.toLocaleString()} chars
      </span>
    {:else}
      <span class="status-item status-stats" style="font-variant-numeric:tabular-nums">
        {stats.lines.toLocaleString()} lines
      </span>
    {/if}
  {/if}

  <span class="status-item status-local" title="Files stay on this machine; nothing is synced by default.">
    <CloudOff size={10} /> Local only
  </span>
  <span class="status-item status-encoding">{caps ? `UTF-8 · ${eol} · ${caps.label}` : "UTF-8"}</span>
</footer>
