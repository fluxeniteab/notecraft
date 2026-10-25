export function applyTheme(theme: "system" | "light" | "dark", accent?: string, reducedMotion?: boolean, highContrast?: boolean) {
  const root = document.documentElement;
  const isDark =
    theme === "dark" ||
    (theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);
  root.classList.toggle("dark", isDark);
  root.classList.toggle("reduced-motion", !!reducedMotion);
  root.classList.toggle("high-contrast", !!highContrast);
  if (accent) {
    root.style.setProperty("--nc-accent", accent);
  }
}
