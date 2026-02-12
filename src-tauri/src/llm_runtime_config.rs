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
        return LLMConfigStatus {
            configured: true,
            source: "runtime".to_string(),
            endpoint: Some(cfg.endpoint),
            model: Some(cfg.model),
            max_tokens: Some(cfg.max_tokens),
            temperature: Some(cfg.temperature),
        };
    }

    if let Some(cfg) = load_llm_config_from_file() {
        return LLMConfigStatus {
            configured: true,
            source: "file".to_string(),
            endpoint: Some(cfg.endpoint),
            model: Some(cfg.model),
            max_tokens: Some(cfg.max_tokens),
            temperature: Some(cfg.temperature),
        };
    }

    if let Some(cfg) = load_llm_config_from_env() {
        return LLMConfigStatus {
            configured: true,
            source: "env".to_string(),
            endpoint: Some(cfg.endpoint),
            model: Some(cfg.model),
            max_tokens: Some(cfg.max_tokens),
            temperature: Some(cfg.temperature),
        };
    }

    LLMConfigStatus {
        configured: false,
        source: "none".to_string(),
        endpoint: None,
        model: None,
        max_tokens: None,
        temperature: None,
    }
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
