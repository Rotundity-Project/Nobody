use crate::noname_errors::NoNameConfigError;
use crate::noname_types::NoNameMode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameTokenBudget {
    pub total: usize,
    pub context_reserved: usize,
    pub planning_reserved: usize,
    pub tool_reserved: usize,
    pub response_reserved: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameTimeoutPolicy {
    pub planning_timeout_ms: u64,
    pub tool_timeout_ms: u64,
    pub total_turn_timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameCapabilityPolicy {
    #[serde(default)]
    pub whitelist: Vec<String>,
    pub allow_prompt_capabilities: bool,
    pub allow_resource_capabilities: bool,
    pub allow_tool_capabilities: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameTracePolicy {
    pub enabled: bool,
    pub max_recent_traces: usize,
    pub include_payload_preview: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameConfig {
    pub mode: NoNameMode,
    pub token_budget: NoNameTokenBudget,
    pub timeout_policy: NoNameTimeoutPolicy,
    pub capability_policy: NoNameCapabilityPolicy,
    pub trace_policy: NoNameTracePolicy,
}

impl Default for NoNameConfig {
    fn default() -> Self {
        Self::disabled()
    }
}

impl NoNameConfig {
    pub fn from_mode(mode: NoNameMode) -> Self {
        match mode {
            NoNameMode::Disabled => Self::disabled(),
            NoNameMode::ObserveOnly => Self::observe_only(),
            NoNameMode::Assisted => Self::assisted(),
        }
    }

    pub fn disabled() -> Self {
        Self {
            mode: NoNameMode::Disabled,
            token_budget: NoNameTokenBudget {
                total: 1200,
                context_reserved: 600,
                planning_reserved: 240,
                tool_reserved: 180,
                response_reserved: 180,
            },
            timeout_policy: NoNameTimeoutPolicy {
                planning_timeout_ms: 1_500,
                tool_timeout_ms: 1_000,
                total_turn_timeout_ms: 4_000,
            },
            capability_policy: NoNameCapabilityPolicy {
                whitelist: Vec::new(),
                allow_prompt_capabilities: true,
                allow_resource_capabilities: true,
                allow_tool_capabilities: true,
            },
            trace_policy: NoNameTracePolicy {
                enabled: false,
                max_recent_traces: 24,
                include_payload_preview: false,
            },
        }
    }

    pub fn observe_only() -> Self {
        let mut config = Self::disabled();
        config.mode = NoNameMode::ObserveOnly;
        config.trace_policy.enabled = true;
        config.trace_policy.include_payload_preview = true;
        config
    }

    pub fn assisted() -> Self {
        let mut config = Self::observe_only();
        config.mode = NoNameMode::Assisted;
        config.timeout_policy.total_turn_timeout_ms = 5_000;
        config
    }

    pub fn validate(&self) -> Result<(), NoNameConfigError> {
        let reserved_sum = self.token_budget.context_reserved
            + self.token_budget.planning_reserved
            + self.token_budget.tool_reserved
            + self.token_budget.response_reserved;

        if reserved_sum > self.token_budget.total {
            return Err(NoNameConfigError::new(
                "token reserved sum exceeds total budget",
                "noname.config.invalid_budget",
                true,
            ));
        }

        if self.timeout_policy.planning_timeout_ms == 0
            || self.timeout_policy.tool_timeout_ms == 0
            || self.timeout_policy.total_turn_timeout_ms == 0
        {
            return Err(NoNameConfigError::new(
                "timeout policy must be greater than zero",
                "noname.config.invalid_timeout",
                true,
            ));
        }

        if self.trace_policy.max_recent_traces == 0 {
            return Err(NoNameConfigError::new(
                "trace retention must be greater than zero",
                "noname.config.invalid_trace_retention",
                true,
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_disabled() {
        let config = NoNameConfig::default();
        assert_eq!(config.mode, NoNameMode::Disabled);
        assert!(!config.trace_policy.enabled);
    }

    #[test]
    fn observe_only_preset_enables_trace() {
        let config = NoNameConfig::observe_only();
        assert_eq!(config.mode, NoNameMode::ObserveOnly);
        assert!(config.trace_policy.enabled);
    }

    #[test]
    fn invalid_budget_fails_validation() {
        let mut config = NoNameConfig::observe_only();
        config.token_budget.total = 10;

        let err = config.validate().expect_err("budget should fail");
        assert_eq!(err.code, "noname.config.invalid_budget");
    }

    #[test]
    fn invalid_timeout_fails_validation() {
        let mut config = NoNameConfig::observe_only();
        config.timeout_policy.planning_timeout_ms = 0;

        let err = config.validate().expect_err("timeout should fail");
        assert_eq!(err.code, "noname.config.invalid_timeout");
    }

    #[test]
    fn from_mode_builds_assisted_preset() {
        let config = NoNameConfig::from_mode(NoNameMode::Assisted);
        assert_eq!(config.mode, NoNameMode::Assisted);
        assert!(config.trace_policy.enabled);
    }
}
