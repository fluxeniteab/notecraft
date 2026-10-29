<script lang="ts">
  import { onMount } from "svelte";
  import { CornerDownLeft, X } from "lucide-svelte";
  import { captureOpen, settings } from "../lib/state";
  import { appendCapture } from "../lib/actions";

  let text = "";
  let textarea: HTMLTextAreaElement;
  let busy = false;

  onMount(() => {
    textarea?.focus();
    const onEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") close();
    };
    window.addEventListener("keydown", onEscape);
    return () => window.removeEventListener("keydown", onEscape);
  });

  function close(): void {
    captureOpen.set(false);
    text = "";
  }

  async function submit(): Promise<void> {
    const value = text.trim();
    if (!value || busy) return;
    busy = true;
    try {
      await appendCapture(value);
      close();
    } finally {
      busy = false;
    }
  }

  function onKey(event: KeyboardEvent): void {
    if (event.key === "Escape") {
      event.preventDefault();
      close();
    } else if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      void submit();
    }
  }
</script>

<div class="overlay center" role="presentation" on:click={(event) => event.target === event.currentTarget && close()}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div tabindex="-1" class="dialog" role="dialog" aria-label="Quick capture" on:click|stopPropagation style="width:min(500px, calc(100vw - 40px))">
    <div class="dialog-header">
      <div class="titles">
        <h3>Quick capture</h3>
        <p>Appended to <strong>{$settings.capture_note}</strong> without leaving what you are doing.</p>
      </div>
      <button type="button" class="icon-btn" aria-label="Close" on:click={close}><X size={15} /></button>
    </div>
    <div class="dialog-body">
      <textarea
        class="textarea"
        bind:this={textarea}
        bind:value={text}
        placeholder="A thought, a link, a task…"
        style="min-height:130px"
        spellcheck="true"
        on:keydown={onKey}
      ></textarea>
    </div>
    <div class="dialog-footer">
      <span class="left"><span class="kbd">Ctrl</span> <span class="kbd">↵</span> to capture</span>
      <button type="button" class="btn" on:click={close}>Cancel</button>
      <button type="button" class="btn primary" disabled={!text.trim() || busy} on:click={() => void submit()}>
        <CornerDownLeft size={12} /> Capture
      </button>
    </div>
  </div>
</div>
