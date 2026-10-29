import { get } from "svelte/store";
import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
import { openPath } from "@tauri-apps/plugin-opener";
import { api } from "./ipc";
import type { DocKind } from "./documentTypes";
import { capabilitiesOf, ensureDocumentName } from "./documentTypes";
import { baseName, dirName, joinRel, normalizeRel, renamedFileName, stem, uniqueName } from "./files";
import { applyTheme, isTauri } from "./theme";
import {
  activeTabId,
  allFiles,
  askConfirm,
  askPrompt,
  closeTab,
  DEFAULT_SETTINGS,
  flushAllTabs,
  openTab,
  profile,
  pushToast,
  recentVaults,
  refreshFiles,
  resetWorkspace,
  restoreSession,
  saveSession,
  saveTab,
  searchQuery,
  searchResults,
  searching,
  selectedPath,
  settings,
  sidebarView,
  tabs,
  theme,
  vaultRoot,
} from "./state";

/* --------------------------------------------------------------- bootstrap -- */

export async function loadSettings(): Promise<void> {
  try {
    const loaded = await api.getSettings();
    const merged: typeof DEFAULT_SETTINGS = {
      ...DEFAULT_SETTINGS,
      ...loaded,
      theme: loaded.theme === "light" ? "light" : "dark",
    };
    settings.set(merged);
    applySettingsToDom();
    if (loaded.theme !== merged.theme) void api.saveSettings(merged).catch(() => {});
  } catch {
    settings.set({ ...DEFAULT_SETTINGS });
    applySettingsToDom();
  }
}

export function applySettingsToDom(): void {
  const value = get(settings);
  applyTheme(value.theme, {
    accent: value.accent,
    reducedMotion: value.reduced_motion,
    editorFontSize: value.editor_font_size,
    editorLineHeight: value.editor_line_height,
    readingWidth: value.reading_width,
  });
}

export async function persistSettings(patch: Partial<typeof DEFAULT_SETTINGS>): Promise<void> {
  const next = { ...get(settings), ...patch };
  settings.set(next);
  applySettingsToDom();
  try {
    await api.saveSettings(next);
  } catch (error) {
    pushToast("error", "Could not save settings", String(error));
  }
}

export async function toggleTheme(): Promise<void> {
  await persistSettings({ theme: get(theme) === "dark" ? "light" : "dark" });
}

export async function bootstrap(): Promise<void> {
  await loadSettings();

  try {
    recentVaults.set(await api.listRecentVaults());
  } catch {
    /* browser preview */
  }

  try {
    profile.set(await api.currentProfile());
  } catch {
    profile.set(null);
  }

  if (get(profile)) {
    try {
      const root = await api.getVaultRoot();
      if (root) {
        vaultRoot.set(root);
        await refreshFiles();
        await restoreSession();
      }
    } catch {
      /* no vault yet */
    }
  }
}

export async function afterUnlock(): Promise<void> {
  try {
    recentVaults.set(await api.listRecentVaults());
    const root = await api.getVaultRoot();
    if (root) {
      vaultRoot.set(root);
      await refreshFiles();
      await restoreSession();
    }
  } catch (error) {
    pushToast("error", "Could not open your vault", String(error));
  }
}

export async function lockProfile(): Promise<void> {
  await flushAllTabs();
  await api.lockProfile().catch(() => {});
  profile.set(null);
  vaultRoot.set(null);
  allFiles.set([]);
  resetWorkspace();
}

/* ------------------------------------------------------------------ vaults -- */

export async function pickVault(mode: "open" | "create" = "open"): Promise<void> {
  try {
    const picked = await openDialog({
      directory: true,
      multiple: false,
      title: mode === "create" ? "Choose a folder for the new vault" : "Open a vault",
    });
    if (typeof picked !== "string") return;
    const root = mode === "create" ? await api.createVault(picked) : await api.openVault(picked);
    await activateVault(root, mode === "create");
  } catch (error) {
    pushToast("error", mode === "create" ? "Could not create vault" : "Could not open vault", String(error));
  }
}

async function activateVault(root: string, created = false): Promise<void> {
  await flushAllTabs();
  vaultRoot.set(root);
  resetWorkspace();
  await refreshFiles();
  recentVaults.set(await api.listRecentVaults().catch(() => get(recentVaults)));
  pushToast("success", created ? "Vault created" : "Vault opened", root);
}

export async function openVaultAt(path: string): Promise<void> {
  try {
    const root = await api.openVault(path);
    await activateVault(root);
  } catch (error) {
    pushToast("error", "Could not open vault", String(error));
    recentVaults.update((list) => list.filter((item) => item !== path));
  }
}

export async function closeVault(): Promise<void> {
  await flushAllTabs();
  await api.closeVault().catch(() => {});
  vaultRoot.set(null);
  allFiles.set([]);
  resetWorkspace();
}

/* --------------------------------------------------------------- documents -- */

function focusEditorSoon(): void {
  requestAnimationFrame(() => {
    document.querySelector<HTMLElement>(".editor-mount .cm-content")?.focus();
  });
}

export function currentDirectory(): string {
  const selected = get(selectedPath);
  if (!selected) return "";
  const entry = get(allFiles).find((file) => file.path === selected);
  if (!entry) return dirName(selected);
  return entry.is_dir ? entry.path : dirName(selected);
}

export async function createDocument(kind: DocKind, dir = currentDirectory()): Promise<string | null> {
  if (!get(vaultRoot)) {
    pushToast("info", "Open a vault first");
    return null;
  }
  try {
    const caps = capabilitiesOf(kind);
    const label = kind === "plain-text" ? "Note" : caps.label;
    const wanted = `Untitled ${label}.${caps.extension}`;
    const name = uniqueName(wanted, get(allFiles), dir);
    const rel = joinRel(dir, name);
    await api.createDocument(rel);
    await refreshFiles();
    await openTab(rel);
    focusEditorSoon();
    return rel;
  } catch (error) {
    pushToast("error", "Could not create the file", String(error));
    return null;
  }
}

export async function createNamedDocument(kind: DocKind): Promise<void> {
  const caps = capabilitiesOf(kind);
  const dir = currentDirectory();
  const name = await askPrompt({
    title: `New ${caps.label} file`,
    description: `Created in ${dir || "the vault root"}. Include any supported extension; it is respected as typed.`,
    label: "File name",
    value: `Untitled.${caps.extension}`,
    placeholder: `notes.${caps.extension}`,
    confirmLabel: "Create",
  });
  if (!name || !name.trim()) return;
  if (!get(vaultRoot)) {
    pushToast("info", "Open a vault first");
    return;
  }
  try {
    let nameWithExt = ensureDocumentName(name.trim(), kind);
    // ensureDocumentName appends the fallback only when no known extension is
    // present, so `data.mdx` and `schema.json` are never rewritten to `.md`.
    nameWithExt = uniqueName(nameWithExt, get(allFiles), dir);
    const rel = joinRel(dir, nameWithExt);
    await api.createDocument(rel);
    await refreshFiles();
    await openTab(rel);
    focusEditorSoon();
  } catch (error) {
    pushToast("error", "Could not create the file", String(error));
  }
}

export async function createFolder(): Promise<void> {
  if (!get(vaultRoot)) return;
  const dir = currentDirectory();
  const name = await askPrompt({
    title: "New folder",
    description: dir ? `Inside ${dir}` : "At the vault root",
    label: "Folder name",
    value: "New folder",
    confirmLabel: "Create folder",
  });
  if (!name?.trim()) return;
  try {
    await api.createDir(joinRel(dir, name.trim().replace(/[\\/]+/g, "-")));
    await refreshFiles();
  } catch (error) {
    pushToast("error", "Could not create the folder", String(error));
  }
}

export function remapOpenTabs(from: string, to: string): void {
  const fromPath = normalizeRel(from);
  tabs.update((list) =>
    list.map((tab) => {
      if (tab.path !== fromPath && !tab.path.startsWith(`${fromPath}/`)) return tab;
      const nextPath = tab.path === fromPath ? normalizeRel(to) : `${normalizeRel(to)}/${tab.path.slice(fromPath.length + 1)}`;
      return { ...tab, path: nextPath, title: stem(nextPath) || baseName(nextPath) };
    })
  );
  if (get(activeTabId)) saveSession();
}

export async function renamePath(path: string): Promise<void> {
  const entry = get(allFiles).find((file) => file.path === path);
  const current = baseName(path);
  const name = await askPrompt({
    title: entry?.is_dir ? "Rename folder" : "Rename file",
    description: "Links elsewhere in the vault can be updated after renaming.",
    label: "New name",
    value: current,
    confirmLabel: "Rename",
  });
  if (!name?.trim() || name.trim() === current) return;
  const dir = dirName(path);
  const typed = name.trim().replace(/[\\/]+/g, "-");
  const nextName = entry?.is_dir ? typed : renamedFileName(typed, path);
  const to = normalizeRel(joinRel(dir, nextName));
  try {
    await api.renamePath(path, to);
    remapOpenTabs(path, to);
    await refreshFiles();
    if (!entry?.is_dir) {
      const impacts = await api.propagateLinkRename(path, to, false).catch(() => []);
      const count = impacts.reduce((sum, item) => sum + item.replacements, 0);
      if (count > 0) {
        const apply = await askConfirm({
          title: "Update links to the renamed file?",
          description: `${count} link${count === 1 ? "" : "s"} in ${impacts.length} file${impacts.length === 1 ? "" : "s"} still point at the old name.`,
          confirmLabel: "Update links",
        });
        if (apply) {
          await api.propagateLinkRename(path, to, true).catch((error) => pushToast("error", "Link update failed", String(error)));
          await refreshFiles();
        }
      }
    }
    pushToast("success", "Renamed", to);
  } catch (error) {
    pushToast("error", "Rename failed", String(error));
  }
}

export async function duplicatePath(path: string): Promise<void> {
  try {
    const content = await api.readFile(path);
    const dir = dirName(path);
    const ext = path.slice(path.lastIndexOf("."));
    const name = uniqueName(`${stem(path)} copy${ext}`, get(allFiles), dir);
    const rel = joinRel(dir, name);
    await api.createFile(rel, content);
    await refreshFiles();
    await openTab(rel);
    pushToast("success", "Duplicated", rel);
  } catch (error) {
    pushToast("error", "Could not duplicate the file", String(error));
  }
}

export async function deletePath(path: string): Promise<void> {
  const entry = get(allFiles).find((file) => file.path === path);
  const ok = await askConfirm({
    title: entry?.is_dir ? "Move folder to trash?" : "Move file to trash?",
    description: `${path} can be restored from .notecraft/trash inside the vault.`,
    confirmLabel: "Move to trash",
    danger: true,
  });
  if (!ok) return;
  try {
    const affected = get(tabs).filter((tab) => tab.path === path || tab.path.startsWith(`${path}/`));
    for (const tab of affected) {
      const closed = await closeTab(tab.id);
      if (!closed) return;
    }
    await api.deletePath(path, true);
    await refreshFiles();
    if (get(selectedPath) === path) selectedPath.set(null);
    pushToast("success", "Moved to trash", path);
  } catch (error) {
    pushToast("error", "Delete failed", String(error));
  }
}

export function movePath(from: string, toDir: string): Promise<void> {
  return api
    .movePath(from, toDir)
    .then(async (next) => {
      remapOpenTabs(from, next);
      await refreshFiles();
      pushToast("success", "Moved", next);
    })
    .catch((error) => pushToast("error", "Move failed", String(error)));
}

export async function saveActive(): Promise<void> {
  const id = get(activeTabId);
  if (!id) return;
  await saveTab(id);
  const tab = get(tabs).find((item) => item.id === id);
  if (tab?.status === "error") pushToast("error", "Save failed", tab.error ?? undefined);
  else pushToast("success", "Saved", tab?.path);
}

export async function snapshotActive(): Promise<void> {
  const tab = get(tabs).find((item) => item.id === get(activeTabId));
  if (!tab) return;
  try {
    await saveTab(tab.id);
    await api.snapshotNow(tab.path, "manual");
    pushToast("success", "Snapshot created", tab.title);
  } catch (error) {
    pushToast("error", "Could not create a snapshot", String(error));
  }
}

export type ExportFormat = "markdown" | "html" | "pdf";

export async function exportActive(format: ExportFormat): Promise<void> {
  const tab = get(tabs).find((item) => item.id === get(activeTabId));
  if (!tab) return;
  if (!isTauri()) {
    pushToast("info", "Exporting is available in the desktop app");
    return;
  }
  try {
    await saveTab(tab.id);
    if (format === "pdf") {
      const path = await api.exportPrintHtml(tab.path);
      await openPath(path);
      pushToast("success", "Opened for printing", "Choose “Save as PDF” in the print dialog.");
      return;
    }
    const ext = format === "html" ? "html" : "md";
    const dest = await saveDialog({
      defaultPath: `${stem(tab.path)}.${ext}`,
      filters: [{ name: format === "html" ? "HTML" : "Markdown", extensions: [ext] }],
    });
    if (!dest) return;
    await api.exportNote(tab.path, dest, format);
    pushToast("success", "Exported", dest);
  } catch (error) {
    pushToast("error", "Export failed", String(error));
  }
}

export async function createDailyNote(): Promise<void> {
  try {
    const path = await api.createDailyNote();
    await refreshFiles();
    await openTab(path);
    pushToast("success", "Daily note ready", path);
  } catch (error) {
    pushToast("error", "Could not open the daily note", String(error));
  }
}

export async function appendCapture(text: string): Promise<void> {
  try {
    const path = await api.appendCapture(text);
    await refreshFiles();
    pushToast("success", "Captured", path);
  } catch (error) {
    pushToast("error", "Could not save the capture", String(error));
  }
}

/* ------------------------------------------------------------------ search -- */

let searchSeq = 0;

export async function runSearch(query: string): Promise<void> {
  searchQuery.set(query);
  sidebarView.set("search");
  const trimmed = query.trim();
  if (!trimmed) {
    searchResults.set([]);
    searching.set(false);
    return;
  }
  const seq = ++searchSeq;
  searching.set(true);
  try {
    const hits = await api.search(trimmed);
    if (seq === searchSeq) searchResults.set(hits);
  } catch (error) {
    if (seq === searchSeq) {
      searchResults.set([]);
      pushToast("error", "Search failed", String(error));
    }
  } finally {
    if (seq === searchSeq) searching.set(false);
  }
}

