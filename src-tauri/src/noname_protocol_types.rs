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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoNameTaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameTaskLifecycle {
    pub task_id: String,
    pub status: NoNameTaskStatus,
    pub parent_task_id: Option<String>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_header_defaults_to_nncp_v1() {
        let header = NoNameProtocolHeader::new("trace-1", "session-1");
        assert_eq!(header.protocol, "NNCP");
        assert_eq!(header.version, NoNameProtocolVersion::V1);
    }
}
