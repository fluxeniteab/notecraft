import type { Config } from "tailwindcss";

export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  darkMode: ["class"],
  theme: {
    extend: {
      colors: {
        bg: "var(--nc-bg)",
        "bg-elev": "var(--nc-bg-elev)",
        "bg-muted": "var(--nc-bg-muted)",
        fg: "var(--nc-fg)",
        "fg-muted": "var(--nc-fg-muted)",
        border: "var(--nc-border)",
        accent: "var(--nc-accent)",
        "accent-fg": "var(--nc-accent-fg)",
        danger: "var(--nc-danger)",
        success: "var(--nc-success)",
      },
      fontFamily: {
        ui: "var(--nc-font-ui)",
        editor: "var(--nc-font-editor)",
      },
    },
  },
  plugins: [],
} satisfies Config;
