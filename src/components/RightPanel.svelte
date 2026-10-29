<script lang="ts">
  import {
    Camera,
    CheckSquare,
    ChevronRight,
    History,
    Info,
    Link2,
    List,
    Loader2,
    RotateCcw,
  } from "lucide-svelte";
  import type { ParsedNote, Snapshot, TaskItem } from "../lib/types";
  import {
    activeTab,
    askConfirm,
    openTab,
    pushToast,
    rightTab,
    updateTab,
  } from "../lib/state";
  import { api } from "../lib/ipc";
  import { goToLine } from "../lib/editor";
  import { baseName, formatRelativeTime } from "../lib/files";
  import { runSearch, snapshotActive } from "../lib/actions";

  let parsed: ParsedNote | null = null;
  let backlinks: string[] = [];
  let mentions: Array<{ path: string; line: number; text: string }> = [];
  let tasks: TaskItem[] = [];
  let snapshots: Snapshot[] = [];
  let loading = false;
  let parseTimer = 0;
  let parsedKey = "";
  let backlinkKey = "";

  $: tab = $activeTab;
  $: if (tab) scheduleParse(tab.path, tab.content);
  $: if (tab?.path !== backlinkKey) {
    backlinkKey = tab?.path ?? "";
    void loadBacklinks(tab?.path ?? null);
  }
  $: if ($rightTab === "tasks") void loadTasks();
  $: if ($rightTab === "history" && tab) void loadSnapshots(tab.path);
  $: properties = parsed ? Object.entries(parsed.frontmatter.data ?? {}) : [];

  function scheduleParse(path: string, content: string): void {
    const key = path + "\u0000" + content;
    if (key === parsedKey) return;
    parsedKey = key;
    window.clearTimeout(parseTimer);
    parseTimer = window.setTimeout(async () => {
      try {
        parsed = await api.parseNote(content);
      } catch {
        parsed = null;
      }
    }, 320);
  }

  async function loadBacklinks(path: string | null): Promise<void> {
    if (!path) {
      backlinks = [];
      mentions = [];
      return;
    }
    try {
      const [backs, unlinked] = await Promise.all([
        api.backlinks(path).catch(() => []),
        api.unlinkedMentions(path).catch(() => []),
      ]);
      backlinks = backs;
      mentions = unlinked;
    } catch {
      backlinks = [];
      mentions = [];
    }
  }

  async function loadTasks(): Promise<void> {
    loading = true;
    try {
      tasks = await api.listTasks();
    } catch (problem) {
      pushToast("error", "Could not load tasks", String(problem));
    } finally {
      loading = false;
    }
  }

  async function loadSnapshots(path: string): Promise<void> {
    try {
      snapshots = await api.listSnapshots(path);
    } catch {
      snapshots = [];
    }
  }

  function jump(line: number): void {
    if (!tab) return;
    if (tab.mode === "reading") updateTab(tab.id, { mode: "source" });
    window.setTimeout(() => goToLine(line), 90);
  }

  async function toggleTask(task: TaskItem, done: boolean): Promise<void> {
    try {
      await api.setTaskDone(task.note_path, task.line, done);
      await loadTasks();
      await reloadIfOpen(task.note_path);
    } catch (problem) {
      pushToast("error", "Could not update the task", String(problem));
    }
  }

  async function reloadIfOpen(path: string): Promise<void> {
    if (!tab || tab.path !== path || tab.dirty) return;
    const content = await api.readFile(path);
    updateTab(tab.id, { content, base: content });
  }

  async function restore(snapshot: Snapshot): Promise<void> {
    const ok = await askConfirm({
      title: "Restore this version?",
      description: `${formatRelativeTime(snapshot.created_at)} · ${snapshot.size.toLocaleString()} bytes. A new snapshot of the current file is taken first.`,
      confirmLabel: "Restore",
      danger: true,
    });
    if (!ok) return;
    try {
      await api.restoreSnapshot(snapshot.id);
      if (tab) {
        const content = await api.readFile(tab.path);
        updateTab(tab.id, { content, base: content, dirty: false, status: "idle" });
      }
      await loadSnapshots(tab?.path ?? "");
      pushToast("success", "Version restored");
    } catch (problem) {
      pushToast("error", "Restore failed", String(problem));
    }
  }

  function displayValue(value: unknown): string {
    if (value === null || value === undefined) return "—";
    if (typeof value === "object") return JSON.stringify(value);
    return String(value);
  }
</script>

<aside class="rightbar">
  <div class="rightbar-tabs" role="tablist" aria-label="Document panels">
    <button type="button" class="rightbar-tab" class:on={$rightTab === "outline"} title="Outline" on:click={() => rightTab.set("outline")}>
      <List size={14} />
    </button>
    <button type="button" class="rightbar-tab" class:on={$rightTab === "backlinks"} title="Backlinks and mentions" on:click={() => rightTab.set("backlinks")}>
      <Link2 size={14} />
    </button>
    <button type="button" class="rightbar-tab" class:on={$rightTab === "tags"} title="Properties and tags" on:click={() => rightTab.set("tags")}>
      <Info size={14} />
    </button>
    <button type="button" class="rightbar-tab" class:on={$rightTab === "tasks"} title="Tasks" on:click={() => rightTab.set("tasks")}>
      <CheckSquare size={14} />
    </button>
    <button type="button" class="rightbar-tab" class:on={$rightTab === "history"} title="Version history" on:click={() => rightTab.set("history")}>
      <History size={14} />
    </button>
  </div>

  <div class="rightbar-scroll">
    {#if !tab}
      <div class="panel-empty" style="padding-top:18px">Open a document to see its outline, links and properties.</div>
    {:else if $rightTab === "outline"}
      <div class="panel-section">
        <div class="panel-title">
          Outline
          <span class="count">{parsed?.outline.length ?? 0}</span>
        </div>
        {#if !parsed}
          <div class="panel-empty">Reading document…</div>
        {:else if parsed.outline.length === 0}
          <div class="panel-empty">No headings yet. A <code>#</code> heading becomes an outline entry.</div>
        {:else}
          {#each parsed.outline as item}
            <button
              type="button"
              class="outline-item"
              style="padding-left:{13 + (item.level - 1) * 11}px"
              title={item.text}
              on:click={() => jump(item.line)}
            >
              <ChevronRight size={10} style="color: var(--nc-fg-faint)" />
              <span>{item.text}</span>
            </button>
          {/each}
        {/if}
      </div>
      <div class="panel-section">
        <div class="panel-title">Document</div>
        <div class="prop-row"><span class="prop-key">Format</span><span class="prop-value">{baseName(tab.path)}</span></div>
        <div class="prop-row"><span class="prop-key">Words</span><span class="prop-value">{tab.stats?.words.toLocaleString() ?? "—"}</span></div>
        <div class="prop-row"><span class="prop-key">Characters</span><span class="prop-value">{tab.stats?.chars.toLocaleString() ?? "—"}</span></div>
        <div class="prop-row"><span class="prop-key">Lines</span><span class="prop-value">{tab.stats?.lines.toLocaleString() ?? "—"}</span></div>
      </div>

    {:else if $rightTab === "backlinks"}
      <div class="panel-section">
        <div class="panel-title">Linked from <span class="count">{backlinks.length}</span></div>
        {#if backlinks.length === 0}
          <div class="panel-empty">Nothing links here yet. Type <code>[[{baseName(tab.path)}]]</code> from another note.</div>
        {:else}
          {#each backlinks as path}
            <button type="button" class="panel-row" title={path} on:click={() => void openTab(path)}>
              <Link2 size={11} style="color: var(--nc-fg-faint)" />
              <span class="truncate">{path}</span>
            </button>
          {/each}
        {/if}
      </div>
      <div class="panel-section">
        <div class="panel-title">Unlinked mentions <span class="count">{mentions.length}</span></div>
        {#if mentions.length === 0}
          <div class="panel-empty">No unlinked mentions.</div>
        {:else}
          {#each mentions.slice(0, 20) as mention}
            <button type="button" class="panel-row" title={mention.path} on:click={() => void openTab(mention.path)}>
              <div style="min-width:0">
                <div class="truncate" style="color: var(--nc-fg)">{baseName(mention.path)}</div>
                <div class="truncate meta">{mention.text}</div>
              </div>
            </button>
          {/each}
        {/if}
      </div>

    {:else if $rightTab === "tags"}
      <div class="panel-section">
        <div class="panel-title">Properties <span class="count">{properties.length}</span></div>
        {#if properties.length === 0}
          <div class="panel-empty">No frontmatter. Add <code>---</code> at the top of the note to define properties.</div>
        {:else}
          {#each properties as [key, value]}
            <div class="prop-row"><span class="prop-key">{key}</span><span class="prop-value">{displayValue(value)}</span></div>
          {/each}
        {/if}
      </div>
      <div class="panel-section">
        <div class="panel-title">Tags <span class="count">{parsed?.tags.length ?? 0}</span></div>
        {#if !parsed || parsed.tags.length === 0}
          <div class="panel-empty">No tags in this note.</div>
        {:else}
          <div class="tag-cloud">
            {#each parsed.tags as tag}
              <button type="button" class="tag-pill" title="Search for #{tag}" on:click={() => void runSearch(tag)}>#{tag}</button>
            {/each}
          </div>
        {/if}
      </div>
      <div class="panel-section">
        <div class="panel-title">Links <span class="count">{parsed?.links.length ?? 0}</span></div>
        {#if !parsed || parsed.links.length === 0}
          <div class="panel-empty">No outgoing links.</div>
        {:else}
          {#each parsed.links as link}
            <button
              type="button"
              class="panel-row"
              title={link.target}
              on:click={() => void openTab(link.target.endsWith(".md") ? link.target : `${link.target}.md`)}
            >
              <Link2 size={11} style="color: var(--nc-fg-faint)" />
              <span class="truncate">{link.target}</span>
            </button>
          {/each}
        {/if}
      </div>

    {:else if $rightTab === "tasks"}
      <div class="panel-section">
        <div class="panel-title">
          Open tasks
          <span class="count">{tasks.filter((task) => !task.done).length}</span>
          {#if loading}<Loader2 size={11} class="spin" />{/if}
        </div>
        {#if tasks.length === 0}
          <div class="panel-empty">No tasks in this vault. Write <code>- [ ] something</code> in any note.</div>
        {:else}
          {#each tasks as task (task.note_path + ":" + task.line)}
            <div class="panel-row">
              <input
                type="checkbox"
                checked={task.done}
                aria-label={`Mark ${task.text} ${task.done ? "not done" : "done"}`}
                style="accent-color: var(--nc-accent)"
                on:change={(event) => void toggleTask(task, (event.currentTarget as HTMLInputElement).checked)}
              />
              <button
                type="button"
                style="flex:1; min-width:0; text-align:left; color: inherit; font: inherit; background:none; padding:0"
                on:click={() => void openTab(task.note_path).then(() => jump(task.line))}
              >
                <div class="truncate" style="color:{task.done ? 'var(--nc-fg-faint)' : 'var(--nc-fg)'}">{task.text}</div>
                <div class="truncate meta">{baseName(task.note_path)}{task.due ? ` · due ${task.due}` : ""}</div>
              </button>
            </div>
          {/each}
        {/if}
      </div>

    {:else if $rightTab === "history"}
      <div class="panel-section">
        <div class="panel-title">
          Version history
          <span class="count">{snapshots.length}</span>
          <button type="button" class="icon-btn sm" style="margin-left:auto" title="Snapshot now" on:click={() => void snapshotActive().then(() => loadSnapshots(tab?.path ?? ""))}>
            <Camera size={12} />
          </button>
        </div>
        {#if snapshots.length === 0}
          <div class="panel-empty">No snapshots yet. They are created automatically while editing, or on demand.</div>
        {:else}
          {#each snapshots as snapshot (snapshot.id)}
            <div class="panel-row">
              <div style="min-width:0; flex:1">
                <div class="truncate" style="color: var(--nc-fg)">{formatRelativeTime(snapshot.created_at)}</div>
                <div class="meta">{snapshot.label || "automatic"} · {snapshot.size.toLocaleString()} bytes</div>
              </div>
              <button type="button" class="btn ghost sm" title="Restore this version" on:click={() => void restore(snapshot)}>
                <RotateCcw size={12} />
              </button>
            </div>
          {/each}
        {/if}
      </div>
    {/if}
  </div>
</aside>

