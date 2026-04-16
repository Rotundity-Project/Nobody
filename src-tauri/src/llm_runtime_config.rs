use crate::llm_service::LLMConfig;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

static RUNTIME_LLM_CONFIG: OnceLock<Mutex<Option<LLMConfig>>> = OnceLock::new();

fn config_slot() -> &'static Mutex<Option<LLMConfig>> {
    RUNTIME_LLM_CONFIG.get_or_init(|| Mutex::new(None))
}

fn config_file_path() -> PathBuf {
    PathBuf::from(".nobody_llm_config.json")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfigStatus {
    pub configured: bool,
    pub source: String,
    pub endpoint: Option<String>,
    pub model: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub api_key_saved: bool,
    pub api_key_hint: Option<String>,
}

pub fn set_runtime_llm_config(config: LLMConfig) {
    let mut guard = config_slot().lock().unwrap();
    *guard = Some(config.clone());
    drop(guard);
    let _ = persist_llm_config_to_disk(&config);
}

pub fn clear_runtime_llm_config() {
    let mut guard = config_slot().lock().unwrap();
    *guard = None;
    let _ = remove_llm_config_file();
}

pub fn get_runtime_llm_config() -> Option<LLMConfig> {
    let guard = config_slot().lock().unwrap();
    guard.clone()
}

pub fn resolve_llm_config() -> Option<LLMConfig> {
    get_runtime_llm_config()
        .or_else(load_llm_config_from_file)
        .or_else(load_llm_config_from_env)
}

pub fn get_llm_config_status() -> LLMConfigStatus {
    if let Some(cfg) = get_runtime_llm_config() {
        return status_from_config(&cfg, "runtime");
    }

    if let Some(cfg) = load_llm_config_from_file() {
        return status_from_config(&cfg, "file");
    }

    if let Some(cfg) = load_llm_config_from_env() {
        return status_from_config(&cfg, "env");
    }

    LLMConfigStatus {
        configured: false,
        source: "none".to_string(),
        endpoint: None,
        model: None,
        max_tokens: None,
        temperature: None,
        api_key_saved: false,
        api_key_hint: None,
    }
}

fn status_from_config(cfg: &LLMConfig, source: &str) -> LLMConfigStatus {
    let api_key_hint = mask_api_key(&cfg.api_key);
    LLMConfigStatus {
        configured: true,
        source: source.to_string(),
        endpoint: Some(cfg.endpoint.clone()),
        model: Some(cfg.model.clone()),
        max_tokens: Some(cfg.max_tokens),
        temperature: Some(cfg.temperature),
        api_key_saved: api_key_hint.is_some(),
        api_key_hint,
    }
}

fn mask_api_key(api_key: &str) -> Option<String> {
    let trimmed = api_key.trim();
    if trimmed.is_empty() {
        return None;
    }
    let suffix = trimmed
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<String>();
    Some(format!("***{}", suffix))
}

fn load_llm_config_from_env() -> Option<LLMConfig> {
    let endpoint = std::env::var("NOBODY_LLM_ENDPOINT").ok()?;
    let api_key = std::env::var("NOBODY_LLM_API_KEY").ok()?;
    let model = std::env::var("NOBODY_LLM_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
    let max_tokens = std::env::var("NOBODY_LLM_MAX_TOKENS")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(1024);
    let temperature = std::env::var("NOBODY_LLM_TEMPERATURE")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(0.7);

    Some(LLMConfig {
        endpoint,
        api_key,
        model,
        max_tokens,
        temperature,
    })
}

fn load_llm_config_from_file() -> Option<LLMConfig> {
    let path = config_file_path();
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str::<LLMConfig>(&content).ok()
}

fn persist_llm_config_to_disk(cfg: &LLMConfig) -> Result<(), String> {
    let content = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    fs::write(config_file_path(), content).map_err(|e| e.to_string())
}

fn remove_llm_config_file() -> Result<(), String> {
    let path = config_file_path();
    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_api_key_returns_last_four_hint() {
        assert_eq!(
            mask_api_key("dummy-api-key-1234"),
            Some("***1234".to_string())
        );
        assert_eq!(mask_api_key(""), None);
    }

    #[test]
    fn test_status_from_config_marks_saved_api_key() {
        let cfg = LLMConfig {
            endpoint: "https://api.example.com/v1/chat/completions".to_string(),
            api_key: "secret-key".to_string(),
            model: "demo-model".to_string(),
            max_tokens: 512,
            temperature: 0.7,
        };

        let status = status_from_config(&cfg, "file");
        assert!(status.api_key_saved);
        assert_eq!(status.api_key_hint, Some("***-key".to_string()));
    }
}
