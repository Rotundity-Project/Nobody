use crate::app_error::{AppError, AppErrorKind};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoNameErrorKind {
    Config,
    Protocol,
    Capability,
    Memory,
    Context,
    Guardrail,
    Runtime,
    Trace,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameError {
    pub kind: NoNameErrorKind,
    pub message: String,
    pub code: String,
    pub recoverable: bool,
}

impl NoNameError {
    pub fn new(
        kind: NoNameErrorKind,
        message: impl Into<String>,
        code: impl Into<String>,
        recoverable: bool,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
            code: code.into(),
            recoverable,
        }
    }
}

impl fmt::Display for NoNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.message, self.code)
    }
}

impl std::error::Error for NoNameError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameConfigError {
    pub message: String,
    pub code: String,
    pub recoverable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameProtocolError {
    pub message: String,
    pub code: String,
    pub recoverable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameCapabilityError {
    pub message: String,
    pub code: String,
    pub recoverable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameMemoryError {
    pub message: String,
    pub code: String,
    pub recoverable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameContextError {
    pub message: String,
    pub code: String,
    pub recoverable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameGuardrailError {
    pub message: String,
    pub code: String,
    pub recoverable: bool,
}

macro_rules! impl_domain_error {
    ($name:ident, $kind:expr) => {
        impl $name {
            pub fn new(
                message: impl Into<String>,
                code: impl Into<String>,
                recoverable: bool,
            ) -> Self {
                Self {
                    message: message.into(),
                    code: code.into(),
                    recoverable,
                }
            }
        }

        impl From<$name> for NoNameError {
            fn from(err: $name) -> Self {
                NoNameError::new($kind, err.message, err.code, err.recoverable)
            }
        }
    };
}

impl_domain_error!(NoNameConfigError, NoNameErrorKind::Config);
impl_domain_error!(NoNameProtocolError, NoNameErrorKind::Protocol);
impl_domain_error!(NoNameCapabilityError, NoNameErrorKind::Capability);
impl_domain_error!(NoNameMemoryError, NoNameErrorKind::Memory);
impl_domain_error!(NoNameContextError, NoNameErrorKind::Context);
impl_domain_error!(NoNameGuardrailError, NoNameErrorKind::Guardrail);

impl From<NoNameCapabilityError> for AppError {
    fn from(err: NoNameCapabilityError) -> Self {
        let top_level: NoNameError = err.into();
        top_level.into()
    }
}

impl From<NoNameProtocolError> for AppError {
    fn from(err: NoNameProtocolError) -> Self {
        let top_level: NoNameError = err.into();
        top_level.into()
    }
}

impl From<NoNameError> for AppError {
    fn from(err: NoNameError) -> Self {
        let kind = match err.kind {
            NoNameErrorKind::Config | NoNameErrorKind::Protocol | NoNameErrorKind::Context => {
                AppErrorKind::InvalidInput
            }
            NoNameErrorKind::Capability
            | NoNameErrorKind::Memory
            | NoNameErrorKind::Guardrail
            | NoNameErrorKind::Runtime
            | NoNameErrorKind::Trace
            | NoNameErrorKind::Unknown => AppErrorKind::Unknown,
        };

        AppError::new(kind, format!("[{}] {}", err.code, err.message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_error_maps_to_top_level_error() {
        let err =
            NoNameConfigError::new("invalid token budget", "noname.config.invalid_budget", true);
        let mapped: NoNameError = err.into();

        assert_eq!(mapped.kind, NoNameErrorKind::Config);
        assert!(mapped.recoverable);
    }

    #[test]
    fn protocol_error_is_recoverable() {
        let err =
            NoNameProtocolError::new("unsupported message", "noname.protocol.unsupported", true);
        let mapped: NoNameError = err.into();

        assert_eq!(mapped.kind, NoNameErrorKind::Protocol);
        assert!(mapped.recoverable);
    }

    #[test]
    fn guardrail_error_can_convert_to_app_error() {
        let err =
            NoNameGuardrailError::new("proposal rejected", "noname.guardrail.rejected", false);
        let top_level: NoNameError = err.into();
        let app_error: AppError = top_level.into();

        assert_eq!(app_error.kind, AppErrorKind::Unknown);
        assert!(app_error.message.contains("proposal rejected"));
    }
}
