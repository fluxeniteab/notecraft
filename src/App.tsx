import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export default function App() {
  const [greeting, setGreeting] = useState("");

  async function greet() {
    setGreeting(await invoke<string>("greet", { name: "NoteCraft" }));
  }

  return (
    <main className="min-h-screen bg-slate-950 text-slate-100 flex flex-col items-center justify-center gap-4">
      <h1 className="text-3xl font-semibold">NoteCraft</h1>
      <p className="text-slate-400">Tauri + React scaffold. Backend connected.</p>
      <button
        className="px-4 py-2 rounded bg-violet-600 hover:bg-violet-500"
        onClick={greet}
      >
        Test backend
      </button>
      {greeting && <p>{greeting}</p>}
    </main>
  );
}
