<script lang="ts">
  import { onMount } from "svelte";
  import { ChevronDown, ChevronRight, Loader2, Play, Plus, Trash2, X, Zap } from "lucide-svelte";
  import { activeTab, automationsOpen, pushToast, vaultRoot } from "../lib/state";
  import { api } from "../lib/ipc";
  import { baseName } from "../lib/files";

  interface AutomationRule {
    id: string;
    name: string;
    enabled: boolean;
    config: Record<string, unknown>;
  }

  interface AutomationRun {
    id: string;
    automation_id: string;
    started_at: string;
    status: string;
    log: string;
  }

  type Action = "set_property" | "move_note" | "insert_template";

  const actionLabels: Record<Action, string> = {
    set_property: "Set a property",
    move_note: "Move the note",
    insert_template: "Insert a template",
  };

  let rules: AutomationRule[] = [];
  let runs: AutomationRun[] = [];
  let loading = true;
  let busy: string | null = null;
  let expanded: string | null = null;
  let creating = false;

  let name = "";
  let action: Action = "set_property";
  let key = "status";
  let value = "done";
  let toDir = "";
  let template = "";

  $: notes = (runs ?? []).slice(0, 20);

  onMount(() => {
    void load();
    const onEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") automationsOpen.set(false);
    };
    window.addEventListener("keydown", onEscape);
    return () => window.removeEventListener("keydown", onEscape);
  });

  function asString(input: unknown, fallback = ""): string {
    return typeof input === "string" ? input : fallback;
  }

  async function load(): Promise<void> {
    loading = true;
    try {
      const [nextRules, nextRuns] = await Promise.all([
        api.listAutomations().catch(() => [] as unknown[]),
        api.listAutomationRuns().catch(() => [] as unknown[]),
      ]);
      rules = (nextRules as Array<Record<string, unknown>>)
        .filter((rule) => typeof rule?.id === "string")
        .map((rule) => ({
          id: String(rule.id),
          name: asString(rule.name, "Untitled rule"),
          enabled: rule.enabled !== false,
          config:
            rule.config && typeof rule.config === "object"
              ? (rule.config as Record<string, unknown>)
              : {},
        }));
      runs = nextRuns as AutomationRun[];
    } finally {
      loading = false;
    }
  }

  function describe(rule: AutomationRule): string {
    const kind = asString(rule.config.action) as Action;
    if (kind === "set_property") {
      return `Set ${asString(rule.config.key, "property")} = ${asString(rule.config.value)}`;
    }
    if (kind === "move_note") return `Move to ${asString(rule.config.to_dir, "…")}`;
    if (kind === "insert_template") return `Insert ${asString(rule.config.template, "…")}`;
    return "No action configured";
  }

  function draftConfig(): Record<string, unknown> {
    if (action === "set_property") return { action, key: key.trim(), value };
    if (action === "move_note") return { action, to_dir: toDir.trim() };
    return { action, template: template.trim() };
  }

  async function create(): Promise<void> {
    if (!name.trim()) {
      pushToast("error", "Give the rule a name first");
      return;
    }
    if (action === "set_property" && !key.trim()) {
      pushToast("error", "A property key is required");
      return;
    }
    if (action === "move_note" && !toDir.trim()) {
      pushToast("error", "Choose a destination folder");
      return;
    }
    if (action === "insert_template" && !template.trim()) {
      pushToast("error", "Choose a template path");
      return;
    }
    busy = "create";
    try {
      await api.saveAutomation({ name: name.trim(), enabled: true, config: draftConfig() });
      name = "";
      creating = false;
      await load();
      pushToast("success", "Automation saved");
    } catch (problem) {
      pushToast("error", "Could not save the automation", String(problem));
    } finally {
      busy = null;
    }
  }

  async function toggle(rule: AutomationRule): Promise<void> {
    try {
      await api.saveAutomation({ ...rule, enabled: !rule.enabled });
      rules = rules.map((item) => (item.id === rule.id ? { ...item, enabled: !item.enabled } : item));
    } catch (problem) {
      pushToast("error", "Could not update the rule", String(problem));
    }
  }

  async function remove(rule: AutomationRule): Promise<void> {
    try {
      await api.deleteAutomation(rule.id);
      rules = rules.filter((item) => item.id !== rule.id);
      if (expanded === rule.id) expanded = null;
      pushToast("success", "Automation deleted", rule.name);
    } catch (problem) {
      pushToast("error", "Could not delete the automation", String(problem));
    }
  }

  async function dryRun(rule: AutomationRule): Promise<void> {
    busy = rule.id;
    try {
      const result = await api.runAutomationDryRun(rule.id);
      pushToast("info", "Dry run", JSON.stringify(result).slice(0, 220));
    } catch (problem) {
      pushToast("error", "Dry run failed", String(problem));
    } finally {
      busy = null;
    }
  }

  async function run(rule: AutomationRule): Promise<void> {
    const path = $activeTab?.path;
    if (!path) {
      pushToast("info", "Open the note you want to run this rule on");
      return;
    }
    busy = rule.id;
    try {
      const result = (await api.runAutomation(rule.id, path)) as { status?: string; log?: string };
      if (result?.status === "error") {
        pushToast("error", "Automation reported an error", String(result.log ?? "").slice(0, 200));
      } else {
        pushToast("success", `Ran on ${baseName(path)}`, String(result?.log ?? "").trim().slice(0, 200));
      }
      runs = (await api.listAutomationRuns().catch(() => [])) as AutomationRun[];
    } catch (problem) {
      pushToast("error", "Could not run the automation", String(problem));
    } finally {
      busy = null;
    }
  }

  function close(): void {
    automationsOpen.set(false);
  }
</script>

<div class="overlay center" role="presentation" on:click={(event) => event.target === event.currentTarget && close()}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    tabindex="-1"
    class="dialog wide"
    role="dialog"
    aria-label="Automations"
    on:click|stopPropagation
    style="height:min(80vh, 700px); display:flex; flex-direction:column"
  >
    <div class="dialog-header" style="align-items:center">
      <div class="titles">
        <h3>Automations</h3>
        <p>Rules run on the note you have open. Nothing is scheduled in the background.</p>
      </div>
      <button type="button" class="btn sm" on:click={() => (creating = !creating)}>
        <Plus size={12} /> New rule
      </button>
      <button type="button" class="icon-btn" title="Close" aria-label="Close automations" on:click={close}>
        <X size={15} />
      </button>
    </div>

    {#if creating}
      <div class="rule-form">
        <label class="field">
          <span>Name</span>
          <input bind:value={name} placeholder="Mark as done" spellcheck="false" />
        </label>
        <div class="field-row">
          <label class="field">
            <span>Action</span>
            <select bind:value={action}>
              {#each Object.entries(actionLabels) as [id, label]}
                <option value={id}>{label}</option>
              {/each}
            </select>
          </label>
          {#if action === "set_property"}
            <label class="field">
              <span>Property</span>
              <input bind:value={key} placeholder="status" spellcheck="false" />
            </label>
            <label class="field">
              <span>Value</span>
              <input bind:value={value} placeholder="done" spellcheck="false" />
            </label>
          {:else if action === "move_note"}
            <label class="field">
              <span>Destination folder</span>
              <input bind:value={toDir} placeholder="archive/2026" spellcheck="false" />
            </label>
          {:else}
            <label class="field">
              <span>Template path</span>
              <input bind:value={template} placeholder="templates/meeting.md" spellcheck="false" />
            </label>
          {/if}
        </div>
        <div class="form-actions">
          <span class="spacer"></span>
          <button type="button" class="btn sm" on:click={() => (creating = false)}>Cancel</button>
          <button type="button" class="btn sm primary" disabled={busy === "create"} on:click={create}>
            {#if busy === "create"}<Loader2 size={11} class="spin" />{/if} Save rule
          </button>
        </div>
      </div>
    {/if}

    <div class="automation-body">
      {#if loading}
        <div class="editor-loading"><Loader2 size={15} class="spin" /> Loading automations…</div>
      {:else if !$vaultRoot}
        <div class="empty" style="background:transparent">
          <div class="empty-mark">·</div>
          <h2>Open a vault first</h2>
          <p>Automations write to notes, so they need a vault to work in.</p>
        </div>
      {:else if rules.length === 0}
        <div class="empty" style="background:transparent">
          <div class="empty-mark"><Zap size={16} /></div>
          <h2>No automations yet</h2>
          <p>Create a rule to set a property, move a note or insert a template with one click.</p>
          <div class="empty-actions">
            <button type="button" class="btn primary" on:click={() => (creating = true)}>
              <Plus size={12} /> New rule
            </button>
          </div>
        </div>
      {:else}
        <div class="rule-list">
          {#each rules as rule (rule.id)}
            <div class="rule" class:disabled={!rule.enabled}>
              <button
                type="button"
                class="rule-row"
                aria-expanded={expanded === rule.id}
                on:click={() => (expanded = expanded === rule.id ? null : rule.id)}
              >
                {#if expanded === rule.id}<ChevronDown size={13} />{:else}<ChevronRight size={13} />{/if}
                <span class="truncate rule-name">{rule.name}</span>
                <span class="rule-summary truncate">{describe(rule)}</span>
                <span class="spacer"></span>
                <span class="state-pill" data-on={rule.enabled}>{rule.enabled ? "on" : "off"}</span>
              </button>

              {#if expanded === rule.id}
                <div class="rule-detail">
                  <div class="detail-actions">
                    <button type="button" class="btn sm" on:click={() => void toggle(rule)}>
                      {rule.enabled ? "Disable" : "Enable"}
                    </button>
                    <button type="button" class="btn sm" disabled={busy === rule.id} on:click={() => void dryRun(rule)}>
                      Dry run
                    </button>
                    <button
                      type="button"
                      class="btn sm primary"
                      disabled={busy === rule.id || !$activeTab}
                      on:click={() => void run(rule)}
                    >
                      {#if busy === rule.id}<Loader2 size={11} class="spin" />{:else}<Play size={11} />{/if}
                      Run on {$activeTab ? baseName($activeTab.path) : "no open note"}
                    </button>
                    <span class="spacer"></span>
                    <button type="button" class="btn sm danger" on:click={() => void remove(rule)}>
                      <Trash2 size={11} /> Delete
                    </button>
                  </div>
                  <pre class="config">{JSON.stringify(rule.config, null, 2)}</pre>
                </div>
              {/if}
            </div>
          {/each}
        </div>

        <div class="run-history">
          <div class="head">Recent runs</div>
          {#if notes.length === 0}
            <div class="panel-empty">No runs recorded yet.</div>
          {:else}
            {#each notes as entry (entry.id)}
              <div class="panel-row" style="align-items:flex-start">
                <div style="min-width:0; flex:1">
                  <div class="truncate" style="color:var(--nc-fg)">
                    {rules.find((rule) => rule.id === entry.automation_id)?.name ?? "Deleted rule"}
                    <span class="state-pill" data-on={entry.status === "success"}>{entry.status}</span>
                  </div>
                  <div class="meta truncate">{entry.log || "no output"}</div>
                </div>
                <span class="meta">{entry.started_at.replace("T", " ").slice(0, 16)}</span>
              </div>
            {/each}
          {/if}
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .rule-form {
    display: grid;
    gap: 10px;
    padding: 12px 16px;
    border-bottom: 1px solid var(--nc-border-soft);
    background: var(--nc-panel);
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .field span {
    font-size: 10.5px;
    color: var(--nc-fg-muted);
  }
  .field input,
  .field select {
    height: 30px;
    padding: 0 8px;
    border: 1px solid var(--nc-border);
    border-radius: var(--nc-r-sm);
    background: var(--nc-bg);
    color: var(--nc-fg);
    font: inherit;
    font-size: 12px;
  }
  .field input:focus-visible,
  .field select:focus-visible {
    border-color: var(--nc-accent);
    outline: none;
  }
  .field-row {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 10px;
  }
  .form-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .automation-body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }
  .rule-list {
    display: flex;
    flex-direction: column;
  }
  .rule {
    border-bottom: 1px solid var(--nc-border-soft);
  }
  .rule.disabled {
    opacity: 0.62;
  }
  .rule-row {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 9px 16px;
    color: var(--nc-fg-secondary);
    font-size: 12px;
    text-align: left;
  }
  .rule-row:hover {
    background: var(--nc-hover);
  }
  .rule-name {
    color: var(--nc-fg);
    font-weight: 500;
    max-width: 34%;
  }
  .rule-summary {
    color: var(--nc-fg-faint);
    font-size: 11px;
  }
  .state-pill {
    flex: none;
    padding: 0 6px;
    height: 16px;
    display: inline-flex;
    align-items: center;
    border-radius: 999px;
    border: 1px solid var(--nc-border);
    font-size: 9.5px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--nc-fg-faint);
  }
  .state-pill[data-on="true"] {
    border-color: rgb(74 222 128 / 0.4);
    color: var(--nc-success);
  }
  .rule-detail {
    padding: 0 16px 12px;
  }
  .detail-actions {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 8px;
  }
  .config {
    margin: 0;
    padding: 8px 10px;
    border: 1px solid var(--nc-border-soft);
    border-radius: var(--nc-r-sm);
    background: var(--nc-bg);
    font-family: var(--nc-font-mono);
    font-size: 10.5px;
    line-height: 1.55;
    color: var(--nc-fg-secondary);
    overflow-x: auto;
  }
  .run-history {
    border-top: 1px solid var(--nc-border);
    padding-top: 6px;
  }
  .run-history .head {
    padding: 6px 16px;
    font-size: 10.5px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--nc-fg-faint);
  }
</style>
