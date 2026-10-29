import type { Theme } from "./types";

export interface ThemeOptions {
  accent?: string;
  reducedMotion?: boolean;
  editorFontSize?: number;
  editorLineHeight?: number;
  readingWidth?: number;
}

export function normalizeTheme(value: unknown): Theme {
  return value === "light" ? "light" : "dark";
}

export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function hexToRgb(hex: string): [number, number, number] | null {
  const clean = hex.trim().replace(/^#/, "");
  if (!/^[0-9a-f]{3}$|^[0-9a-f]{6}$/i.test(clean)) return null;
  const full =
    clean.length === 3
      ? clean
          .split("")
          .map((c) => c + c)
          .join("")
      : clean;
  const n = Number.parseInt(full, 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

function readableOn(rgb: [number, number, number]): string {
  // Relative luminance, good enough for choosing black/white foreground.
  const [r, g, b] = rgb.map((v) => {
    const c = v / 255;
    return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  });
  const luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
  return luminance > 0.55 ? "#09090b" : "#ffffff";
}

export function applyTheme(theme: Theme, options: ThemeOptions = {}) {
  const root = document.documentElement;
  root.dataset.theme = theme;
  root.classList.toggle("dark", theme === "dark");
  root.classList.toggle("light", theme === "light");
  root.classList.toggle("reduced-motion", !!options.reducedMotion);

  if (options.accent) {
    const rgb = hexToRgb(options.accent);
    root.style.setProperty("--nc-accent", options.accent);
    root.style.setProperty("--nc-accent-fg", rgb ? readableOn(rgb) : "#ffffff");
    if (rgb) {
      // CSS consumes this as `rgb(var(--nc-accent-rgb) / 0.2)`, which needs
      // space-separated channels — commas would make the declaration invalid.
      root.style.setProperty("--nc-accent-rgb", `${rgb[0]} ${rgb[1]} ${rgb[2]}`);
    }
  }
  if (options.editorFontSize) {
    root.style.setProperty("--nc-editor-size", `${options.editorFontSize}px`);
  }
  if (options.editorLineHeight) {
    root.style.setProperty("--nc-editor-line", String(options.editorLineHeight));
  }
  if (options.readingWidth) {
    root.style.setProperty("--nc-reading-width", `${options.readingWidth}px`);
  }
}

export function systemPrefersDark(): boolean {
  return typeof window !== "undefined" && window.matchMedia("(prefers-color-scheme: dark)").matches;
}
