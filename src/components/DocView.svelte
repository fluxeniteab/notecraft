<script lang="ts">
  import type { Tab } from "../lib/state";
  import { capabilitiesFor } from "../lib/documentTypes";
  import Preview from "./Preview.svelte";
  import JsonView from "./JsonView.svelte";

  export let tab: Tab;
  $: caps = capabilitiesFor(tab.path);
</script>

{#if caps.kind === "json"}
  <JsonView {tab} />
{:else if caps.preview === "prose"}
  <Preview {tab} />
{:else}
  <div class="preview-scroll">
    <div class="preview-inner" style="padding-top:32px">
      <div class="panel-title" style="padding-left:0; margin-bottom:8px">
        {caps.label} source
        <span class="count">{tab.content.split("\n").length} lines</span>
      </div>
      <pre class="source-view">{tab.content}</pre>
    </div>
  </div>
{/if}

<style>
  .source-view {
    margin: 0;
    padding: 16px 18px;
    border: 1px solid var(--nc-border);
    border-radius: var(--nc-r-md);
    background: var(--nc-panel);
    color: var(--nc-fg-secondary);
    font-family: var(--nc-font-mono);
    font-size: 12.5px;
    line-height: 1.65;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    user-select: text;
  }
</style>
