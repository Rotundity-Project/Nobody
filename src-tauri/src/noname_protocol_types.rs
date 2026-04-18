use crate::noname_errors::{NoNameError, NoNameErrorKind};
use crate::noname_trace::NoNameTrace;
use crate::noname_types::NoNameRole;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoNameProtocolVersion {
    V1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameProtocolHeader {
    pub protocol: String,
    pub version: NoNameProtocolVersion,
    pub trace_id: String,
    pub session_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameAgentAddress {
    pub agent_id: String,
    pub role: NoNameRole,
    pub runtime: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameTaskError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoNameTaskStatus {
    Pending,
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
}

impl NoNameTaskStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::TimedOut => "timed_out",
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Cancelled | Self::TimedOut
        )
    }

    pub fn can_transition_to(self, next: Self) -> bool {
        if self == next {
            return true;
        }

        match (self, next) {
            (Self::Pending, Self::Queued | Self::Running | Self::Cancelled) => true,
            (Self::Queued, Self::Running | Self::Cancelled | Self::TimedOut) => true,
            (Self::Running, Self::Completed | Self::Failed | Self::Cancelled | Self::TimedOut) => {
                true
            }
            _ if self.is_terminal() => false,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameTaskLifecycle {
    pub task_id: String,
    pub status: NoNameTaskStatus,
    pub parent_task_id: Option<String>,
    pub attempt: u32,
    pub timeout_ms: Option<u64>,
    pub last_error: Option<NoNameTaskError>,
    pub cancellation_reason: Option<String>,
}

pub trait NoNameTraceWritable {
    fn record_on_trace(&self, trace: &mut NoNameTrace, status: &str);
}

impl NoNameProtocolHeader {
    pub fn new(trace_id: impl Into<String>, session_id: impl Into<String>) -> Self {
        Self {
            protocol: "NNCP".to_string(),
            version: NoNameProtocolVersion::V1,
            trace_id: trace_id.into(),
            session_id: session_id.into(),
        }
    }
}

impl NoNameTaskLifecycle {
    pub fn new(task_id: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            status: NoNameTaskStatus::Pending,
            parent_task_id: None,
            attempt: 1,
            timeout_ms: None,
            last_error: None,
            cancellation_reason: None,
        }
    }

    pub fn with_parent(mut self, parent_task_id: impl Into<String>) -> Self {
        self.parent_task_id = Some(parent_task_id.into());
        self
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    pub fn transition_to(&self, next: NoNameTaskStatus) -> Result<Self, NoNameError> {
        if !self.status.can_transition_to(next) {
            return Err(NoNameError::new(
                NoNameErrorKind::Protocol,
                format!(
                    "task {} cannot transition from {} to {}",
                    self.task_id,
                    self.status.as_str(),
                    next.as_str()
                ),
                "noname.protocol.invalid_task_transition",
                true,
            ));
        }

        let mut next_state = self.clone();
        next_state.status = next;
        if matches!(next, NoNameTaskStatus::Queued | NoNameTaskStatus::Running) {
            next_state.last_error = None;
            next_state.cancellation_reason = None;
        }
        Ok(next_state)
    }

    pub fn queued(&self) -> Result<Self, NoNameError> {
        self.transition_to(NoNameTaskStatus::Queued)
    }

    pub fn running(&self) -> Result<Self, NoNameError> {
        self.transition_to(NoNameTaskStatus::Running)
    }

    pub fn completed(&self) -> Result<Self, NoNameError> {
        self.transition_to(NoNameTaskStatus::Completed)
    }

    pub fn failed(
        &self,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Result<Self, NoNameError> {
        let mut next_state = self.transition_to(NoNameTaskStatus::Failed)?;
        next_state.last_error = Some(NoNameTaskError {
            code: code.into(),
            message: message.into(),
        });
        Ok(next_state)
    }

    pub fn cancelled(&self, reason: impl Into<String>) -> Result<Self, NoNameError> {
        let mut next_state = self.transition_to(NoNameTaskStatus::Cancelled)?;
        next_state.cancellation_reason = Some(reason.into());
        Ok(next_state)
    }

    pub fn timed_out(&self) -> Result<Self, NoNameError> {
        self.transition_to(NoNameTaskStatus::TimedOut)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_header_defaults_to_nncp_v1() {
        let header = NoNameProtocolHeader::new("trace-1", "session-1");
        assert_eq!(header.protocol, "NNCP");
        assert_eq!(header.version, NoNameProtocolVersion::V1);
    }

    #[test]
    fn lifecycle_supports_valid_transitions() {
        let lifecycle = NoNameTaskLifecycle::new("task-1").with_timeout(1500);
        let queued = lifecycle.queued().expect("pending -> queued");
        let running = queued.running().expect("queued -> running");
        let completed = running.completed().expect("running -> completed");

        assert_eq!(completed.status, NoNameTaskStatus::Completed);
        assert_eq!(completed.timeout_ms, Some(1500));
        assert!(completed.status.is_terminal());
    }

    #[test]
    fn invalid_transition_returns_protocol_error() {
        let lifecycle = NoNameTaskLifecycle::new("task-2");
        let err = lifecycle
            .completed()
            .expect_err("pending -> completed should be rejected");

        assert_eq!(err.code, "noname.protocol.invalid_task_transition");
    }

    #[test]
    fn failed_and_cancelled_states_preserve_reason() {
        let running = NoNameTaskLifecycle::new("task-3")
            .queued()
            .expect("queue")
            .running()
            .expect("run");
        let failed = running
            .failed("noname.tool.failure", "tool execution failed")
            .expect("fail");
        let cancelled = NoNameTaskLifecycle::new("task-4")
            .queued()
            .expect("queue")
            .cancelled("operator request")
            .expect("cancel");

        assert_eq!(
            failed.last_error.as_ref().map(|err| err.code.as_str()),
            Some("noname.tool.failure")
        );
        assert_eq!(
            cancelled.cancellation_reason.as_deref(),
            Some("operator request")
        );
    }
}
