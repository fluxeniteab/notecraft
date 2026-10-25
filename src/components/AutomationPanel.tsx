import { useEffect, useState } from "react";
import { useAppStore } from "../store/appStore";
import { api } from "../lib/tauri";

export default function AutomationPanel() {
  const activeTab = useAppStore((s) => s.tabs.find((t) => t.id === s.activeTabId));
  const [rules, setRules] = useState<any[]>([]);
  const [name, setName] = useState("");
  const [action, setAction] = useState("set_property");
  const [key, setKey] = useState("status");
  const [value, setValue] = useState("done");
  const [runs, setRuns] = useState<any[]>([]);

  const load = async () => {
    try { setRules(await api.listAutomations() as any[]); } catch { /* ignore */ }
    try { setRuns(await api.listAutomationRuns() as any[]); } catch { /* ignore */ }
  };
  useEffect(() => { load(); }, []);

  const create = async () => {
    if (!name.trim()) return;
    const rule = {
      id: crypto.randomUUID(),
      name,
      enabled: true,
      action,
      key,
      value,
      config: { action, key, value },
    };
    await api.saveAutomation(rule);
    setName("");
    await load();
  };

  const run = async (id: string) => {
    try { await api.runAutomation(id, activeTab?.path); await load(); } catch (e) { console.error(e); }
  };

  const del = async (id: string) => {
    await api.deleteAutomation(id);
    await load();
  };

  return (
    <div className="text-xs space-y-2">
      <div className="font-semibold">Automations</div>
      <div className="space-y-1">
        <input value={name} onChange={(e) => setName(e.target.value)} placeholder="Rule name" className="w-full px-2 py-1 rounded bg-bg-muted outline-none" />
        <select value={action} onChange={(e) => setAction(e.target.value)} className="w-full px-2 py-1 rounded bg-bg-muted">
          <option value="set_property">Set property</option>
          <option value="move_note">Move note</option>
          <option value="insert_template">Insert template</option>
        </select>
        {action === "set_property" && (
          <>
            <input value={key} onChange={(e) => setKey(e.target.value)} placeholder="property key" className="w-full px-2 py-1 rounded bg-bg-muted outline-none" />
            <input value={value} onChange={(e) => setValue(e.target.value)} placeholder="value" className="w-full px-2 py-1 rounded bg-bg-muted outline-none" />
          </>
        )}
        <button onClick={create} className="w-full px-2 py-1 rounded bg-accent text-accent-fg">Create rule</button>
      </div>
      <div className="space-y-1">
        {rules.map((r) => (
          <div key={r.id} className="p-1.5 rounded bg-bg-muted">
            <div className="flex items-center gap-1">
              <span className="font-medium truncate flex-1">{r.name}</span>
              <button onClick={() => run(r.id)} className="px-1.5 py-0.5 rounded bg-accent text-accent-fg">Run</button>
              <button onClick={() => del(r.id)} className="px-1.5 py-0.5 rounded border border-border">Del</button>
            </div>
          </div>
        ))}
        {rules.length === 0 && <p className="text-fg-muted">No rules.</p>}
      </div>
      <div>
        <div className="font-semibold mt-2">Recent runs</div>
        {runs.slice(0, 5).map((r: any) => (
          <div key={r.id} className="text-[10px] text-fg-muted truncate">{r.status}: {r.log}</div>
        ))}
      </div>
    </div>
  );
}
