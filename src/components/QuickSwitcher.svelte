<script lang="ts">
  import { onMount } from "svelte";
  import { Search } from "lucide-svelte";
  import { allFiles, openTab, recentNotes, switcherOpen } from "../lib/state";
  import { capabilitiesFor } from "../lib/documentTypes";

  let query = "";
  let selected = 0;
  let input: HTMLInputElement;

  $: files = $allFiles.filter((file) => !file.is_dir);
  $: matches = rank(files, query, $recentNotes);
  $: if (selected >= matches.length) selected = Math.max(0, matches.length - 1);

  let lastQuery = "";
  $: if (query !== lastQuery) {
    lastQuery = query;
    selected = 0;
  }

  onMount(() => {
    input?.focus();
    const onEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") switcherOpen.set(false);
    };
    window.addEventListener("keydown", onEscape);
    return () => window.removeEventListener("keydown", onEscape);
  });

  function score(path: string, needle: string): number {
    if (!needle) return 0;
    const lower = path.toLowerCase();
    const at = lower.indexOf(needle);
    if (at === 0) return 3;
    if (at > 0) return 2;
    // Subsequence match: "rc" finds "recent-notes.md".
    let cursor = 0;
    for (const char of needle) {
      cursor = lower.indexOf(char, cursor);
      if (cursor === -1) return -1;
      cursor += 1;
    }
    return 1;
  }

  function rank(
    entries: Array<{ path: string; name: string }>,
    raw: string,
    recents: string[]
  ): Array<{ path: string; name: string }> {
    const needle = raw.trim().toLowerCase();
    return entries
      .map((file) => {
        const byPath = score(file.path, needle);
        const byName = score(file.name, needle);
        const best = Math.max(byPath, byName);
        if (needle && best < 0) return null;
        const recent = recents.indexOf(file.path);
        return { file, score: best * 10 + (recent >= 0 ? 4 - Math.min(3, recent) : 0) };
      })
      .filter((entry): entry is { file: { path: string; name: string }; score: number } => !!entry)
      .sort((a, b) => b.score - a.score || a.file.name.localeCompare(b.file.name))
      .slice(0, 60)
      .map((entry) => entry.file);
  }

  function close(): void {
    switcherOpen.set(false);
  }

  function choose(path: string): void {
    close();
    void openTab(path);
  }

  function onKey(event: KeyboardEvent): void {
    if (event.key === "Escape") {
      event.preventDefault();
      close();
    } else if (event.key === "ArrowDown") {
      event.preventDefault();
      selected = Math.min(matches.length - 1, selected + 1);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      selected = Math.max(0, selected - 1);
    } else if (event.key === "Enter") {
      event.preventDefault();
      const match = matches[selected];
      if (match) choose(match.path);
    }
  }
</script>

<div class="overlay" role="presentation" on:click={(event) => event.target === event.currentTarget && close()}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div tabindex="-1" class="palette" role="dialog" aria-label="Quick switcher" on:click|stopPropagation>
    <div class="palette-input">
      <Search size={15} />
      <input
        bind:this={input}
        bind:value={query}
        placeholder="Go to a file…"
        spellcheck="false"
        aria-label="Find file"
        on:keydown={onKey}
      />
      <span class="kbd">Esc</span>
    </div>
    <div class="palette-list">
      {#each matches as file, index (file.path)}
        <button
          type="button"
          class="palette-item"
          class:selected={index === selected}
          on:click={() => choose(file.path)}
          on:mouseenter={() => (selected = index)}
        >
          <span style="font-family:var(--nc-font-mono); font-size:9px; color:var(--nc-fg-faint); width:22px; text-transform:uppercase">
            {capabilitiesFor(file.path).extension}
          </span>
          <span style="min-width:0; overflow:hidden; text-overflow:ellipsis; white-space:nowrap">{file.name}</span>
          <span class="spacer"></span>
          <span class="hint" style="max-width:45%; overflow:hidden; text-overflow:ellipsis; white-space:nowrap">{file.path}</span>
        </button>
      {/each}
      {#if matches.length === 0}
        <div class="panel-empty" style="padding:16px">No file matches “{query}”.</div>
      {/if}
    </div>
    <div class="palette-footer">
      <span class="k"><span class="kbd">↑</span><span class="kbd">↓</span> navigate</span>
      <span class="k"><span class="kbd">↵</span> open</span>
      <span>{files.length.toLocaleString()} files</span>
    </div>
  </div>
</div>
