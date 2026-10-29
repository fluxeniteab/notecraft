<script lang="ts">
  import { AlertTriangle, CheckCircle2, Info, X } from "lucide-svelte";
  import { dismissToast, toasts } from "../lib/state";
</script>

<div class="toasts" aria-live="polite">
  {#each $toasts as toast (toast.id)}
    <div class="toast {toast.kind}" role="status">
      <span class="accent">
        {#if toast.kind === "success"}
          <CheckCircle2 size={14} />
        {:else if toast.kind === "error"}
          <AlertTriangle size={14} />
        {:else}
          <Info size={14} />
        {/if}
      </span>
      <div class="body">
        <div class="title">{toast.title}</div>
        {#if toast.description}<div class="desc">{toast.description}</div>{/if}
      </div>
      <button type="button" class="icon-btn sm" aria-label="Dismiss" on:click={() => dismissToast(toast.id)}>
        <X size={12} />
      </button>
    </div>
  {/each}
</div>
