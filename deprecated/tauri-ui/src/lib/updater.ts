import { toast } from "sonner";
import { isTauri } from "./theme";

let checkInFlight = false;

/**
 * Ask the Tauri updater whether a newer signed release exists.
 * Updates are signed with the project key, so a tampered download is rejected
 * before it can be installed.
 */
export async function checkForUpdates(options: { silent?: boolean } = {}) {
  if (!isTauri() || checkInFlight) return;
  checkInFlight = true;
  try {
    const [{ check }, { relaunch }] = await Promise.all([
      import("@tauri-apps/plugin-updater"),
      import("@tauri-apps/plugin-process"),
    ]);
    const update = await check();
    if (!update) {
      if (!options.silent) {
        toast.success("You are on the latest version of NoteCraft");
      }
      return;
    }
    toast(`NoteCraft ${update.version} is available`, {
      description: update.body?.split("\n")[0] ?? "A new signed build is ready to install.",
      duration: Infinity,
      action: {
        label: "Install and restart",
        onClick: async () => {
          try {
            toast.loading("Downloading the update…", { id: "notecraft-update" });
            await update.downloadAndInstall();
            toast.success("Update installed", { id: "notecraft-update" });
            await relaunch();
          } catch (error) {
            toast.error("Update failed", { id: "notecraft-update", description: String(error) });
          }
        },
      },
    });
  } catch (error) {
    // Offline or a release without an update manifest: not worth a toast unless
    // the user asked for a check.
    if (!options.silent) {
      toast.error("Could not check for updates", { description: String(error) });
    }
  } finally {
    checkInFlight = false;
  }
}
