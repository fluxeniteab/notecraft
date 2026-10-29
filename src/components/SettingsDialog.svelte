<script lang="ts">
  import { onMount } from "svelte";
  import { Check, Download, Puzzle, Trash2, X } from "lucide-svelte";
  import { askConfirm, pushToast, settings, settingsOpen, settingsSection } from "../lib/state";
  import { persistSettings } from "../lib/actions";
  import { api } from "../lib/ipc";
  import { isTauri } from "../lib/theme";

  interface PluginInfo {
    id: string;
    name: string;
    version: string;
    author: string;
    description: string;
    enabled: boolean;
    permissions: string[];
  }
  interface CatalogEntry {
    id: string;
    name: string;
    version: string;
    description: string;
    installed: boolean;
  }

  const accents = ["#5b8cff", "#8b5cf6", "#2dd4bf", "#f59e0b", "#f43f5e", "#a3a3a3"];
  const sections = [
    { id: "appearance", label: "Appearance" },
    { id: "editor", label: "Editor" },
    { id: "workspace", label: "Workspace" },
    { id: "plugins", label: "Plugins" },
  ] as const;

  let plugins: PluginInfo[] = [];
  let catalog: CatalogEntry[] = [];
  let pluginsLoading = false;
  let saveTimer = 0;
  let pending: Parameters<typeof persistSettings>[0] = {};

  $: section = $settingsSection;

  onMount(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") close();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  $: if (section === "plugins" && plugins.length === 0 && !pluginsLoading) void loadPlugins();

  async function loadPlugins(): Promise<void> {
    if (!isTauri()) return;
    pluginsLoading = true;
    try {
      [plugins, catalog] = await Promise.all([api.listPlugins(), api.pluginCatalog().catch(() => [])]);
    } catch (problem) {
      pushToast("error", "Could not load plugins", String(problem));
    } finally {
      pluginsLoading = false;
    }
  }

  function close(): void {
    settingsOpen.set(false);
  }

  function queue(patch: Parameters<typeof persistSettings>[0]): void {
    pending = { ...pending, ...patch };
    window.clearTimeout(saveTimer);
    // Sliders fire many events; persist once the values settle.
    saveTimer = window.setTimeout(() => {
      const next = pending;
      pending = {};
      void persistSettings(next);
    }, 220);
  }

  function immediate(patch: Parameters<typeof persistSettings>[0]): void {
    window.clearTimeout(saveTimer);
    pending = {};
    void persistSettings(patch);
  }

  async function togglePlugin(plugin: PluginInfo): Promise<void> {
    try {
      await api.enablePlugin(plugin.id, !plugin.enabled);
      await loadPlugins();
      pushToast("success", plugin.enabled ? "Plugin disabled" : "Plugin enabled", plugin.name);
    } catch (problem) {
      pushToast("error", "Could not change the plugin", String(problem));
    }
  }

  async function uninstall(plugin: PluginInfo): Promise<void> {
    const ok = await askConfirm({
      title: `Uninstall ${plugin.name}?`,
      description: "The plugin is removed from this machine. Its notes and data are not touched.",
      confirmLabel: "Uninstall",
      danger: true,
    });
    if (!ok) return;
    try {
      await api.uninstallPlugin(plugin.id);
      await loadPlugins();
      pushToast("success", "Plugin uninstalled", plugin.name);
    } catch (problem) {
      pushToast("error", "Could not uninstall the plugin", String(problem));
    }
  }

  async function install(entry: CatalogEntry): Promise<void> {
    try {
      await api.installCatalogPlugin(entry.id);
      await loadPlugins();
      pushToast("success", "Plugin installed", entry.name);
    } catch (problem) {
      pushToast("error", "Could not install the plugin", String(problem));
    }
  }
</script>

<div class="overlay center" role="presentation" on:click={(event) => event.target === event.currentTarget && close()}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div tabindex="-1" class="dialog wide" role="dialog" aria-label="Settings" on:click|stopPropagation>
    <div class="dialog-header">
      <div class="titles">
        <h3>Settings</h3>
        <p>Preferences are stored locally and apply to every vault.</p>
      </div>
      <button type="button" class="icon-btn" aria-label="Close settings" on:click={close}><X size={15} /></button>
    </div>

    <div class="settings-body">
      <nav class="settings-nav" aria-label="Settings sections">
        {#each sections as item}
          <button
            type="button"
            class="nav-row"
            class:active={section === item.id}
            on:click={() => settingsSection.set(item.id)}
          >{item.label}</button>
        {/each}
      </nav>

      <div class="settings-content">
        {#if section === "appearance"}
          <div class="setting-block">
            <div class="field-label">Theme</div>
            <div class="segmented" style="width:fit-content">
              <button type="button" class:on={$settings.theme === "dark"} on:click={() => immediate({ theme: "dark" })}>Dark</button>
              <button type="button" class:on={$settings.theme === "light"} on:click={() => immediate({ theme: "light" })}>Light</button>
            </div>
            <p class="setting-note">Both themes are fully supported across menus, dialogs, previews and the editor.</p>
          </div>

          <div class="setting-block">
            <div class="field-label">Accent</div>
            <div class="swatches">
              {#each accents as accent}
                <button
                  type="button"
                  class="swatch"
                  class:on={$settings.accent.toLowerCase() === accent}
                  style="background:{accent}"
                  aria-label={`Accent ${accent}`}
                  on:click={() => immediate({ accent })}
                ></button>
              {/each}
            </div>
            <p class="setting-note">Used for selection, focus, links and active tabs.</p>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <strong>Reduce motion</strong>
              <small>Turns off panel, menu and toast transitions.</small>
            </div>
            <button
              type="button"
              class="toggle"
              class:on={$settings.reduced_motion}
              aria-label="Reduce motion"
              aria-pressed={$settings.reduced_motion}
              on:click={() => immediate({ reduced_motion: !$settings.reduced_motion })}
            ></button>
          </div>

        {:else if section === "editor"}
          <div class="setting-row">
            <div class="setting-info">
              <strong>Editor font size</strong>
              <small>Applies to every text and code buffer.</small>
            </div>
            <div class="setting-control">
              <input
                type="range"
                min="12"
                max="20"
                step="0.5"
                value={$settings.editor_font_size}
                on:input={(event) => queue({ editor_font_size: Number((event.currentTarget as HTMLInputElement).value) })}
              />
              <span class="value">{$settings.editor_font_size}px</span>
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <strong>Line height</strong>
              <small>Comfortable writing needs generous leading.</small>
            </div>
            <div class="setting-control">
              <input
                type="range"
                min="1.4"
                max="2"
                step="0.02"
                value={$settings.editor_line_height}
                on:input={(event) => queue({ editor_line_height: Number((event.currentTarget as HTMLInputElement).value) })}
              />
              <span class="value">{$settings.editor_line_height.toFixed(2)}</span>
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <strong>Reading width</strong>
              <small>Maximum width of the editor column, preview and split view.</small>
            </div>
            <div class="setting-control">
              <input
                type="range"
                min="560"
                max="980"
                step="20"
                value={$settings.reading_width}
                on:input={(event) => queue({ reading_width: Number((event.currentTarget as HTMLInputElement).value) })}
              />
              <span class="value">{$settings.reading_width}px</span>
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <strong>Autosave delay</strong>
              <small>Writes are atomic; unsaved changes are flushed when a tab closes.</small>
            </div>
            <div class="setting-control">
              <select
                class="select"
                style="width:120px"
                value={String($settings.autosave_ms)}
                on:change={(event) => immediate({ autosave_ms: Number((event.currentTarget as HTMLSelectElement).value) })}
              >
                <option value="400">0.4 s</option>
                <option value="800">0.8 s</option>
                <option value="1200">1.2 s</option>
                <option value="2000">2 s</option>
              </select>
            </div>
          </div>

        {:else if section === "workspace"}
          <div class="setting-row">
            <div class="setting-info">
              <strong>Explorer sort</strong>
              <small>Folders stay grouped above files.</small>
            </div>
            <select
              class="select"
              style="width:130px"
              value={$settings.explorer_sort}
              on:change={(event) => immediate({ explorer_sort: (event.currentTarget as HTMLSelectElement).value as typeof $settings.explorer_sort })}
            >
              <option value="name">Name</option>
              <option value="modified">Modified</option>
              <option value="size">Size</option>
            </select>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <strong>Group folders first</strong>
              <small>Keep the hierarchy readable when sorting by date.</small>
            </div>
            <button
              type="button"
              class="toggle"
              class:on={$settings.explorer_group_dirs}
              aria-label="Group folders first"
              aria-pressed={$settings.explorer_group_dirs}
              on:click={() => immediate({ explorer_group_dirs: !$settings.explorer_group_dirs })}
            ></button>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <strong>Daily note folder</strong>
              <small>Where “Daily note” creates new entries.</small>
            </div>
            <input
              class="input"
              style="width:150px"
              value={$settings.daily_note_folder}
              on:change={(event) => immediate({ daily_note_folder: (event.currentTarget as HTMLInputElement).value.trim() })}
            />
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <strong>Writing goal</strong>
              <small>Words per day. Set to 0 to hide the goal.</small>
            </div>
            <input
              class="input"
              type="number"
              min="0"
              style="width:110px"
              value={$settings.writing_goal_words}
              on:change={(event) => immediate({ writing_goal_words: Math.max(0, Number((event.currentTarget as HTMLInputElement).value)) })}
            />
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <strong>Search index</strong>
              <small>Rebuild after importing many files outside NoteCraft.</small>
            </div>
            <button
              type="button"
              class="btn sm"
              disabled={!isTauri()}
              on:click={() =>
                void api
                  .rebuildIndex()
                  .then(() => pushToast("success", "Search index rebuilt"))
                  .catch((error) => pushToast("error", "Index rebuild failed", String(error)))}
            >
              Rebuild index
            </button>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <strong>Data directory</strong>
              <small>Settings, sessions and profiles live here.</small>
            </div>
            <span class="value" style="max-width:220px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap" title={$settings.data_dir ?? ""}>
              {$settings.data_dir ?? "Default app data folder"}
            </span>
          </div>

        {:else}
          {#if !isTauri()}
            <p class="setting-note">Plugins are available in the desktop application.</p>
          {:else}
            <div class="setting-block">
              <div class="field-label">Installed</div>
              {#if plugins.length === 0}
                <p class="setting-note">No plugins installed. Plugins live in your vault and can be enabled per machine.</p>
              {:else}
                {#each plugins as plugin}
                  <div class="plugin-row">
                    <div class="plugin-icon"><Puzzle size={15} /></div>
                    <div class="plugin-info">
                      <strong>{plugin.name} <span class="meta">v{plugin.version}</span></strong>
                      <small>{plugin.description}</small>
                      {#if plugin.permissions.length}
                        <div class="meta" style="margin-top:3px">Permissions: {plugin.permissions.join(", ")}</div>
                      {/if}
                    </div>
                    <button
                      type="button"
                      class="toggle"
                      class:on={plugin.enabled}
                      aria-label={`${plugin.enabled ? "Disable" : "Enable"} ${plugin.name}`}
                      on:click={() => void togglePlugin(plugin)}
                    ></button>
                    <button type="button" class="icon-btn sm" title="Uninstall" on:click={() => void uninstall(plugin)}>
                      <Trash2 size={13} />
                    </button>
                  </div>
                {/each}
              {/if}
            </div>

            <div class="setting-block">
              <div class="field-label">Catalog</div>
              {#if catalog.length === 0}
                <p class="setting-note">No catalog entries are available offline.</p>
              {:else}
                {#each catalog as entry}
                  <div class="plugin-row">
                    <div class="plugin-icon"><Puzzle size={15} /></div>
                    <div class="plugin-info">
                      <strong>{entry.name} <span class="meta">v{entry.version}</span></strong>
                      <small>{entry.description}</small>
                    </div>
                    {#if entry.installed}
                      <span class="chip"><Check size={10} /> Installed</span>
                    {:else}
                      <button type="button" class="btn sm" on:click={() => void install(entry)}>
                        <Download size={12} /> Install
                      </button>
                    {/if}
                  </div>
                {/each}
              {/if}
            </div>
          {/if}
        {/if}
      </div>
    </div>

    <div class="dialog-footer">
      <span class="left">NoteCraft 0.4.0 · local-first</span>
      <button type="button" class="btn primary" on:click={close}>Done</button>
    </div>
  </div>
</div>

<style>
  .settings-body {
    display: flex;
    min-height: 420px;
    max-height: 62vh;
    border-top: 1px solid var(--nc-border);
  }
  .settings-nav {
    width: 170px;
    flex: 0 0 auto;
    padding: 10px 8px;
    border-right: 1px solid var(--nc-border);
    background: var(--nc-panel);
  }
  .settings-content {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
    padding: 16px 20px 22px;
  }
  .setting-block {
    margin-bottom: 18px;
  }
  .setting-block .field-label {
    margin-bottom: 8px;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--nc-fg-muted);
  }
  .setting-row {
    padding: 10px 0;
  }
  .setting-note {
    margin: 6px 0 0;
    color: var(--nc-fg-muted);
    font-size: 11px;
    line-height: 1.6;
    max-width: 52ch;
  }
  .value {
    min-width: 52px;
    text-align: right;
    color: var(--nc-fg-secondary);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
  }
  .plugin-row {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 10px 0;
    border-top: 1px solid var(--nc-border-soft);
  }
  .plugin-row:first-of-type {
    border-top: 0;
  }
  .plugin-icon {
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    border-radius: var(--nc-r-sm);
    background: var(--nc-panel);
    border: 1px solid var(--nc-border);
    color: var(--nc-fg-muted);
    flex: 0 0 auto;
  }
  .plugin-info {
    flex: 1;
    min-width: 0;
  }
  .plugin-info strong {
    display: block;
    font-size: 12px;
    font-weight: 500;
    color: var(--nc-fg);
  }
  .plugin-info small {
    display: block;
    margin-top: 2px;
    color: var(--nc-fg-muted);
    font-size: 10.5px;
    line-height: 1.55;
  }
  .meta {
    color: var(--nc-fg-faint);
    font-weight: 400;
  }
  @media (max-width: 720px) {
    .settings-nav {
      width: 128px;
    }
  }
</style>
