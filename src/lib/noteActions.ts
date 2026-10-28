import { toast } from "sonner";
import { api } from "./tauri";
import { joinRel, stem, uniqueName } from "./files";
import { refreshVault } from "./vaultActions";
import { useAppStore } from "../store/appStore";

/**
 * Create a note with a non-colliding name, refresh the vault index and open it.
 * Returns the new vault-relative path, or null when creation failed.
 */
export async function createNote(dir = "", base = "Untitled note"): Promise<string | null> {
  try {
    const { allFiles, openTab } = useAppStore.getState();
    const name = uniqueName(base, allFiles, dir);
    const rel = joinRel(dir, name);
    await api.createFile(rel, `# ${stem(name)}\n\n`);
    await refreshVault();
    openTab(rel);
    requestAnimationFrame(() => {
      document
        .querySelector<HTMLElement>('.nc-pane[data-active="true"] .cm-content')
        ?.focus();
    });
    return rel;
  } catch (error) {
    toast.error("Could not create note", { description: String(error) });
    return null;
  }
}

export async function duplicateNote(path: string): Promise<string | null> {
  try {
    const { allFiles, openTab } = useAppStore.getState();
    const content = await api.readFile(path);
    const dir = path.includes("/") ? path.slice(0, path.lastIndexOf("/")) : "";
    const name = uniqueName(stem(path), allFiles, dir);
    const rel = joinRel(dir, name);
    await api.createFile(rel, content);
    await refreshVault();
    openTab(rel);
    toast.success("Note duplicated", { description: rel });
    return rel;
  } catch (error) {
    toast.error("Could not duplicate note", { description: String(error) });
    return null;
  }
}
