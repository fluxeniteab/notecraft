import { useCallback, useEffect, useMemo, useState } from "react";
import { toast } from "sonner";
import {
  ChevronDown,
  ChevronRight,
  Loader2,
  Play,
  Plus,
  Trash2,
  Zap,
} from "lucide-react";
import { useAppStore } from "../store/appStore";
import { api } from "../lib/tauri";
import { stem } from "../lib/files";

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

function asString(value: unknown, fallback = ""): string {
  return typeof value === "string" ? value : fallback;
}

export default function AutomationPanel() {
  const activePath = useAppStore(
    (s) => s.tabs.find((t) => t.id === s.activeTabId)?.path ?? null
  );
  const [rules, setRules] = useState<AutomationRule[]>([]);
  const [runs, setRuns] = useState<AutomationRun[]>([]);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState<string | null>(null);
  const [name, setName] = useState("");
  const [action, setAction] = useState<Action>("set_property");
  const [key, setKey] = useState("status");
  const [value, setValue] = useState("done");
  const [toDir, setToDir] = useState("");
  const [template, setTemplate] = useState("");
  const [expanded, setExpanded] = useState<string | null>(null);

  const load = useCallback(async () => {
    const [nextRules, nextRuns] = await Promise.all([
      api.listAutomations().catch(() => [] as unknown[]),
      api.listAutomationRuns().catch(() => [] as unknown[]),
    ]);
    setRules(
      (nextRules as Array<Record<string, unknown>>)
        .filter((rule) => typeof rule?.id === "string")
        .map((rule) => ({
          id: String(rule.id),
          name: asString(rule.name, "Untitled rule"),
          enabled: rule.enabled !== false,
          config:
            rule.config && typeof rule.config === "object"
              ? (rule.config as Record<string, unknown>)
              : {},
        }))
    );
    setRuns(nextRuns as AutomationRun[]);
    setLoading(false);
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const config = useMemo(() => {
    if (action === "set_property") return { action, key: key.trim(), value };
    if (action === "move_note") return { action, to_dir: toDir.trim() };
    return { action, template: template.trim() };
  }, [action, key, value, toDir, template]);

  const create = async () => {
    if (!name.trim()) {
      toast.error("Give the rule a name first");
      return;
    }
    if (action === "set_property" && !key.trim()) {
      toast.error("A property key is required");
      return;
    }
    if (action === "move_note" && !toDir.trim()) {
      toast.error("Choose a destination folder");
      return;
    }
    if (action === "insert_template" && !template.trim()) {
      toast.error("Choose a template path");
      return;
    }
    setBusy("create");
    try {
      await api.saveAutomation({
        id: crypto.randomUUID(),
        name: name.trim(),
        enabled: true,
        config,
      });
      setName("");
      setExpanded(null);
      await load();
      toast.success("Automation created", { description: name.trim() });
    } catch (error) {
      toast.error("Could not create the automation", { description: String(error) });
    } finally {
      setBusy(null);
    }
  };

  const toggle = async (rule: AutomationRule) => {
    setBusy(rule.id);
    try {
      await api.saveAutomation({ ...rule, enabled: !rule.enabled });
      await load();
    } catch (error) {
      toast.error("Could not update the automation", { description: String(error) });
    } finally {
      setBusy(null);
    }
  };

  const run = async (rule: AutomationRule) => {
    if (!activePath) {
      toast.info("Open a note before running an automation");
      return;
    }
    setBusy(rule.id);
    try {
      const result = (await api.runAutomation(rule.id, activePath)) as {
        status?: string;
        log?: string;
      };
      await load();
      setExpanded(rule.id);
      if (result?.status === "success") {
        toast.success(`Ran “${rule.name}”`, { description: stem(activePath) });
      } else {
        toast.error(`“${rule.name}” reported an error`, {
          description: result?.log?.trim() || "See the run log below.",
        });
      }
    } catch (error) {
      toast.error("Could not run the automation", { description: String(error) });
    } finally {
      setBusy(null);
    }
  };

  const remove = async (rule: AutomationRule) => {
    setBusy(rule.id);
    try {
      await api.deleteAutomation(rule.id);
      await load();
      toast.success("Automation deleted", { description: rule.name });
    } catch (error) {
      toast.error("Could not delete the automation", { description: String(error) });
    } finally {
      setBusy(null);
    }
  };

  return (
    <div className="pb-2">
      <div className="nc-panel-title">
        <Zap size={12} /> Automations
        <span className="nc-count">{rules.length}</span>
      </div>
      <p className="mb-2.5 px-1 text-[10.5px] leading-relaxed text-fg-faint">
        Rules run on the open note when you press Run. They never run in the background.
      </p>

      <div className="nc-auto-form">
        <label className="nc-field-label" htmlFor="auto-name">
          Name
        </label>
        <input
          id="auto-name"
          className="nc-input"
          value={name}
          onChange={(event) => setName(event.target.value)}
          placeholder="e.g. Mark as done"
          onKeyDown={(event) => {
            if (event.key === "Enter") void create();
          }}
        />

        <label className="nc-field-label" htmlFor="auto-action">
          Action
        </label>
        <select
          id="auto-action"
          className="nc-select w-full"
          value={action}
          onChange={(event) => setAction(event.target.value as Action)}
        >
          {(Object.keys(actionLabels) as Action[]).map((value) => (
            <option key={value} value={value}>
              {actionLabels[value]}
            </option>
          ))}
        </select>

        {action === "set_property" && (
          <div className="grid grid-cols-2 gap-2">
            <div>
              <label className="nc-field-label" htmlFor="auto-key">
                Property
              </label>
              <input
                id="auto-key"
                className="nc-input"
                value={key}
                onChange={(event) => setKey(event.target.value)}
                placeholder="status"
              />
            </div>
            <div>
              <label className="nc-field-label" htmlFor="auto-value">
                Value
              </label>
              <input
                id="auto-value"
                className="nc-input"
                value={value}
                onChange={(event) => setValue(event.target.value)}
                placeholder="done"
              />
            </div>
          </div>
        )}

        {action === "move_note" && (
          <>
            <label className="nc-field-label" htmlFor="auto-dir">
              Destination folder
            </label>
            <input
              id="auto-dir"
              className="nc-input"
              value={toDir}
              onChange={(event) => setToDir(event.target.value)}
              placeholder="archive/2026"
            />
          </>
        )}

        {action === "insert_template" && (
          <>
            <label className="nc-field-label" htmlFor="auto-template">
              Template path
            </label>
            <input
              id="auto-template"
              className="nc-input font-mono text-[11px]"
              value={template}
              onChange={(event) => setTemplate(event.target.value)}
              placeholder="templates/daily.md"
            />
          </>
        )}

        <button
          type="button"
          className="nc-btn nc-btn-primary mt-1 w-full justify-center"
          onClick={() => void create()}
          disabled={busy === "create"}
        >
          {busy === "create" ? (
            <Loader2 size={13} className="animate-spin" />
          ) : (
            <Plus size={13} />
          )}
          Create rule
        </button>
      </div>

      <div className="mt-3">
        {loading && (
          <div className="flex items-center gap-2 px-1 py-2 text-[11px] text-fg-muted">
            <Loader2 size={12} className="animate-spin" /> Loading rules…
          </div>
        )}

        {!loading && rules.length === 0 && (
          <div className="nc-panel-empty">
            No automations yet. Create one above to stamp properties, file notes away or append
            a template.
          </div>
        )}

        {rules.map((rule) => {
          const isOpen = expanded === rule.id;
          const ruleRuns = runs.filter((entry) => entry.automation_id === rule.id).slice(0, 3);
          return (
            <div key={rule.id} className="nc-auto-rule" data-enabled={rule.enabled}>
              <div className="flex items-center gap-1.5">
                <button
                  type="button"
                  className="nc-auto-expand"
                  aria-expanded={isOpen}
                  aria-label={isOpen ? "Hide run details" : "Show run details"}
                  onClick={() => setExpanded(isOpen ? null : rule.id)}
                >
                  {isOpen ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
                </button>
                <input
                  type="checkbox"
                  checked={rule.enabled}
                  onChange={() => void toggle(rule)}
                  aria-label={`Enable ${rule.name}`}
                  className="nc-checkbox"
                />
                <span className="min-w-0 flex-1 truncate text-[12px] font-medium" title={rule.name}>
                  {rule.name}
                </span>
                <button
                  type="button"
                  className="nc-icon-btn"
                  aria-label={`Run ${rule.name}`}
                  title={activePath ? `Run on ${stem(activePath)}` : "Open a note first"}
                  disabled={busy === rule.id}
                  onClick={() => void run(rule)}
                >
                  {busy === rule.id ? (
                    <Loader2 size={12} className="animate-spin" />
                  ) : (
                    <Play size={12} />
                  )}
                </button>
                <button
                  type="button"
                  className="nc-icon-btn hover:text-danger"
                  aria-label={`Delete ${rule.name}`}
                  title="Delete rule"
                  disabled={busy === rule.id}
                  onClick={() => void remove(rule)}
                >
                  <Trash2 size={12} />
                </button>
              </div>

              <div className="pl-[26px] text-[10px] text-fg-faint">
                {actionLabels[
                  (asString(rule.config.action, "set_property") as Action) in actionLabels
                    ? (asString(rule.config.action, "set_property") as Action)
                    : "set_property"
                ]}
                {rule.config.action === "set_property" && (
                  <>
                    {" · "}
                    <span className="font-mono">
                      {asString(rule.config.key, "?")}: {String(rule.config.value ?? "")}
                    </span>
                  </>
                )}
                {rule.config.action === "move_note" && (
                  <>
                    {" → "}
                    <span className="font-mono">{asString(rule.config.to_dir) || "vault root"}</span>
                  </>
                )}
                {rule.config.action === "insert_template" && (
                  <>
                    {" · "}
                    <span className="font-mono">{asString(rule.config.template) || "—"}</span>
                  </>
                )}
              </div>

              {isOpen && (
                <div className="mt-1.5 space-y-1 border-t border-border/70 pl-[26px] pt-1.5">
                  {ruleRuns.length === 0 ? (
                    <div className="text-[10px] text-fg-faint">No runs recorded yet.</div>
                  ) : (
                    ruleRuns.map((entry) => (
                      <div key={entry.id} className="text-[10px] leading-relaxed">
                        <span
                          className={
                            entry.status === "success" ? "text-success" : "text-danger"
                          }
                        >
                          {entry.status}
                        </span>
                        <span className="text-fg-faint">
                          {" · "}
                          {new Date(entry.started_at).toLocaleString()}
                        </span>
                        {entry.log && (
                          <pre className="mt-0.5 whitespace-pre-wrap break-words font-mono text-[9.5px] text-fg-muted">
                            {entry.log.trim()}
                          </pre>
                        )}
                      </div>
                    ))
                  )}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
