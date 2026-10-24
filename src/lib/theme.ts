export function applyTheme(theme: "system" | "light" | "dark", accent?: string) {
  const root = document.documentElement;
  const isDark =
    theme === "dark" ||
    (theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);
  root.classList.toggle("dark", isDark);
  if (accent) {
    root.style.setProperty("--nc-accent", accent);
  }
  // reduced motion / high contrast applied via classes elsewhere
}
