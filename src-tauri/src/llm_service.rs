use crate::prompt_builder::estimate_token_count;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;
use std::time::{Duration, Instant};

const DEFAULT_TIMEOUT_SECS: u64 = 30;
const DEFAULT_CACHE_MAX_ENTRIES: usize = 512;
const DEFAULT_CACHE_TTL_SECS: u64 = 600;
const DEFAULT_MAX_RETRIES: u32 = 2;
const DEFAULT_RETRY_BACKOFF_MS: u64 = 200;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LLMConfig {
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

impl LLMConfig {
    pub fn validate(&self) -> Result<(), LLMServiceError> {
        if self.endpoint.trim().is_empty() {
            return Err(LLMServiceError::InvalidConfig(
                "endpoint must not be empty".to_string(),
            ));
        }
        if self.api_key.trim().is_empty() {
            return Err(LLMServiceError::InvalidConfig(
                "api_key must not be empty".to_string(),
            ));
        }
        if self.model.trim().is_empty() {
            return Err(LLMServiceError::InvalidConfig(
                "model must not be empty".to_string(),
            ));
        }
        if self.max_tokens == 0 {
            return Err(LLMServiceError::InvalidConfig(
                "max_tokens must be greater than 0".to_string(),
            ));
        }
        if !(0.0..=2.0).contains(&self.temperature) {
            return Err(LLMServiceError::InvalidConfig(
                "temperature must be in range [0.0, 2.0]".to_string(),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LLMRequest {
    pub prompt: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LLMResponse {
    pub text: String,
    pub model: Option<String>,
    pub finish_reason: Option<String>,
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}

#[derive(Debug)]
pub enum LLMServiceError {
    InvalidConfig(String),
    InvalidRequest(String),
    Http(reqwest::Error),
    Api(String),
    InvalidResponse(String),
    Timeout,
}

impl fmt::Display for LLMServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LLMServiceError::InvalidConfig(msg) => write!(f, "invalid config: {msg}"),
            LLMServiceError::InvalidRequest(msg) => write!(f, "invalid request: {msg}"),
            LLMServiceError::Http(err) => write!(f, "http request failed: {err}"),
            LLMServiceError::Api(msg) => write!(f, "llm api returned error: {msg}"),
            LLMServiceError::InvalidResponse(msg) => write!(f, "invalid llm response: {msg}"),
            LLMServiceError::Timeout => write!(f, "llm request timed out"),
        }
    }
}

impl std::error::Error for LLMServiceError {}

impl From<reqwest::Error> for LLMServiceError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            LLMServiceError::Timeout
        } else {
            LLMServiceError::Http(err)
        }
    }
}

pub struct LLMService {
    pub api_config: LLMConfig,
    client: Client,
    cache: Mutex<ResponseCache>,
}

impl LLMService {
    pub fn new(api_config: LLMConfig) -> Result<Self, LLMServiceError> {
        api_config.validate()?;

        let client = Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .map_err(LLMServiceError::Http)?;

        let cache = Mutex::new(ResponseCache::new(
            DEFAULT_CACHE_MAX_ENTRIES,
            Duration::from_secs(DEFAULT_CACHE_TTL_SECS),
        ));

        Ok(Self {
            api_config,
            client,
            cache,
        })
    }

    pub async fn generate(&self, request: LLMRequest) -> Result<LLMResponse, LLMServiceError> {
        if request.prompt.trim().is_empty() {
            return Err(LLMServiceError::InvalidRequest(
                "prompt must not be empty".to_string(),
            ));
        }

        let max_tokens = request
            .max_tokens
            .unwrap_or(self.api_config.max_tokens)
            .min(self.api_config.max_tokens);
        if max_tokens == 0 {
            return Err(LLMServiceError::InvalidRequest(
                "max_tokens must be greater than 0".to_string(),
            ));
        }

        let temperature = request.temperature.unwrap_or(self.api_config.temperature);
        if !(0.0..=2.0).contains(&temperature) {
            return Err(LLMServiceError::InvalidRequest(
                "temperature must be in range [0.0, 2.0]".to_string(),
            ));
        }

        let estimated_prompt_tokens = estimate_token_count(&request.prompt);
        let prompt_limit = max_tokens.saturating_mul(64);
        if estimated_prompt_tokens > prompt_limit {
            return Err(LLMServiceError::InvalidRequest(format!(
                "estimated prompt tokens {} exceeds prompt limit {}",
                estimated_prompt_tokens, prompt_limit
            )));
        }

        let request_hash = self.build_request_hash(&request.prompt, max_tokens, temperature);
        if let Some(cached) = self.get_cached_response(&request_hash) {
            return Ok(cached);
        }

        let payload = json!({
            "model": self.api_config.model,
            "messages": [
                { "role": "user", "content": request.prompt }
            ],
            "max_tokens": max_tokens,
            "temperature": temperature,
            "stream": false
        });

        let mut attempt = 0;
        loop {
            attempt += 1;
            let response = self
                .client
                .post(&self.api_config.endpoint)
                .bearer_auth(&self.api_config.api_key)
                .json(&payload)
                .send()
                .await;

            let response = match response {
                Ok(resp) => resp,
                Err(err) => {
                    let err = LLMServiceError::from(err);
                    if attempt <= DEFAULT_MAX_RETRIES && is_retryable_error(&err) {
                        backoff_sleep(attempt).await;
                        continue;
                    }
                    return Err(err);
                }
            };

            if !response.status().is_success() {
                let status = response.status();
                let body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "failed to read error response body".to_string());
                let err = LLMServiceError::Api(format!("status={status} body={body}"));
                if attempt <= DEFAULT_MAX_RETRIES && is_retryable_status(status.as_u16()) {
                    backoff_sleep(attempt).await;
                    continue;
                }
                return Err(err);
            }

            let value: Value = response.json().await?;
            let parsed = Self::parse_response(value)?;
            self.cache_response(&request_hash, &parsed);
            return Ok(parsed);
        }
    }

    pub fn cache_response(&self, request_hash: &str, response: &LLMResponse) {
        self.with_cache(|cache| {
            cache.insert(request_hash.to_string(), response.clone());
        });
    }

    pub fn cache_response_for_request(&self, request: &LLMRequest, response: &LLMResponse) {
        let max_tokens = request.max_tokens.unwrap_or(self.api_config.max_tokens);
        let temperature = request.temperature.unwrap_or(self.api_config.temperature);
        let request_hash = self.build_request_hash(&request.prompt, max_tokens, temperature);
        self.cache_response(&request_hash, response);
    }

    pub fn get_cached_response(&self, request_hash: &str) -> Option<LLMResponse> {
        self.with_cache(|cache| cache.get(request_hash))
    }

    fn build_request_hash(&self, prompt: &str, max_tokens: u32, temperature: f32) -> String {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.api_config.endpoint.hash(&mut hasher);
        self.api_config.model.hash(&mut hasher);
        prompt.hash(&mut hasher);
        max_tokens.hash(&mut hasher);
        temperature.to_bits().hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    fn with_cache<T>(&self, f: impl FnOnce(&mut ResponseCache) -> T) -> T {
        match self.cache.lock() {
            Ok(mut guard) => f(&mut guard),
            Err(poisoned) => {
                let mut guard = poisoned.into_inner();
                f(&mut guard)
            }
        }
    }

    fn parse_response(value: Value) -> Result<LLMResponse, LLMServiceError> {
        let text = value
            .pointer("/choices/0/message/content")
            .and_then(Value::as_str)
            .or_else(|| value.pointer("/choices/0/text").and_then(Value::as_str))
            .or_else(|| value.get("output_text").and_then(Value::as_str))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                LLMServiceError::InvalidResponse("unable to locate text content".to_string())
            })?;

        let model = value
            .get("model")
            .and_then(Value::as_str)
            .map(|s| s.to_string());
        let finish_reason = value
            .pointer("/choices/0/finish_reason")
            .and_then(Value::as_str)
            .map(|s| s.to_string());

        let prompt_tokens = value
            .pointer("/usage/prompt_tokens")
            .and_then(Value::as_u64)
            .and_then(|v| u32::try_from(v).ok());
        let completion_tokens = value
            .pointer("/usage/completion_tokens")
            .and_then(Value::as_u64)
            .and_then(|v| u32::try_from(v).ok());
        let total_tokens = value
            .pointer("/usage/total_tokens")
            .and_then(Value::as_u64)
            .and_then(|v| u32::try_from(v).ok());

        Ok(LLMResponse {
            text,
            model,
            finish_reason,
            prompt_tokens,
            completion_tokens,
            total_tokens,
        })
    }
}

fn is_retryable_status(status: u16) -> bool {
    status == 429 || (500..=599).contains(&status)
}

fn is_retryable_error(err: &LLMServiceError) -> bool {
    match err {
        LLMServiceError::Timeout => true,
        LLMServiceError::Http(_) => true,
        LLMServiceError::Api(msg) => {
            let status = msg
                .split_whitespace()
                .find_map(|chunk| {
                    if let Some(rest) = chunk.strip_prefix("status=") {
                        rest.trim_end_matches(|c: char| c == ',' || c == ';')
                            .parse::<u16>()
                            .ok()
                    } else {
                        None
                    }
                })
                .unwrap_or(0);
            is_retryable_status(status)
        }
        _ => false,
    }
}

async fn backoff_sleep(attempt: u32) {
    let backoff = DEFAULT_RETRY_BACKOFF_MS.saturating_mul(attempt as u64);
    tokio::time::sleep(Duration::from_millis(backoff)).await;
}

#[derive(Debug, Clone)]
pub struct ResponseCache {
    entries: HashMap<String, CacheEntry>,
    max_entries: usize,
    ttl: Duration,
    access_counter: u64,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    response: LLMResponse,
    cached_at: Instant,
    last_access: u64,
}

impl ResponseCache {
    fn new(max_entries: usize, ttl: Duration) -> Self {
        Self {
            entries: HashMap::new(),
            max_entries: max_entries.max(1),
            ttl,
            access_counter: 0,
        }
    }

    fn insert(&mut self, key: String, response: LLMResponse) {
        self.purge_expired();

        if !self.entries.contains_key(&key) && self.entries.len() >= self.max_entries {
            self.evict_lru();
        }

        let tick = self.next_tick();
        self.entries.insert(
            key,
            CacheEntry {
                response,
                cached_at: Instant::now(),
                last_access: tick,
            },
        );
    }

    fn get(&mut self, key: &str) -> Option<LLMResponse> {
        self.purge_expired();
        let tick = self.next_tick();

        self.entries.get_mut(key).map(|entry| {
            entry.last_access = tick;
            entry.response.clone()
        })
    }

    fn next_tick(&mut self) -> u64 {
        self.access_counter = self.access_counter.saturating_add(1);
        self.access_counter
    }

    fn purge_expired(&mut self) {
        let ttl = self.ttl;
        self.entries.retain(|_, entry| entry.cached_at.elapsed() <= ttl);
    }

    fn evict_lru(&mut self) {
        if let Some(oldest_key) = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.last_access)
            .map(|(k, _)| k.clone())
        {
            self.entries.remove(&oldest_key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use serde_json::json;
    use std::thread::sleep;

    fn valid_config() -> LLMConfig {
        LLMConfig {
            endpoint: "https://example.com/v1/chat/completions".to_string(),
            api_key: "test-key".to_string(),
            model: "gpt-test".to_string(),
            max_tokens: 512,
            temperature: 0.7,
        }
    }

    #[test]
    fn test_config_validation_success() {
        let cfg = valid_config();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_config_validation_failure() {
        let mut cfg = valid_config();
        cfg.endpoint = String::new();
        assert!(cfg.validate().is_err());

        cfg = valid_config();
        cfg.temperature = 2.5;
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn test_parse_chat_style_response() {
        let raw = json!({
            "id": "abc",
            "model": "gpt-test",
            "choices": [
                {
                    "message": { "role": "assistant", "content": "hello cultivator" },
                    "finish_reason": "stop"
                }
            ],
            "usage": {
                "prompt_tokens": 12,
                "completion_tokens": 8,
                "total_tokens": 20
            }
        });

        let parsed = LLMService::parse_response(raw).unwrap();
        assert_eq!(parsed.text, "hello cultivator");
        assert_eq!(parsed.model, Some("gpt-test".to_string()));
        assert_eq!(parsed.total_tokens, Some(20));
    }

    #[test]
    fn test_parse_completion_style_response() {
        let raw = json!({
            "model": "gpt-test",
            "choices": [
                {
                    "text": "plain completion response",
                    "finish_reason": "stop"
                }
            ]
        });

        let parsed = LLMService::parse_response(raw).unwrap();
        assert_eq!(parsed.text, "plain completion response");
    }

    #[test]
    fn test_parse_response_without_text_fails() {
        let raw = json!({
            "model": "gpt-test",
            "choices": [
                {
                    "message": { "role": "assistant", "content": "   " },
                    "finish_reason": "stop"
                }
            ]
        });

        let result = LLMService::parse_response(raw);
        assert!(matches!(result, Err(LLMServiceError::InvalidResponse(_))));
    }

    #[tokio::test]
    async fn test_generate_rejects_prompt_over_max_tokens() {
        let service = LLMService::new(valid_config()).unwrap();
        let long_prompt = (0..300).map(|_| "token").collect::<Vec<_>>().join(" ");
        let request = LLMRequest {
            prompt: long_prompt,
            max_tokens: Some(3),
            temperature: Some(0.7),
        };

        let result = service.generate(request).await;
        assert!(matches!(result, Err(LLMServiceError::InvalidRequest(_))));
    }

    #[test]
    fn test_cache_hit_returns_cached_response() {
        let service = LLMService::new(valid_config()).unwrap();
        let response = LLMResponse {
            text: "cached".to_string(),
            model: Some("gpt-test".to_string()),
            finish_reason: Some("stop".to_string()),
            prompt_tokens: Some(1),
            completion_tokens: Some(1),
            total_tokens: Some(2),
        };

        service.cache_response("k1", &response);
        let cached = service.get_cached_response("k1");
        assert_eq!(cached, Some(response));
    }

    #[test]
    fn test_cache_expiry_removes_entry() {
        let mut cache = ResponseCache::new(10, Duration::from_millis(10));
        cache.insert(
            "k1".to_string(),
            LLMResponse {
                text: "cached".to_string(),
                model: None,
                finish_reason: None,
                prompt_tokens: None,
                completion_tokens: None,
                total_tokens: None,
            },
        );

        sleep(Duration::from_millis(20));
        assert!(cache.get("k1").is_none());
    }

    #[test]
    fn test_cache_lru_eviction() {
        let mut cache = ResponseCache::new(2, Duration::from_secs(60));

        cache.insert(
            "a".to_string(),
            LLMResponse {
                text: "a".to_string(),
                model: None,
                finish_reason: None,
                prompt_tokens: None,
                completion_tokens: None,
                total_tokens: None,
            },
        );
        cache.insert(
            "b".to_string(),
            LLMResponse {
                text: "b".to_string(),
                model: None,
                finish_reason: None,
                prompt_tokens: None,
                completion_tokens: None,
                total_tokens: None,
            },
        );

        let _ = cache.get("a");
        cache.insert(
            "c".to_string(),
            LLMResponse {
                text: "c".to_string(),
                model: None,
                finish_reason: None,
                prompt_tokens: None,
                completion_tokens: None,
                total_tokens: None,
            },
        );

        assert!(cache.get("a").is_some());
        assert!(cache.get("b").is_none());
        assert!(cache.get("c").is_some());
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        // Feature: Nobody, Property 32: LLM response cache hit
        #[test]
        fn prop_same_request_key_hits_cached_response(
            key in "[a-zA-Z0-9_-]{1,24}",
            text in "[a-zA-Z0-9 _.-]{1,48}"
        ) {
            let mut cache = ResponseCache::new(16, Duration::from_secs(60));
            let response = LLMResponse {
                text: text.clone(),
                model: Some("gpt-test".to_string()),
                finish_reason: Some("stop".to_string()),
                prompt_tokens: Some(1),
                completion_tokens: Some(1),
                total_tokens: Some(2),
            };

            cache.insert(key.clone(), response.clone());
            let cached = cache.get(&key);

            prop_assert_eq!(cached, Some(response));
        }
    }

    #[test]
    fn test_retryable_status_detection() {
        assert!(is_retryable_status(500));
        assert!(is_retryable_status(503));
        assert!(is_retryable_status(429));
        assert!(!is_retryable_status(400));
        assert!(!is_retryable_status(401));
    }

    #[test]
    fn test_retryable_error_detection() {
        let api_err = LLMServiceError::Api("status=503 body=oops".to_string());
        assert!(is_retryable_error(&api_err));
        let api_err = LLMServiceError::Api("status=400 body=bad".to_string());
        assert!(!is_retryable_error(&api_err));
        let timeout = LLMServiceError::Timeout;
        assert!(is_retryable_error(&timeout));
    }
}
