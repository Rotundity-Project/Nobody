#[cfg(test)]
mod tests {
    use crate::llm_service::{LLMConfig, LLMServiceError};

    #[test]
    fn test_llm_config_validation() {
        // Test valid config
        let config = LLMConfig {
            endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            api_key: "sk-test".to_string(),
            model: "gpt-4".to_string(),
            max_tokens: 2000,
            temperature: 0.7,
        };
        assert!(config.validate().is_ok());

        // Test empty endpoint
        let mut invalid_config = config.clone();
        invalid_config.endpoint = "".to_string();
        assert!(invalid_config.validate().is_err());

        // Test empty api_key
        invalid_config = config.clone();
        invalid_config.api_key = "".to_string();
        assert!(invalid_config.validate().is_err());

        // Test empty model
        invalid_config = config.clone();
        invalid_config.model = "".to_string();
        assert!(invalid_config.validate().is_err());

        // Test invalid max_tokens
        invalid_config = config.clone();
        invalid_config.max_tokens = 0;
        assert!(invalid_config.validate().is_err());

        // Test temperature out of range
        invalid_config = config.clone();
        invalid_config.temperature = 2.5;
        assert!(invalid_config.validate().is_err());

        invalid_config.temperature = -0.1;
        assert!(invalid_config.validate().is_err());
    }
}
