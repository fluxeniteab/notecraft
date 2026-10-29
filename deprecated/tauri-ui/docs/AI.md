# AI assistant

The **AI assistant** lives in the right rail (sparkle icon). It talks to a model
you choose, can read and search the vault, and proposes changes you approve one
by one.

## Providers

Pick a provider in the panel; every field is editable, so any OpenAI-compatible
gateway works too.

| Provider | Kind | Key | Runs locally |
| --- | --- | --- | --- |
| DeepSeek | OpenAI-compatible | required | no |
| Google Gemini | `generateContent` | required | no |
| Mistral | OpenAI-compatible | required | no |
| OpenAI | OpenAI-compatible | required | no |
| OpenRouter | OpenAI-compatible | required | no |
| Groq | OpenAI-compatible | required | no |
| Ollama | OpenAI-compatible | not needed | **yes** |
| LM Studio | OpenAI-compatible | not needed | **yes** |
| llama.cpp server | OpenAI-compatible | not needed | **yes** |

The three local options expect a server already running on this machine
(defaults: Ollama `127.0.0.1:11434`, LM Studio `127.0.0.1:1234`, llama.cpp
`127.0.0.1:8080`). Use the **Custom** endpoint field for anything else.

## Where the key goes

- Requests are made from the Rust core, not the webview. The page has no network
  permission in its CSP, so keys never reach JavaScript.
- The provider configuration is written to `ai.json` in the app data folder. It
  is encrypted with your profile key (ChaCha20-Poly1305) whenever a profile is
  unlocked, and only that profile can read it.

## What the assistant can do

The model answers with a single JSON object per step. Tools available:

| Tool | Effect |
| --- | --- |
| `list_notes` | list Markdown, MDX and JSON documents in the vault |
| `read_note` | read a note |
| `search` | full-text search with snippets |
| `create_note` | create a new note (never overwrites) |
| `propose_edit` | propose replacing a note, shown as a diff |
| `open_note` | open a note in the editor |
| `rename_note` | rename or move a note |
| `create_daily_note` | create or open today's daily note |
| `list_tasks` | list open tasks |
| `set_theme` | switch between the dark and light theme |

Edits are **never** applied automatically. `propose_edit` returns a card with a
line diff and **Accept edit** / **Reject** buttons; accepting writes the file
through the normal save path, so the search index and link graph update.

Each turn is capped at eight tool steps to avoid loops. Press `Esc` in the
composer or start a **New conversation** to reset context.

## Privacy

The only network traffic is the request to the provider you configured. Note
content is sent as part of that request — that is what makes answers useful.
With a local model nothing leaves the machine.
