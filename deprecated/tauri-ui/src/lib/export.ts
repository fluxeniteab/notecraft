import { save } from "@tauri-apps/plugin-dialog";
import { openPath } from "@tauri-apps/plugin-opener";
import { api } from "./tauri";
import { stem } from "./files";
import { isTauri } from "./theme";

export type ExportFormat = "markdown" | "html" | "pdf";

const FORMATS: Record<ExportFormat, { extension: string; filter: { name: string; extensions: string[] } }> = {
  markdown: { extension: "md", filter: { name: "Markdown", extensions: ["md"] } },
  html: { extension: "html", filter: { name: "HTML", extensions: ["html"] } },
  pdf: { extension: "html", filter: { name: "HTML", extensions: ["html"] } },
};

/**
 * Export a note through a real OS save dialog and a real file write.
 *
 * Returns the written path, or null when the user cancels the dialog.
 * "pdf" renders standalone HTML into the temp folder and hands it to the
 * system browser, whose print dialog can save a PDF fully offline.
 */
export async function exportNote(rel: string, format: ExportFormat): Promise<string | null> {
  if (!isTauri()) {
    throw new Error("Exporting is available in the desktop app");
  }
  if (format === "pdf") {
    const path = await api.exportPrintHtml(rel);
    await openPath(path);
    return path;
  }
  const meta = FORMATS[format];
  const dest = await save({
    defaultPath: `${stem(rel)}.${meta.extension}`,
    filters: [meta.filter],
  });
  if (!dest) return null;
  await api.exportNote(rel, dest, format);
  return dest;
}

/** Save arbitrary text (settings, diagnostics) where the user chooses. */
export async function saveTextFile(
  defaultName: string,
  content: string,
  filter: { name: string; extensions: string[] } = { name: "JSON", extensions: ["json"] }
): Promise<string | null> {
  if (!isTauri()) {
    const blob = new Blob([content], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = defaultName;
    anchor.click();
    URL.revokeObjectURL(url);
    return defaultName;
  }
  const dest = await save({ defaultPath: defaultName, filters: [filter] });
  if (!dest) return null;
  await api.saveTextFile(dest, content);
  return dest;
}
