use crate::llm_service::LLMServiceError;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppErrorKind {
    NotFound,
    InvalidInput,
    Llm,
    Io,
    Parse,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppError {
    pub kind: AppErrorKind,
    pub message: String,
}

impl AppError {
    pub fn new(kind: AppErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn with_context(self, context: impl Into<String>) -> Self {
        let ctx = context.into();
        let message = if self.message.is_empty() {
            ctx
        } else {
            format!("{}: {}", ctx, self.message)
        };
        Self {
            kind: self.kind,
            message,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AppError {}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::new(AppErrorKind::Unknown, err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::new(AppErrorKind::Io, err.to_string())
    }
}

impl From<LLMServiceError> for AppError {
    fn from(err: LLMServiceError) -> Self {
        AppError::new(AppErrorKind::Llm, err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::new(AppErrorKind::Parse, err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_context() {
        let err = AppError::new(AppErrorKind::InvalidInput, "missing field")
            .with_context("load script");
        assert!(err.to_string().contains("load script"));
        assert!(err.to_string().contains("missing field"));
    }

    #[test]
    fn test_llm_error_conversion() {
        let err = LLMServiceError::Timeout;
        let app_err: AppError = err.into();
        assert_eq!(app_err.kind, AppErrorKind::Llm);
    }
}
