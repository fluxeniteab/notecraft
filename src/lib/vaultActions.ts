import { toast } from "sonner";
import { api } from "./tauri";
import { flushAll } from "./saveRegistry";
import { useAppStore } from "../store/appStore";

export async function refreshVault() {
  const { vaultRoot, setFiles, setAllFiles } = useAppStore.getState();
  if (!vaultRoot) {
    setFiles([]);
    setAllFiles([]);
    return;
  }
  try {
    const [files, allFiles] = await Promise.all([api.listDir(""), api.listAllFiles()]);
    setFiles(files);
    setAllFiles(allFiles);
  } catch (error) {
    console.error(error);
    toast.error("Could not read the vault", { description: String(error) });
  }
}

async function activateVault(root: string, created = false) {
  await flushAll();
  const store = useAppStore.getState();
  store.setVaultRoot(root);
  useAppStore.setState({
    tabs: [],
    activeTabId: null,
    selectedFile: null,
    tabStats: {},
    saveStatus: {},
    cursors: {},
  });
  await refreshVault();
  api.listRecentVaults().then(store.setRecentVaults).catch(() => {});
  toast.success(created ? "Vault created" : "Vault opened", { description: root });
}

export async function pickVault(mode: "open" | "create" = "open") {
  try {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const picked = await open({
      directory: true,
      multiple: false,
      title: mode === "create" ? "Choose a folder for the new vault" : "Open a vault",
    });
    if (typeof picked !== "string") return;
    const root = mode === "create" ? await api.createVault(picked) : await api.openVault(picked);
    await activateVault(root, mode === "create");
  } catch (error) {
    console.error(error);
    toast.error(mode === "create" ? "Could not create vault" : "Could not open vault", {
      description: String(error),
    });
  }
}

export async function openVaultAt(path: string) {
  try {
    const root = await api.openVault(path);
    await activateVault(root);
  } catch (error) {
    console.error(error);
    toast.error("Could not open vault", { description: String(error) });
    useAppStore.setState({
      recentVaults: useAppStore.getState().recentVaults.filter((v) => v !== path),
    });
  }
}

export async function closeCurrentVault() {
  await flushAll();
  try {
    await api.closeVault();
  } catch (error) {
    console.error(error);
  }
  useAppStore.setState({
    vaultRoot: null,
    files: [],
    allFiles: [],
    tabs: [],
    activeTabId: null,
    selectedFile: null,
    tabStats: {},
    saveStatus: {},
    cursors: {},
  });
}
