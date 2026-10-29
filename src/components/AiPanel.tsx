import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { toast } from "sonner";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  ArrowUp,
  Check,
  ChevronLeft,
  CircleSlash,
  ExternalLink,
  KeyRound,
  Loader2,
  Server,
  Sparkles,
  X,
} from "lucide-react";
import { ai, isConfigured, type AiConfig, type AiMessage, type AiProviderPreset } from "../lib/ai";
import { diffLines, runAgentTurn, type AgentStep, type EditProposal } from "../lib/agent";
import { api } from "../lib/tauri";
import { useAppStore } from "../store/appStore";
import ProviderLogo from "./ProviderLogo";

type Bubble =
  | { id: string; kind: "user" | "assistant"; text: string }
  | { id: string; kind: "tool" | "thought"; text: string };

let counter = 0;
const nextId = () => `ai-${Date.now()}-${(counter += 1)}`;

const EMPTY_CONFIG: AiConfig = {
  provider: "deepseek",
  base_url: "",
  api_key: "",
  model: "",
  temperature: 0.4,
};

export default function AiPanel() {
  const [providers, setProviders] = useState<AiProviderPreset[]>([]);
  const [config, setConfig] = useState<AiConfig | null>(null);
  const [form, setForm] = useState<AiConfig>(EMPTY_CONFIG);
  const [editing, setEditing] = useState(false);
  const [bubbles, setBubbles] = useState<Bubble[]>([]);
  const [history, setHistory] = useState<AiMessage[]>([]);
  const [proposals, setProposals] = useState<EditProposal[]>([]);
  const [input, setInput] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const scrollRef = useRef<HTMLDivElement>(null);

  const activeTabId = useAppStore((state) => state.activeTabId);
  const tabs = useAppStore((state) => state.tabs);
  const activePath = tabs.find((tab) => tab.id === activeTabId)?.path ?? null;

  const provider = useMemo(
    () => providers.find((entry) => entry.id === form.provider),
    [providers, form.provider]
  );
  const activeProvider = useMemo(
    () => providers.find((entry) => entry.id === config?.provider),
    [providers, config?.provider]
  );
  const ready = isConfigured(config, activeProvider);

  useEffect(() => {
    let alive = true;
    Promise.all([ai.providers(), ai.loadConfig()])
      .then(([list, stored]) => {
        if (!alive) return;
        setProviders(list);
        setConfig(stored);
        setForm(stored);
        setEditing(!isConfigured(stored, list.find((entry) => entry.id === stored.provider)));
      })
      .catch((problem) => alive && setError(String(problem)))
      .finally(() => alive && setLoading(false));
    return () => {
      alive = false;
    };
  }, []);

  useEffect(() => {
    const node = scrollRef.current;
    if (node) node.scrollTop = node.scrollHeight;
  }, [bubbles, proposals]);

  const chooseProvider = useCallback(
    (preset: AiProviderPreset) => {
      setForm((current) => ({
        ...current,
        provider: preset.id,
        base_url: preset.base_url,
        model: preset.model,
        api_key: preset.id === current.provider ? current.api_key : "",
      }));
    },
    []
  );

  const save = useCallback(async () => {
    const cleaned: AiConfig = {
      ...form,
      base_url: form.base_url.trim(),
      model: form.model.trim(),
      api_key: form.api_key.trim(),
      temperature: Math.min(2, Math.max(0, Number(form.temperature) || 0.4)),
    };
    if (!cleaned.base_url || !cleaned.model) {
      setError("An endpoint and a model are required.");
      return;
    }
    try {
      await ai.saveConfig(cleaned);
      setConfig(cleaned);
      setForm(cleaned);
      setEditing(false);
      setError(null);
      toast.success("AI provider saved", { description: `${provider?.name ?? cleaned.provider} · ${cleaned.model}` });
    } catch (problem) {
      setError(String(problem));
    }
  }, [form, provider]);

  const send = useCallback(async () => {
    const text = input.trim();
    if (!text || busy || !config) return;
    setInput("");
    setBusy(true);
    setError(null);
    setBubbles((current) => [...current, { id: nextId(), kind: "user", text }]);
    const steps: AgentStep[] = [];
    try {
      const result = await runAgentTurn({
        config,
        history,
        message: activePath ? `${text}\n\n(Active note: ${activePath})` : text,
        onEvent: (event) => {
          steps.push(event);
          if (event.kind === "proposal") {
            try {
              const proposal = JSON.parse(event.text) as EditProposal;
              setProposals((current) => [...current, proposal]);
              setBubbles((current) => [
                ...current,
                { id: nextId(), kind: "tool", text: `proposed an edit to ${proposal.path}` },
              ]);
            } catch {
              // ignore malformed proposals
            }
            return;
          }
          setBubbles((current) => [
            ...current,
            {
              id: nextId(),
              kind: event.kind === "final" ? "assistant" : event.kind === "tool" ? "tool" : "thought",
              text: event.text,
            },
          ]);
        },
      });
      setHistory(result.messages);
    } catch (problem) {
      const message = String(problem).replace(/^Error:\s*/, "");
      setError(message);
      setBubbles((current) => [
        ...current,
        { id: nextId(), kind: "assistant", text: `I could not reach the provider.\n\n${message}` },
      ]);
    } finally {
      setBusy(false);
    }
  }, [activePath, busy, config, history, input]);

  const accept = useCallback(async (proposal: EditProposal) => {
    try {
      await api.writeFile(proposal.path, proposal.after);
      setProposals((current) => current.filter((entry) => entry !== proposal));
      toast.success("Edit applied", { description: proposal.path });
      void api.rebuildLinks(proposal.path, proposal.after).catch(() => {});
    } catch (problem) {
      toast.error("Could not apply the edit", { description: String(problem) });
    }
  }, []);

  const reject = useCallback((proposal: EditProposal) => {
    setProposals((current) => current.filter((entry) => entry !== proposal));
    toast.message("Proposed edit rejected", { description: proposal.path });
  }, []);

  if (loading) {
    return (
      <div className="nc-ai">
        <div className="nc-ai-loading">
          <Loader2 size={14} className="animate-spin" /> Loading providers…
        </div>
      </div>
    );
  }

  if (editing || !ready) {
    return (
      <div className="nc-ai nc-ai-setup">
        <div className="nc-ai-setup-head">
          <div className="nc-ai-setup-title">
            <Sparkles size={14} /> Assistant
          </div>
          <p className="nc-ai-setup-sub">
            Pick a provider. Keys are stored on this machine and encrypted with your profile when
            it is unlocked. Requests go through the Rust core, not the webview.
          </p>
        </div>

        <div className="nc-ai-providers">
          {providers.map((entry) => (
            <button
              key={entry.id}
              type="button"
              className="nc-ai-provider"
              data-active={form.provider === entry.id}
              onClick={() => chooseProvider(entry)}
            >
              <span className="nc-ai-provider-logo">
                <ProviderLogo id={entry.id} size={18} />
              </span>
              <span className="nc-ai-provider-name">{entry.name}</span>
              <span className="nc-ai-provider-tag">
                {entry.local ? <><Server size={9} /> Local</> : "Cloud"}
              </span>
            </button>
          ))}
        </div>

        {provider && (
          <div className="nc-ai-fields">
            <label className="nc-ai-field">
              <span>Endpoint</span>
              <input
                className="nc-input !h-8"
                value={form.base_url}
                onChange={(event) => setForm({ ...form, base_url: event.target.value })}
                placeholder={provider.base_url}
                spellCheck={false}
              />
            </label>
            <label className="nc-ai-field">
              <span>Model</span>
              <input
                className="nc-input !h-8"
                value={form.model}
                onChange={(event) => setForm({ ...form, model: event.target.value })}
                placeholder={provider.model}
                spellCheck={false}
              />
            </label>
            <label className="nc-ai-field">
              <span>
                API key {provider.needs_key ? "" : "(not needed)"}
              </span>
              <input
                className="nc-input !h-8"
                type="password"
                value={form.api_key}
                onChange={(event) => setForm({ ...form, api_key: event.target.value })}
                placeholder={provider.needs_key ? "sk-…" : "local server"}
                spellCheck={false}
              />
            </label>
            <div className="nc-ai-hint">
              {provider.note}{" "}
              <button type="button" className="nc-ai-link" onClick={() => void openUrl(provider.docs)}>
                Get a key <ExternalLink size={9} />
              </button>
            </div>
          </div>
        )}

        {error && (
          <div className="nc-ai-error">
            <CircleSlash size={12} /> {error}
          </div>
        )}

        <div className="nc-ai-setup-actions">
          {config && ready && (
            <button type="button" className="nc-btn nc-btn-ghost" onClick={() => setEditing(false)}>
              Cancel
            </button>
          )}
          <button type="button" className="nc-btn nc-btn-primary" onClick={() => void save()}>
            <Check size={13} /> Save provider
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="nc-ai nc-ai-chat">
      <div className="nc-ai-head">
        <span className="nc-ai-head-logo">
          <ProviderLogo id={config?.provider ?? "deepseek"} size={16} />
        </span>
        <div className="nc-ai-head-text">
          <div className="nc-ai-head-name">{activeProvider?.name ?? config?.provider}</div>
          <div className="nc-ai-head-model">{config?.model}</div>
        </div>
        <button
          type="button"
          className="nc-icon-btn"
          title="Change provider"
          onClick={() => {
            setForm(config ?? EMPTY_CONFIG);
            setEditing(true);
          }}
        >
          <KeyRound size={13} />
        </button>
      </div>

      <div className="nc-ai-scroll nc-scroll" ref={scrollRef}>
        {bubbles.length === 0 && (
          <div className="nc-ai-empty">
            <Sparkles size={18} />
            <p>Ask about a note, search the vault, or describe the change you want.</p>
            <div className="nc-ai-suggestions">
              {[
                "Summarise the active note",
                "Find notes about this topic",
                "Tighten the writing in this note",
              ].map((suggestion) => (
                <button key={suggestion} type="button" onClick={() => setInput(suggestion)}>
                  {suggestion}
                </button>
              ))}
            </div>
          </div>
        )}

        {bubbles.map((bubble) => (
          <div key={bubble.id} className="nc-ai-bubble" data-kind={bubble.kind}>
            {bubble.kind === "thought" ? (
              <span className="nc-ai-thought">Thinking · {bubble.text}</span>
            ) : bubble.kind === "tool" ? (
              <span className="nc-ai-tool">{bubble.text}</span>
            ) : (
              <div className="nc-ai-text">{bubble.text}</div>
            )}
          </div>
        ))}

        {proposals.map((proposal, index) => (
          <div key={`${proposal.path}-${index}`} className="nc-ai-proposal">
            <div className="nc-ai-proposal-head">
              <div>
                <div className="nc-ai-proposal-title">{proposal.summary}</div>
                <div className="nc-ai-proposal-path">{proposal.path}</div>
              </div>
              <span className="nc-ai-proposal-badge">
                {diffLines(proposal.before, proposal.after).filter((line) => line.kind === "add").length} lines added
              </span>
            </div>
            <pre className="nc-ai-diff">
              {diffLines(proposal.before, proposal.after)
                .slice(0, 200)
                .map((line, position) => (
                  <div key={position} data-kind={line.kind}>
                    {line.kind === "add" ? "+ " : line.kind === "remove" ? "- " : "  "}
                    {line.text || " "}
                  </div>
                ))}
            </pre>
            <div className="nc-ai-proposal-actions">
              <button type="button" className="nc-btn nc-btn-outline" onClick={() => reject(proposal)}>
                <X size={12} /> Reject
              </button>
              <button type="button" className="nc-btn nc-btn-primary" onClick={() => void accept(proposal)}>
                <Check size={12} /> Accept edit
              </button>
            </div>
          </div>
        ))}

        {busy && (
          <div className="nc-ai-working">
            <Loader2 size={12} className="animate-spin" /> Working…
          </div>
        )}
      </div>

      <div className="nc-ai-composer">
        <textarea
          className="nc-ai-input"
          value={input}
          rows={2}
          placeholder={activePath ? `Ask about ${activePath}…` : "Ask the assistant…"}
          onChange={(event) => setInput(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter" && !event.shiftKey) {
              event.preventDefault();
              void send();
            }
          }}
        />
        <button
          type="button"
          className="nc-ai-send"
          disabled={busy || !input.trim()}
          onClick={() => void send()}
          aria-label="Send"
        >
          <ArrowUp size={15} />
        </button>
      </div>

      {error && (
        <div className="nc-ai-error">
          <CircleSlash size={12} /> {error}
        </div>
      )}
      {bubbles.length > 0 && (
        <button
          type="button"
          className="nc-ai-reset"
          onClick={() => {
            setBubbles([]);
            setHistory([]);
            setError(null);
          }}
        >
          <ChevronLeft size={11} /> New conversation
        </button>
      )}
    </div>
  );
}
