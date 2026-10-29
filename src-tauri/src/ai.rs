//! AI provider bridge.
//!
//! Requests are made from Rust, not the webview: the app's CSP forbids network
//! access from the page, API keys never have to touch browser storage, and the
//! same client works for cloud providers and local runtimes. The provider
//! configuration is stored in the app data folder and encrypted with the
//! unlocked profile key when one is available.

use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize)]
pub struct ProviderPreset {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: &'static str, // "openai" | "gemini"
    pub base_url: &'static str,
    pub model: &'static str,
    pub needs_key: bool,
    pub local: bool,
    pub docs: &'static str,
    pub note: &'static str,
}

/// Every provider NoteCraft can talk to. The three local runtimes need no key:
/// they talk to a server already running on the machine.
pub fn presets() -> Vec<ProviderPreset> {
    vec![
        ProviderPreset {
            id: "deepseek",
            name: "DeepSeek",
            kind: "openai",
            base_url: "https://api.deepseek.com/v1",
            model: "deepseek-chat",
            needs_key: true,
            local: false,
            docs: "https://platform.deepseek.com/api_keys",
            note: "OpenAI-compatible chat completions.",
        },
        ProviderPreset {
            id: "google",
            name: "Google Gemini",
            kind: "gemini",
            base_url: "https://generativelanguage.googleapis.com/v1beta",
            model: "gemini-2.0-flash",
            needs_key: true,
            local: false,
            docs: "https://aistudio.google.com/app/apikey",
            note: "Uses the Gemini generateContent API.",
        },
        ProviderPreset {
            id: "mistral",
            name: "Mistral",
            kind: "openai",
            base_url: "https://api.mistral.ai/v1",
            model: "mistral-large-latest",
            needs_key: true,
            local: false,
            docs: "https://console.mistral.ai/api-keys",
            note: "OpenAI-compatible chat completions.",
        },
        ProviderPreset {
            id: "openai",
            name: "OpenAI",
            kind: "openai",
            base_url: "https://api.openai.com/v1",
            model: "gpt-4o-mini",
            needs_key: true,
            local: false,
            docs: "https://platform.openai.com/api-keys",
            note: "OpenAI-compatible chat completions.",
        },
        ProviderPreset {
            id: "openrouter",
            name: "OpenRouter",
            kind: "openai",
            base_url: "https://openrouter.ai/api/v1",
            model: "openrouter/auto",
            needs_key: true,
            local: false,
            docs: "https://openrouter.ai/keys",
            note: "One key for many hosted models.",
        },
        ProviderPreset {
            id: "groq",
            name: "Groq",
            kind: "openai",
            base_url: "https://api.groq.com/openai/v1",
            model: "llama-3.3-70b-versatile",
            needs_key: true,
            local: false,
            docs: "https://console.groq.com/keys",
            note: "Fast hosted inference.",
        },
        ProviderPreset {
            id: "ollama",
            name: "Ollama",
            kind: "openai",
            base_url: "http://127.0.0.1:11434/v1",
            model: "llama3.2",
            needs_key: false,
            local: true,
            docs: "https://ollama.com/download",
            note: "Runs on this PC. Start `ollama serve` first.",
        },
        ProviderPreset {
            id: "lmstudio",
            name: "LM Studio",
            kind: "openai",
            base_url: "http://127.0.0.1:1234/v1",
            model: "local-model",
            needs_key: false,
            local: true,
            docs: "https://lmstudio.ai",
            note: "Runs on this PC. Start the local server in LM Studio.",
        },
        ProviderPreset {
            id: "llamacpp",
            name: "llama.cpp",
            kind: "openai",
            base_url: "http://127.0.0.1:8080/v1",
            model: "local-model",
            needs_key: false,
            local: true,
            docs: "https://github.com/ggml-org/llama.cpp",
            note: "Runs on this PC. Start `llama-server` first.",
        },
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AiConfig {
    pub provider: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub temperature: f32,
}

impl Default for AiConfig {
    fn default() -> Self {
        let preset = presets().remove(0);
        Self {
            provider: preset.id.to_string(),
            base_url: preset.base_url.to_string(),
            api_key: String::new(),
            model: preset.model.to_string(),
            temperature: 0.4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

fn config_path() -> std::path::PathBuf {
    crate::settings::data_dir().join("ai.json")
}

pub fn load_config() -> AiConfig {
    let path = config_path();
    if !path.exists() {
        return AiConfig::default();
    }
    match crate::secure::read_note_text(&path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
        Err(_) => AiConfig::default(),
    }
}

pub fn save_config(config: &AiConfig) -> AppResult<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(AppError::Io)?;
    }
    let text = serde_json::to_string_pretty(config).map_err(|e| AppError::Other(e.to_string()))?;
    crate::secure::write_note_file(&path, text.as_bytes())
}

fn trimmed(value: &str) -> &str {
    value.trim().trim_end_matches('/')
}

fn openai_url(base_url: &str) -> String {
    let base = trimmed(base_url);
    if base.ends_with("/chat/completions") {
        base.to_string()
    } else if base.ends_with("/v1") || base.ends_with("/openai/v1") || base.ends_with("/api/v1") {
        format!("{}/chat/completions", base)
    } else {
        format!("{}/v1/chat/completions", base)
    }
}

fn gemini_url(base_url: &str, model: &str) -> String {
    let base = trimmed(base_url);
    format!("{}/models/{}:generateContent", base, model)
}

fn openai_body(config: &AiConfig, system: &str, messages: &[ChatMessage]) -> Value {
    let mut wire = Vec::new();
    if !system.trim().is_empty() {
        wire.push(json!({ "role": "system", "content": system }));
    }
    for message in messages {
        let role = match message.role.as_str() {
            "assistant" => "assistant",
            "system" => "system",
            _ => "user",
        };
        wire.push(json!({ "role": role, "content": message.content }));
    }
    json!({
        "model": config.model,
        "messages": wire,
        "temperature": config.temperature,
        "stream": false,
    })
}

fn gemini_body(config: &AiConfig, system: &str, messages: &[ChatMessage]) -> Value {
    let contents: Vec<Value> = messages
        .iter()
        .map(|message| {
            let role = if message.role == "assistant" {
                "model"
            } else {
                "user"
            };
            json!({ "role": role, "parts": [{ "text": message.content }] })
        })
        .collect();
    let mut body = json!({
        "contents": contents,
        "generationConfig": { "temperature": config.temperature },
    });
    if !system.trim().is_empty() {
        body["systemInstruction"] = json!({ "parts": [{ "text": system }] });
    }
    body
}

fn text_from_openai(value: &Value) -> Option<String> {
    let content = value
        .get("choices")?
        .as_array()?
        .first()?
        .get("message")?
        .get("content")?;
    if let Some(text) = content.as_str() {
        return Some(text.to_string());
    }
    // Some gateways return an array of content parts.
    let parts = content.as_array()?;
    let joined: String = parts
        .iter()
        .filter_map(|part| part.get("text").and_then(|t| t.as_str()))
        .collect::<Vec<_>>()
        .join("");
    if joined.is_empty() {
        None
    } else {
        Some(joined)
    }
}

fn text_from_gemini(value: &Value) -> Option<String> {
    let candidate = value.get("candidates")?.as_array()?.first()?;
    let parts = candidate.get("content")?.get("parts")?.as_array()?;
    let joined: String = parts
        .iter()
        .filter_map(|part| part.get("text").and_then(|t| t.as_str()))
        .collect::<Vec<_>>()
        .join("");
    if joined.is_empty() {
        None
    } else {
        Some(joined)
    }
}

fn provider_error(status: u16, body: &str) -> AppError {
    let trimmed_body = body.trim();
    let snippet: String = trimmed_body.chars().take(600).collect();
    AppError::Other(format!("provider returned {status}: {snippet}"))
}

/// Send one conversation turn to the configured provider and return the reply.
pub async fn chat(config: &AiConfig, system: &str, messages: &[ChatMessage]) -> AppResult<String> {
    if config.model.trim().is_empty() {
        return Err(AppError::Invalid("no model is configured".into()));
    }
    if config.base_url.trim().is_empty() {
        return Err(AppError::Invalid("no endpoint is configured".into()));
    }
    let provider = presets()
        .into_iter()
        .find(|preset| preset.id == config.provider);
    let is_gemini = provider
        .map(|preset| preset.kind == "gemini")
        .unwrap_or(false);
    let needs_key = provider.map(|preset| preset.needs_key).unwrap_or(true);
    if needs_key && config.api_key.trim().is_empty() {
        return Err(AppError::Invalid(
            "this provider needs an API key (Settings → AI)".into(),
        ));
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| AppError::Other(e.to_string()))?;

    let request = if is_gemini {
        let key = config.api_key.trim();
        let mut builder = client
            .post(gemini_url(&config.base_url, config.model.trim()))
            .header("content-type", "application/json")
            .json(&gemini_body(config, system, messages));
        if !key.is_empty() {
            builder = builder.header("x-goog-api-key", key);
        }
        builder
    } else {
        let mut builder = client
            .post(openai_url(&config.base_url))
            .header("content-type", "application/json")
            .json(&openai_body(config, system, messages));
        if !config.api_key.trim().is_empty() {
            builder = builder.bearer_auth(config.api_key.trim());
        }
        builder
    };

    let response = request
        .send()
        .await
        .map_err(|e| AppError::Other(format!("could not reach the provider: {e}")))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| AppError::Other(format!("could not read the reply: {e}")))?;
    if !status.is_success() {
        return Err(provider_error(status.as_u16(), &body));
    }
    let value: Value = serde_json::from_str(&body)
        .map_err(|e| AppError::Other(format!("the provider returned invalid JSON: {e}")))?;
    let text = if is_gemini {
        text_from_gemini(&value)
    } else {
        text_from_openai(&value)
    };
    text.ok_or_else(|| AppError::Other("the provider returned an empty reply".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_include_local_and_cloud_providers() {
        let all = presets();
        assert!(all.iter().any(|preset| preset.local && !preset.needs_key));
        assert!(all
            .iter()
            .any(|preset| preset.id == "deepseek" && preset.needs_key));
    }

    #[test]
    fn openai_endpoint_is_normalised() {
        assert_eq!(
            openai_url("https://api.deepseek.com/v1"),
            "https://api.deepseek.com/v1/chat/completions"
        );
        assert_eq!(
            openai_url("http://127.0.0.1:11434"),
            "http://127.0.0.1:11434/v1/chat/completions"
        );
        assert_eq!(
            openai_url("https://example.test/v1/chat/completions"),
            "https://example.test/v1/chat/completions"
        );
    }

    #[test]
    fn gemini_endpoint_appends_model() {
        assert_eq!(
            gemini_url("https://generativelanguage.googleapis.com/v1beta", "gemini-2.0-flash"),
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent"
        );
    }

    #[test]
    fn openai_body_keeps_system_prompt_first() {
        let config = AiConfig {
            model: "test".into(),
            ..AiConfig::default()
        };
        let messages = vec![
            ChatMessage {
                role: "user".into(),
                content: "hi".into(),
            },
            ChatMessage {
                role: "assistant".into(),
                content: "hello".into(),
            },
        ];
        let body = openai_body(&config, "be brief", &messages);
        let wire = body["messages"].as_array().unwrap();
        assert_eq!(wire[0]["role"], "system");
        assert_eq!(wire[1]["role"], "user");
        assert_eq!(wire[2]["role"], "assistant");
        assert_eq!(body["stream"], false);
    }

    #[test]
    fn gemini_body_uses_model_role_for_assistant() {
        let config = AiConfig::default();
        let messages = vec![ChatMessage {
            role: "assistant".into(),
            content: "hello".into(),
        }];
        let body = gemini_body(&config, "system", &messages);
        assert_eq!(body["contents"][0]["role"], "model");
        assert_eq!(body["systemInstruction"]["parts"][0]["text"], "system");
    }

    #[test]
    fn replies_are_extracted_from_both_shapes() {
        let openai = json!({ "choices": [{ "message": { "content": "hi" } }] });
        assert_eq!(text_from_openai(&openai).unwrap(), "hi");
        let gemini = json!({ "candidates": [{ "content": { "parts": [{ "text": "yo" }] } }] });
        assert_eq!(text_from_gemini(&gemini).unwrap(), "yo");
    }
}
