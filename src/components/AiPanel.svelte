<script lang="ts">
  import { onMount, tick } from "svelte";
  import {
    ArrowUp,
    Check,
    CircleSlash,
    ExternalLink,
    KeyRound,
    Loader2,
    Plus,
    RotateCcw,
    Server,
    Sparkles,
    X,
  } from "lucide-svelte";
  import { EMPTY_AI_CONFIG, ai, isConfigured, type AiConfig, type AiMessage, type AiProviderPreset } from "../lib/ai";
  import { diffLines, runAgentTurn, type EditProposal } from "../lib/agent";
  import { api } from "../lib/ipc";
  import { activeTab, pushToast } from "../lib/state";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { isTauri } from "../lib/theme";

  type Bubble = { id: string; kind: "user" | "assistant" | "tool" | "thought"; text: string };

  let counter = 0;
  const nextId = () => `ai-${Date.now()}-${(counter += 1)}`;

  let providers: AiProviderPreset[] = [];
  let config: AiConfig | null = null;
  let form: AiConfig = { ...EMPTY_AI_CONFIG };
  let editing = false;
  let bubbles: Bubble[] = [];
  let proposals: EditProposal[] = [];
  let history: AiMessage[] = [];
  let input = "";
  let busy = false;
  let error: string | null = null;
  let loading = true;
  let scroller: HTMLDivElement | null = null;

  $: activePath = $activeTab?.path ?? null;
  $: provider = providers.find((entry) => entry.id === form.provider);
  $: activeProvider = providers.find((entry) => entry.id === config?.provider);
  $: ready = isConfigured(config, activeProvider);

  onMount(() => {
    void (async () => {
      try {
        const [list, stored] = await Promise.all([ai.providers(), ai.loadConfig()]);
        providers = list;
        config = stored;
        form = { ...stored };
        editing = !isConfigured(stored, list.find((entry) => entry.id === stored.provider));
      } catch (problem) {
        error = String(problem);
      } finally {
        loading = false;
      }
    })();
  });

  async function scrollToEnd(): Promise<void> {
    await tick();
    if (scroller) scroller.scrollTop = scroller.scrollHeight;
  }

  function chooseProvider(preset: AiProviderPreset): void {
    form = {
      ...form,
      provider: preset.id,
      base_url: preset.base_url,
      model: preset.model,
      api_key: preset.id === form.provider ? form.api_key : "",
    };
  }

  async function saveConfig(): Promise<void> {
    const cleaned: AiConfig = {
      ...form,
      base_url: form.base_url.trim(),
      model: form.model.trim(),
      api_key: form.api_key.trim(),
      temperature: Math.min(2, Math.max(0, Number(form.temperature) || 0.4)),
    };
    if (!cleaned.base_url || !cleaned.model) {
      error = "An endpoint and a model are required.";
      return;
    }
    try {
      await ai.saveConfig(cleaned);
      config = cleaned;
      form = { ...cleaned };
      editing = false;
      error = null;
      pushToast("success", "Assistant provider saved", `${provider?.name ?? cleaned.provider} · ${cleaned.model}`);
    } catch (problem) {
      error = String(problem);
    }
  }

  async function send(): Promise<void> {
    const text = input.trim();
    if (!text || busy || !config) return;
    input = "";
    error = null;
    busy = true;
    bubbles = [...bubbles, { id: nextId(), kind: "user", text }];
    void scrollToEnd();
    try {
      const result = await runAgentTurn({
        config,
        history,
        message: activePath ? `${text}\n\n(Active note: ${activePath})` : text,
        onEvent: (event) => {
          if (event.kind === "proposal") {
            try {
              const proposal = JSON.parse(event.text) as EditProposal;
              proposals = [...proposals, proposal];
              bubbles = [...bubbles, { id: nextId(), kind: "tool", text: `proposed an edit to ${proposal.path}` }];
            } catch {
              /* malformed proposal: ignore */
            }
          } else {
            bubbles = [
              ...bubbles,
              {
                id: nextId(),
                kind: event.kind === "final" ? "assistant" : event.kind === "tool" ? "tool" : "thought",
                text: event.text,
              },
            ];
          }
          void scrollToEnd();
        },
      });
      history = result.messages;
    } catch (problem) {
      const message = String(problem).replace(/^Error:\s*/, "");
      error = message;
      bubbles = [...bubbles, { id: nextId(), kind: "assistant", text: `I could not reach the provider.\n\n${message}` }];
    } finally {
      busy = false;
      void scrollToEnd();
    }
  }

  async function accept(proposal: EditProposal): Promise<void> {
    try {
      await api.writeFile(proposal.path, proposal.after);
      proposals = proposals.filter((entry) => entry !== proposal);
      pushToast("success", "Edit applied", proposal.path);
      void api.rebuildLinks(proposal.path, proposal.after).catch(() => {});
    } catch (problem) {
      pushToast("error", "Could not apply the edit", String(problem));
    }
  }

  function reject(proposal: EditProposal): void {
    proposals = proposals.filter((entry) => entry !== proposal);
    pushToast("info", "Proposed edit rejected", proposal.path);
  }

  function openDocs(url: string): void {
    if (!url) return;
    if (isTauri()) void openUrl(url).catch(() => {});
    else window.open(url, "_blank", "noopener");
  }

  const suggestions = [
    "Summarise the active note",
    "Find notes about this topic",
    "Tighten the writing in this note",
  ];
</script>

<div class="assistant">
  {#if loading}
    <div class="assistant-loading"><Loader2 size={13} class="spin" /> Loading providers…</div>
  {:else if editing || !ready}
    <div class="assistant-setup">
      <div class="setup-head">
        <Sparkles size={13} />
        <div>
          <strong>Assistant</strong>
          <p>
            Pick a provider. Keys are stored on this machine and sealed with your profile while it is
            unlocked; requests go through the Rust core, never the page.
          </p>
        </div>
      </div>

      <div class="providers">
        {#each providers as preset}
          <button
            type="button"
            class="provider"
            data-active={form.provider === preset.id}
            on:click={() => chooseProvider(preset)}
          >
            <span class="provider-name">{preset.name}</span>
            <span class="provider-tag">
              {#if preset.local}<Server size={9} /> Local{:else}Cloud{/if}
            </span>
          </button>
        {/each}
      </div>

      {#if provider}
        <label class="field">
          <span>Endpoint</span>
          <input value={form.base_url} placeholder={provider.base_url} spellcheck={false} on:input={(e) => (form = { ...form, base_url: e.currentTarget.value })} />
        </label>
        <label class="field">
          <span>Model</span>
          <input value={form.model} placeholder={provider.model} spellcheck={false} on:input={(e) => (form = { ...form, model: e.currentTarget.value })} />
        </label>
        <label class="field">
          <span>API key {provider.needs_key ? "" : "(not needed)"}</span>
          <input
            type="password"
            value={form.api_key}
            placeholder={provider.needs_key ? "sk-…" : "local server"}
            spellcheck={false}
            on:input={(e) => (form = { ...form, api_key: e.currentTarget.value })}
          />
        </label>
        <p class="hint">
          {provider.note}
          {#if provider.docs}
            <button type="button" class="link" on:click={() => openDocs(provider.docs)}>
              Get a key <ExternalLink size={9} />
            </button>
          {/if}
        </p>
      {/if}

      {#if error}<div class="error"><CircleSlash size={12} /> {error}</div>{/if}

      <div class="setup-actions">
        {#if config && ready}
          <button type="button" class="btn" on:click={() => (editing = false)}>Cancel</button>
        {/if}
        <button type="button" class="btn primary" on:click={() => void saveConfig()}>
          <Check size={12} /> Save provider
        </button>
      </div>
    </div>
  {:else}
    <div class="assistant-head">
      <Sparkles size={13} />
      <div class="head-text">
        <div class="head-name">{activeProvider?.name ?? config?.provider}</div>
        <div class="head-model">{config?.model}</div>
      </div>
      <button
        type="button"
        class="icon-btn sm"
        title="Change provider"
        aria-label="Change provider"
        on:click={() => {
          form = { ...(config ?? EMPTY_AI_CONFIG) };
          editing = true;
        }}
      >
        <KeyRound size={12} />
      </button>
      <button
        type="button"
        class="icon-btn sm"
        title="New conversation"
        aria-label="New conversation"
        on:click={() => {
          bubbles = [];
          history = [];
          proposals = [];
          error = null;
        }}
      >
        <RotateCcw size={12} />
      </button>
    </div>

    <div class="assistant-scroll" bind:this={scroller}>
      {#if bubbles.length === 0 && proposals.length === 0}
        <div class="assistant-empty">
          <Sparkles size={16} />
          <p>Ask about a note, search the vault, or describe the change you want.</p>
          <div class="suggestions">
            {#each suggestions as suggestion}
              <button type="button" on:click={() => (input = suggestion)}>{suggestion}</button>
            {/each}
          </div>
        </div>
      {/if}

      {#each bubbles as bubble (bubble.id)}
        {#if bubble.kind === "thought"}
          <p class="thought">Thinking · {bubble.text}</p>
        {:else if bubble.kind === "tool"}
          <p class="tool">{bubble.text}</p>
        {:else}
          <div class="bubble" data-kind={bubble.kind}>{bubble.text}</div>
        {/if}
      {/each}

      {#each proposals as proposal (proposal.path + proposal.summary)}
        <div class="proposal">
          <div class="proposal-head">
            <div>
              <div class="proposal-title">{proposal.summary}</div>
              <div class="proposal-path">{proposal.path}</div>
            </div>
            <span class="proposal-badge">
              {diffLines(proposal.before, proposal.after).filter((line) => line.kind === "add").length} lines added
            </span>
          </div>
          <pre class="diff">{#each diffLines(proposal.before, proposal.after).slice(0, 200) as line}<div data-kind={line.kind}>{line.kind === "add" ? "+ " : line.kind === "remove" ? "- " : "  "}{line.text || " "}</div>{/each}</pre>
          <div class="proposal-actions">
            <button type="button" class="btn sm" on:click={() => reject(proposal)}><X size={11} /> Reject</button>
            <button type="button" class="btn sm primary" on:click={() => void accept(proposal)}><Check size={11} /> Accept edit</button>
          </div>
        </div>
      {/each}

      {#if busy}
        <p class="working"><Loader2 size={11} class="spin" /> Working…</p>
      {/if}
    </div>

    {#if error}<div class="error"><CircleSlash size={12} /> {error}</div>{/if}

    <div class="composer">
      <textarea
        rows={2}
        bind:value={input}
        placeholder={activePath ? `Ask about ${activePath}…` : "Ask the assistant…"}
        on:keydown={(event) => {
          if (event.key === "Enter" && !event.shiftKey) {
            event.preventDefault();
            void send();
          }
        }}
      ></textarea>
      <button type="button" class="send" disabled={busy || !input.trim()} on:click={() => void send()} aria-label="Send">
        <ArrowUp size={14} />
      </button>
    </div>
    {#if bubbles.length > 0}
      <button
        type="button"
        class="reset"
        on:click={() => {
          bubbles = [];
          history = [];
          proposals = [];
          error = null;
        }}
      >
        <Plus size={11} /> New conversation
      </button>
    {/if}
  {/if}
</div>

<style>
  .assistant {
    display: flex;
    flex-direction: column;
    min-height: 0;
    height: 100%;
  }
  .assistant-loading {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 16px;
    color: var(--nc-fg-muted);
    font-size: 11.5px;
  }
  .assistant-setup {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 12px;
    overflow-y: auto;
  }
  .setup-head {
    display: flex;
    gap: 8px;
    color: var(--nc-fg-secondary);
  }
  .setup-head strong {
    display: block;
    color: var(--nc-fg);
    font-size: 12px;
    font-weight: 600;
    margin-bottom: 3px;
  }
  .setup-head p {
    margin: 0;
    font-size: 11px;
    line-height: 1.5;
    color: var(--nc-fg-muted);
  }
  .providers {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 5px;
  }
  .provider {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    padding: 7px 8px;
    border: 1px solid var(--nc-border);
    border-radius: var(--nc-r-sm);
    background: var(--nc-panel);
    text-align: left;
    transition: border-color var(--nc-fast) var(--nc-ease), background var(--nc-fast) var(--nc-ease);
  }
  .provider:hover {
    border-color: var(--nc-border-strong);
  }
  .provider[data-active="true"] {
    border-color: var(--nc-accent);
    background: var(--nc-accent-softer);
  }
  .provider-name {
    font-size: 11.5px;
    color: var(--nc-fg);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
  }
  .provider-tag {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    font-size: 9.5px;
    color: var(--nc-fg-faint);
    text-transform: uppercase;
    letter-spacing: 0.04em;
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
  .field input {
    height: 28px;
    padding: 0 8px;
    border: 1px solid var(--nc-border);
    border-radius: var(--nc-r-sm);
    background: var(--nc-bg);
    color: var(--nc-fg);
    font-size: 11.5px;
    font-family: var(--nc-font-mono);
  }
  .field input:focus-visible {
    border-color: var(--nc-accent);
    outline: none;
  }
  .hint {
    margin: 0;
    font-size: 10.5px;
    line-height: 1.5;
    color: var(--nc-fg-faint);
  }
  .link {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    color: var(--nc-accent);
    font-size: 10.5px;
    padding: 0;
  }
  .setup-actions {
    display: flex;
    justify-content: flex-end;
    gap: 6px;
    padding-top: 2px;
  }
  .error {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 8px;
    border-radius: var(--nc-r-sm);
    background: rgb(248 113 113 / 0.1);
    color: var(--nc-danger);
    font-size: 10.5px;
    line-height: 1.45;
  }
  .assistant-head {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 8px 10px;
    border-bottom: 1px solid var(--nc-border-soft);
    color: var(--nc-fg-secondary);
  }
  .head-text {
    flex: 1;
    min-width: 0;
  }
  .head-name {
    font-size: 11.5px;
    font-weight: 550;
    color: var(--nc-fg);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .head-model {
    font-size: 10px;
    color: var(--nc-fg-faint);
    font-family: var(--nc-font-mono);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .assistant-scroll {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 10px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .assistant-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 22px 8px;
    text-align: center;
    color: var(--nc-fg-faint);
  }
  .assistant-empty p {
    margin: 0;
    font-size: 11.5px;
    line-height: 1.55;
    color: var(--nc-fg-muted);
    max-width: 210px;
  }
  .suggestions {
    display: flex;
    flex-direction: column;
    gap: 4px;
    width: 100%;
  }
  .suggestions button {
    padding: 6px 8px;
    border: 1px solid var(--nc-border);
    border-radius: var(--nc-r-sm);
    font-size: 11px;
    color: var(--nc-fg-secondary);
    text-align: left;
  }
  .suggestions button:hover {
    border-color: var(--nc-border-strong);
    color: var(--nc-fg);
  }
  .bubble {
    font-size: 11.5px;
    line-height: 1.6;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    padding: 7px 9px;
    border-radius: var(--nc-r-md);
  }
  .bubble[data-kind="user"] {
    align-self: flex-end;
    max-width: 92%;
    background: var(--nc-accent-soft);
    color: var(--nc-fg);
  }
  .bubble[data-kind="assistant"] {
    color: var(--nc-fg);
  }
  .thought,
  .tool,
  .working {
    margin: 0;
    font-size: 10.5px;
    color: var(--nc-fg-faint);
    font-family: var(--nc-font-mono);
    display: flex;
    align-items: center;
    gap: 5px;
  }
  .proposal {
    border: 1px solid var(--nc-border);
    border-radius: var(--nc-r-md);
    background: var(--nc-panel);
    overflow: hidden;
  }
  .proposal-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 8px;
    padding: 8px 9px;
    border-bottom: 1px solid var(--nc-border-soft);
  }
  .proposal-title {
    font-size: 11.5px;
    color: var(--nc-fg);
  }
  .proposal-path {
    font-size: 10px;
    color: var(--nc-fg-faint);
    font-family: var(--nc-font-mono);
  }
  .proposal-badge {
    flex: none;
    font-size: 9.5px;
    color: var(--nc-fg-muted);
    border: 1px solid var(--nc-border);
    border-radius: 999px;
    padding: 1px 6px;
    white-space: nowrap;
  }
  .diff {
    margin: 0;
    max-height: 210px;
    overflow: auto;
    padding: 6px 0;
    font-family: var(--nc-font-mono);
    font-size: 10.5px;
    line-height: 1.5;
    color: var(--nc-fg-secondary);
  }
  .diff div {
    padding: 0 9px;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .diff div[data-kind="add"] {
    background: rgb(74 222 128 / 0.1);
    color: var(--nc-fg);
  }
  .diff div[data-kind="remove"] {
    background: rgb(248 113 113 / 0.1);
    color: var(--nc-fg-muted);
  }
  .proposal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 6px;
    padding: 7px 9px;
    border-top: 1px solid var(--nc-border-soft);
  }
  .composer {
    display: flex;
    align-items: flex-end;
    gap: 6px;
    padding: 8px 10px;
    border-top: 1px solid var(--nc-border-soft);
  }
  .composer textarea {
    flex: 1;
    min-width: 0;
    resize: none;
    padding: 6px 8px;
    border: 1px solid var(--nc-border);
    border-radius: var(--nc-r-sm);
    background: var(--nc-bg);
    color: var(--nc-fg);
    font: inherit;
    font-size: 11.5px;
    line-height: 1.5;
  }
  .composer textarea:focus-visible {
    border-color: var(--nc-accent);
    outline: none;
  }
  .send {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: var(--nc-r-sm);
    background: var(--nc-accent);
    color: var(--nc-accent-fg);
  }
  .send:disabled {
    background: var(--nc-hover);
    color: var(--nc-fg-faint);
    cursor: default;
  }
  .reset {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    padding: 0 0 8px;
    font-size: 10px;
    color: var(--nc-fg-faint);
  }
  .reset:hover {
    color: var(--nc-fg-secondary);
  }
</style>
