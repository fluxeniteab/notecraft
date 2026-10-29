/**
 * Provider marks. Drawn inline so they follow the theme colour and add no
 * network requests. Simplified geometry, recognisable at 16–22px.
 */
export default function ProviderLogo({ id, size = 18 }: { id: string; size?: number }) {
  const common = {
    width: size,
    height: size,
    viewBox: "0 0 24 24",
    fill: "none",
    stroke: "currentColor",
    strokeWidth: 1.6,
    strokeLinecap: "round" as const,
    strokeLinejoin: "round" as const,
    "aria-hidden": true,
  };

  switch (id) {
    case "deepseek":
      return (
        <svg {...common}>
          <circle cx="12" cy="12" r="9" />
          <path d="M6 13.5c1.8-3.4 4.6-5 7.4-4.4 2 .4 3.4 1.8 4.2 3.6" />
          <path d="M9.5 16.2c1.6.7 3.4.7 5 0" />
          <circle cx="9.6" cy="11.2" r=".7" fill="currentColor" stroke="none" />
        </svg>
      );
    case "google":
      return (
        <svg {...common}>
          <path
            d="M12 3.2c.9 3.7 2.1 4.9 5.8 5.8-3.7.9-4.9 2.1-5.8 5.8-.9-3.7-2.1-4.9-5.8-5.8 3.7-.9 4.9-2.1 5.8-5.8Z"
            fill="currentColor"
            stroke="none"
          />
          <path
            d="M12 13.2c.5 1.9 1.1 2.5 3 3-1.9.5-2.5 1.1-3 3-.5-1.9-1.1-2.5-3-3 1.9-.5 2.5-1.1 3-3Z"
            fill="currentColor"
            stroke="none"
            opacity=".7"
          />
        </svg>
      );
    case "mistral":
      return (
        <svg {...common}>
          <path d="M4 18V9l4-3v12" fill="currentColor" stroke="none" opacity=".85" />
          <path d="M10 18V6l4-2v14" fill="currentColor" stroke="none" />
          <path d="M16 18v-8l4 2v6" fill="currentColor" stroke="none" opacity=".6" />
        </svg>
      );
    case "openai":
      return (
        <svg {...common}>
          <path d="M12 3.5 18.5 7v7L12 17.5 5.5 14V7L12 3.5Z" />
          <path d="M12 7.2 15.3 9v3.6L12 14.4 8.7 12.6V9L12 7.2Z" opacity=".55" />
          <path d="M5.5 7 12 10.8 18.5 7M12 10.8V17.5" opacity=".55" />
        </svg>
      );
    case "openrouter":
      return (
        <svg {...common}>
          <circle cx="6" cy="18" r="2" />
          <circle cx="18" cy="6" r="2" />
          <path d="M8 18h3.5a4 4 0 0 0 4-4V10a4 4 0 0 1 4-4" />
          <path d="M8 18h2.5" opacity=".6" />
        </svg>
      );
    case "groq":
      return (
        <svg {...common}>
          <rect x="3.5" y="3.5" width="17" height="17" rx="4" />
          <path d="M13 7.5 8.5 13H12l-1 3.5 4.5-5.5H12l1-3.5Z" fill="currentColor" stroke="none" />
        </svg>
      );
    case "ollama":
      return (
        <svg {...common}>
          <path d="M8.5 8.5C7 6.5 7.4 4.5 8.6 4.2c1.2-.3 2 1.1 2.4 2.6" />
          <path d="M15.5 8.5c1.5-2 1.1-4-.1-4.3-1.2-.3-2 1.1-2.4 2.6" />
          <path d="M6.8 12.4c0-2.6 2.3-4.6 5.2-4.6s5.2 2 5.2 4.6c0 3-2.3 5.4-5.2 5.4s-5.2-2.4-5.2-5.4Z" />
          <circle cx="10.2" cy="12.6" r=".7" fill="currentColor" stroke="none" />
          <circle cx="13.8" cy="12.6" r=".7" fill="currentColor" stroke="none" />
        </svg>
      );
    case "lmstudio":
      return (
        <svg {...common}>
          <rect x="3" y="4.5" width="18" height="12" rx="2" />
          <path d="M9 16.5v3h6v-3M8.5 8.5h7M8.5 12h4" opacity=".6" />
        </svg>
      );
    case "llamacpp":
      return (
        <svg {...common}>
          <rect x="3" y="4.5" width="18" height="15" rx="2.5" />
          <path d="m7 9.5 2.5 2.5L7 14.5M12 15h5" />
        </svg>
      );
    default:
      return (
        <svg {...common}>
          <circle cx="12" cy="12" r="8.5" />
          <path d="M12 7.5v9M7.5 12h9" opacity=".7" />
        </svg>
      );
  }
}
