import { askConfirm, pushToast } from "./state";
import { isTauri } from "./theme";

/**
 * Check the signed release feed and, with consent, install the update.
 * The updater plugin verifies the signature before anything is written.
 */
export async function checkForUpdates(silent = true): Promise<void> {
  if (!isTauri()) return;
  try {
    const { check } = await import("@tauri-apps/plugin-updater");
    const update = await check();
    if (!update) {
      if (!silent) pushToast("success", "NoteCraft is up to date");
      return;
    }
    const accepted = await askConfirm({
      title: `NoteCraft ${update.version} is available`,
      description: update.body?.slice(0, 500) || "A signed update is ready to install.",
      confirmLabel: "Download and install",
    });
    if (!accepted) return;
    pushToast("info", "Downloading update", "NoteCraft will restart when the update is ready.");
    await update.downloadAndInstall();
    const { relaunch } = await import("@tauri-apps/plugin-process");
    await relaunch();
  } catch (error) {
    if (!silent) pushToast("error", "Could not check for updates", String(error));
  }
}
