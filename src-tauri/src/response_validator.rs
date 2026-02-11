use crate::llm_service::LLMResponse;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationConstraints {
    pub require_json: bool,
    pub max_realm_level: Option<u32>,
    pub min_combat_power: Option<u64>,
    pub max_combat_power: Option<u64>,
    pub max_current_age: Option<u32>,
}

impl Default for ValidationConstraints {
    fn default() -> Self {
        Self {
            require_json: true,
            max_realm_level: None,
            min_combat_power: None,
            max_combat_power: None,
            max_current_age: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    EmptyResponse,
    InvalidJson(String),
    MissingField(String),
    NumericalConstraintViolation(String),
    RetryExhausted { attempts: u32, last_error: String },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::EmptyResponse => write!(f, "response text is empty"),
            ValidationError::InvalidJson(msg) => write!(f, "invalid json: {msg}"),
            ValidationError::MissingField(field) => write!(f, "missing required field: {field}"),
            ValidationError::NumericalConstraintViolation(msg) => {
                write!(f, "numerical constraint violation: {msg}")
            }
            ValidationError::RetryExhausted {
                attempts,
                last_error,
            } => {
                write!(
                    f,
                    "validation failed after {attempts} attempts, last error: {last_error}"
                )
            }
        }
    }
}

impl std::error::Error for ValidationError {}

#[derive(Debug, Clone)]
pub struct ResponseValidator {
    max_retries: u32,
}

impl ResponseValidator {
    pub fn new(max_retries: u32) -> Self {
        Self { max_retries }
    }

    pub fn validate_response(
        &self,
        response: &LLMResponse,
        constraints: &ValidationConstraints,
    ) -> Result<(), ValidationError> {
        if response.text.trim().is_empty() {
            return Err(ValidationError::EmptyResponse);
        }

        if !constraints.require_json {
            return Ok(());
        }

        let parsed = self.validate_json_format(&response.text)?;
        self.validate_numerical_constraints(&parsed, constraints)
    }

    pub fn validate_json_format(&self, response_text: &str) -> Result<Value, ValidationError> {
        serde_json::from_str::<Value>(response_text)
            .map_err(|e| ValidationError::InvalidJson(e.to_string()))
    }

    pub fn validate_numerical_constraints(
        &self,
        response_json: &Value,
        constraints: &ValidationConstraints,
    ) -> Result<(), ValidationError> {
        if let Some(max_level) = constraints.max_realm_level {
            let level = get_u32(response_json, &["/character_update/realm_level", "/realm_level"])
                .ok_or_else(|| ValidationError::MissingField("realm_level".to_string()))?;
            if level > max_level {
                return Err(ValidationError::NumericalConstraintViolation(format!(
                    "realm_level {level} exceeds max {max_level}"
                )));
            }
        }

        if let Some(min_power) = constraints.min_combat_power {
            let power =
                get_u64(response_json, &["/character_update/combat_power", "/combat_power"])
                    .ok_or_else(|| ValidationError::MissingField("combat_power".to_string()))?;
            if power < min_power {
                return Err(ValidationError::NumericalConstraintViolation(format!(
                    "combat_power {power} is below min {min_power}"
                )));
            }
        }

        if let Some(max_power) = constraints.max_combat_power {
            let power =
                get_u64(response_json, &["/character_update/combat_power", "/combat_power"])
                    .ok_or_else(|| ValidationError::MissingField("combat_power".to_string()))?;
            if power > max_power {
                return Err(ValidationError::NumericalConstraintViolation(format!(
                    "combat_power {power} exceeds max {max_power}"
                )));
            }
        }

        if let Some(max_age) = constraints.max_current_age {
            let age =
                get_u32(response_json, &["/character_update/current_age", "/current_age"])
                    .ok_or_else(|| ValidationError::MissingField("current_age".to_string()))?;
            if age > max_age {
                return Err(ValidationError::NumericalConstraintViolation(format!(
                    "current_age {age} exceeds max {max_age}"
                )));
            }
        }

        Ok(())
    }

    pub fn validate_with_retry_or_fallback<F>(
        &self,
        initial_response: LLMResponse,
        constraints: &ValidationConstraints,
        mut retry_provider: F,
        fallback_response: Option<LLMResponse>,
    ) -> Result<LLMResponse, ValidationError>
    where
        F: FnMut(u32) -> Option<LLMResponse>,
    {
        if self.validate_response(&initial_response, constraints).is_ok() {
            return Ok(initial_response);
        }

        let mut last_error: Option<ValidationError> = None;

        for attempt in 1..=self.max_retries {
            let next = match retry_provider(attempt) {
                Some(resp) => resp,
                None => continue,
            };

            match self.validate_response(&next, constraints) {
                Ok(()) => return Ok(next),
                Err(err) => last_error = Some(err),
            }
        }

        if let Some(fallback) = fallback_response {
            self.validate_response(&fallback, constraints)?;
            return Ok(fallback);
        }

        let last_error_message = last_error
            .map(|e| e.to_string())
            .unwrap_or_else(|| "no retry response available".to_string());

        Err(ValidationError::RetryExhausted {
            attempts: self.max_retries,
            last_error: last_error_message,
        })
    }
}

impl Default for ResponseValidator {
    fn default() -> Self {
        Self::new(3)
    }
}

fn get_u32(value: &Value, pointers: &[&str]) -> Option<u32> {
    pointers.iter().find_map(|path| {
        value
            .pointer(path)
            .and_then(Value::as_u64)
            .and_then(|v| u32::try_from(v).ok())
    })
}

fn get_u64(value: &Value, pointers: &[&str]) -> Option<u64> {
    pointers
        .iter()
        .find_map(|path| value.pointer(path).and_then(Value::as_u64))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use serde_json::json;

    fn build_response(text: String) -> LLMResponse {
        LLMResponse {
            text,
            model: Some("gpt-test".to_string()),
            finish_reason: Some("stop".to_string()),
            prompt_tokens: None,
            completion_tokens: None,
            total_tokens: None,
        }
    }

    fn strict_constraints() -> ValidationConstraints {
        ValidationConstraints {
            require_json: true,
            max_realm_level: Some(3),
            min_combat_power: Some(100),
            max_combat_power: Some(1000),
            max_current_age: Some(200),
        }
    }

    #[test]
    fn test_validate_response_success() {
        let validator = ResponseValidator::default();
        let response = build_response(
            json!({
                "character_update": {
                    "realm_level": 2,
                    "combat_power": 350,
                    "current_age": 28
                }
            })
            .to_string(),
        );

        assert!(validator
            .validate_response(&response, &strict_constraints())
            .is_ok());
    }

    #[test]
    fn test_validate_response_rejects_invalid_json() {
        let validator = ResponseValidator::default();
        let response = build_response("{invalid json}".to_string());

        let result = validator.validate_response(&response, &strict_constraints());
        assert!(matches!(result, Err(ValidationError::InvalidJson(_))));
    }

    #[test]
    fn test_validate_response_rejects_numerical_violation() {
        let validator = ResponseValidator::default();
        let response = build_response(
            json!({
                "character_update": {
                    "realm_level": 9,
                    "combat_power": 350,
                    "current_age": 28
                }
            })
            .to_string(),
        );

        let result = validator.validate_response(&response, &strict_constraints());
        assert!(matches!(
            result,
            Err(ValidationError::NumericalConstraintViolation(_))
        ));
    }

    #[test]
    fn test_validate_with_retry_returns_valid_retry_response() {
        let validator = ResponseValidator::new(2);
        let constraints = strict_constraints();

        let initial = build_response("not json".to_string());

        let result = validator.validate_with_retry_or_fallback(
            initial,
            &constraints,
            |attempt| {
                if attempt == 1 {
                    Some(build_response(
                        json!({
                            "character_update": {
                                "realm_level": 2,
                                "combat_power": 320,
                                "current_age": 31
                            }
                        })
                        .to_string(),
                    ))
                } else {
                    None
                }
            },
            None,
        );

        assert!(result.is_ok());
        assert!(result.unwrap().text.contains("realm_level"));
    }

    #[test]
    fn test_validate_with_retry_uses_fallback() {
        let validator = ResponseValidator::new(1);
        let constraints = strict_constraints();

        let initial = build_response("not json".to_string());
        let fallback = build_response(
            json!({
                "character_update": {
                    "realm_level": 1,
                    "combat_power": 200,
                    "current_age": 18
                }
            })
            .to_string(),
        );

        let result = validator.validate_with_retry_or_fallback(
            initial,
            &constraints,
            |_attempt| Some(build_response("still invalid".to_string())),
            Some(fallback.clone()),
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), fallback);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        // Feature: Nobody, Property 4: LLM output numerical constraints
        #[test]
        fn prop_rejects_responses_violating_numerical_constraints(
            max_level in 1u32..=8,
            over_level_delta in 1u32..=8,
            valid_power in 100u64..=1000,
            valid_age in 1u32..=200
        ) {
            let validator = ResponseValidator::default();
            let constraints = ValidationConstraints {
                require_json: true,
                max_realm_level: Some(max_level),
                min_combat_power: Some(100),
                max_combat_power: Some(1000),
                max_current_age: Some(200),
            };

            let invalid_level = max_level.saturating_add(over_level_delta);
            let response = build_response(
                json!({
                    "character_update": {
                        "realm_level": invalid_level,
                        "combat_power": valid_power,
                        "current_age": valid_age
                    }
                }).to_string()
            );

            let result = validator.validate_response(&response, &constraints);
            prop_assert!(matches!(result, Err(ValidationError::NumericalConstraintViolation(_))));
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        // Feature: Nobody, Property 31: LLM response validation retry
        #[test]
        fn prop_retry_or_fallback_keeps_system_running(
            valid_realm in 0u32..=3,
            valid_power in 100u64..=800,
            valid_age in 1u32..=150
        ) {
            let validator = ResponseValidator::new(2);
            let constraints = strict_constraints();

            let initial = build_response("bad-json".to_string());
            let fallback = build_response(
                json!({
                    "character_update": {
                        "realm_level": valid_realm,
                        "combat_power": valid_power,
                        "current_age": valid_age
                    }
                }).to_string()
            );

            let result = validator.validate_with_retry_or_fallback(
                initial,
                &constraints,
                |_attempt| Some(build_response("still-bad-json".to_string())),
                Some(fallback.clone()),
            );

            prop_assert!(result.is_ok());
            prop_assert_eq!(result.unwrap(), fallback);
        }
    }
}
