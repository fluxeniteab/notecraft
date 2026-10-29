<script lang="ts">
  import { onMount, tick } from "svelte";
  import { X } from "lucide-svelte";
  import { closeModal, modal } from "../lib/state";

  let value = "";
  let input: HTMLInputElement;

  $: if ($modal) {
    value = typeof $modal.value === "string" ? $modal.value : "";
    void tick().then(() => input?.select());
  }

  function cancel(): void {
    closeModal($modal?.kind === "prompt" ? null : false);
  }

  function accept(): void {
    if (!$modal) return;
    closeModal($modal.kind === "prompt" ? value : true);
  }

  onMount(() => {
    const onEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape" && $modal) {
        event.preventDefault();
        cancel();
      }
    };
    window.addEventListener("keydown", onEscape);
    return () => window.removeEventListener("keydown", onEscape);
  });
</script>

{#if $modal}
  <div class="overlay center" role="presentation" on:click={(event) => event.target === event.currentTarget && cancel()}>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div tabindex="-1" class="dialog" role="alertdialog" aria-label={$modal.title} on:click|stopPropagation>
      <div class="dialog-header">
        <div class="titles">
          <h3>{$modal.title}</h3>
          {#if $modal.description}<p>{$modal.description}</p>{/if}
        </div>
        <button type="button" class="icon-btn" aria-label="Close" on:click={cancel}><X size={15} /></button>
      </div>
      <form
        on:submit|preventDefault={() => {
          if ($modal?.kind === "prompt" && !value.trim()) return;
          accept();
        }}
      >
        <div class="dialog-body">
          {#if $modal.kind === "prompt"}
            <div class="field">
              {#if $modal.label}<label for="modal-input">{$modal.label}</label>{/if}
              <input
                id="modal-input"
                class="input"
                bind:this={input}
                bind:value={value}
                placeholder={$modal.placeholder ?? ""}
                spellcheck="false"
              />
            </div>
          {/if}
        </div>
        <div class="dialog-footer">
          <button type="button" class="btn" on:click={cancel}>Cancel</button>
          <button
            type="submit"
            class="btn"
            class:primary={!$modal.danger}
            class:danger={$modal.danger}
            disabled={$modal.kind === "prompt" && !value.trim()}
          >{$modal.confirmLabel ?? "Confirm"}</button>
        </div>
      </form>
    </div>
  </div>
{/if}
