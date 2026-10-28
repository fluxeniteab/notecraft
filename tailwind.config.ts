import type { Config } from "tailwindcss";

export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  darkMode: ["class", '[data-theme="dark"]'],
  theme: {
    extend: {
      colors: {
        bg: "rgb(var(--nc-bg-rgb) / <alpha-value>)",
        surface: "rgb(var(--nc-surface-rgb) / <alpha-value>)",
        elev: "rgb(var(--nc-elev-rgb) / <alpha-value>)",
        "bg-elev": "rgb(var(--nc-elev-rgb) / <alpha-value>)",
        raised: "rgb(var(--nc-raised-rgb) / <alpha-value>)",
        muted: "rgb(var(--nc-muted-rgb) / <alpha-value>)",
        "bg-muted": "rgb(var(--nc-muted-rgb) / <alpha-value>)",
        hover: "rgb(var(--nc-hover-rgb) / <alpha-value>)",
        fg: "rgb(var(--nc-fg-rgb) / <alpha-value>)",
        "fg-secondary": "rgb(var(--nc-fg-secondary-rgb) / <alpha-value>)",
        "fg-muted": "rgb(var(--nc-fg-muted-rgb) / <alpha-value>)",
        "fg-faint": "rgb(var(--nc-fg-faint-rgb) / <alpha-value>)",
        border: "rgb(var(--nc-border-rgb) / <alpha-value>)",
        "border-strong": "rgb(var(--nc-border-strong-rgb) / <alpha-value>)",
        accent: "rgb(var(--nc-accent-rgb) / <alpha-value>)",
        "accent-hover": "var(--nc-accent-hover)",
        "accent-fg": "var(--nc-accent-fg)",
        "accent-soft": "var(--nc-accent-soft)",
        blue: "rgb(var(--nc-blue-rgb) / <alpha-value>)",
        danger: "rgb(var(--nc-danger-rgb) / <alpha-value>)",
        success: "rgb(var(--nc-success-rgb) / <alpha-value>)",
        warning: "rgb(var(--nc-warning-rgb) / <alpha-value>)",
      },
      fontFamily: {
        ui: "var(--nc-font-ui)",
        editor: "var(--nc-font-editor)",
        title: "var(--nc-font-title)",
        mono: "var(--nc-font-mono)",
      },
      borderRadius: {
        xs: "var(--nc-r-xs)",
        sm: "var(--nc-r-sm)",
        md: "var(--nc-r-md)",
        lg: "var(--nc-r-lg)",
        xl: "var(--nc-r-xl)",
      },
      boxShadow: {
        sm: "var(--nc-shadow-sm)",
        DEFAULT: "var(--nc-shadow)",
        lg: "var(--nc-shadow-lg)",
        glow: "var(--nc-glow)",
      },
      transitionTimingFunction: {
        nc: "var(--nc-ease)",
      },
    },
  },
  plugins: [],
} satisfies Config;
