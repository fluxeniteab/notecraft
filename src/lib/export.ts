import { api } from "./tauri";
import { stem } from "./files";

const PRINT_CSS = `
  :root { color-scheme: light; }
  body { margin: 0; padding: 40px; font: 15px/1.65 Georgia, "Times New Roman", serif; color: #111; background: #fff; }
  h1, h2, h3, h4, h5, h6 { font-family: -apple-system, "Segoe UI", sans-serif; line-height: 1.3; margin: 1.4em 0 .5em; }
  h1 { font-size: 1.9em; } h2 { font-size: 1.5em; } h3 { font-size: 1.2em; }
  p, ul, ol { margin: .7em 0; }
  li { margin: .25em 0; }
  blockquote { margin: 1em 0; padding: .4em 1em; border-left: 3px solid #8b5cf6; background: #f6f4ff; }
  code { font-family: "Cascadia Mono", Consolas, monospace; font-size: .92em; background: #f3f3f5; padding: .1em .35em; border-radius: 3px; }
  pre { padding: 12px; background: #f6f6f8; border: 1px solid #e4e4ea; border-radius: 6px; overflow: hidden; white-space: pre-wrap; }
  pre code { background: none; padding: 0; }
  table { border-collapse: collapse; margin: 1em 0; width: 100%; }
  th, td { border: 1px solid #d8d8de; padding: 6px 10px; text-align: left; font-size: .94em; }
  img { max-width: 100%; }
  hr { border: none; border-top: 1px solid #ddd; margin: 1.6em 0; }
  a { color: #6d28d9; text-decoration: none; }
  a::after { content: " (" attr(href) ")"; font-size: .8em; color: #666; }
  a[href^="#"]::after { content: ""; }
  @media print { body { padding: 12mm; } }
`;

/**
 * Export a note as PDF through the OS print pipeline: render the sanitized
 * HTML in a dedicated window and open the WebView print dialog, where
 * "Save as PDF" (and printers) are available. Fully offline.
 */
export async function exportNotePdf(rel: string): Promise<void> {
  const content = await api.readFile(rel);
  const body = await api.renderHtml(content);
  const win = window.open("", "_blank", "width=960,height=1200");
  if (!win) {
    throw new Error("The print window was blocked by the WebView");
  }
  const title = stem(rel);
  win.document.open();
  win.document.write(
    `<!doctype html><html><head><meta charset="utf-8"><title>${title}</title>` +
      `<style>${PRINT_CSS}</style></head><body>${body}</body></html>`
  );
  win.document.close();
  await new Promise<void>((resolve) => {
    if (win.document.readyState === "complete") resolve();
    else win.addEventListener("load", () => resolve(), { once: true });
    // Fallback: never hang the caller if the load event is missed.
    window.setTimeout(resolve, 1200);
  });
  win.focus();
  win.print();
}