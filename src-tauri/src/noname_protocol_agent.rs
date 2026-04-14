use crate::noname_protocol_types::{
    NoNameAgentAddress, NoNameProtocolHeader, NoNameTaskLifecycle, NoNameTraceWritable,
};
use crate::noname_trace::NoNameTrace;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoNameAgentMessageKind {
    TaskRequest,
    Delegation,
    Status,
    Result,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameAgentMessage {
    pub header: NoNameProtocolHeader,
    pub from: NoNameAgentAddress,
    pub to: NoNameAgentAddress,
    pub kind: NoNameAgentMessageKind,
    pub lifecycle: NoNameTaskLifecycle,
    pub payload: Value,
}

impl NoNameTraceWritable for NoNameAgentMessage {
    fn record_on_trace(&self, trace: &mut NoNameTrace, status: &str) {
        trace.record_capability_call(
            format!("agent:{}", self.to.agent_id),
            "agentMessage",
            status.to_string(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noname_protocol_types::{NoNameTaskStatus, NoNameTraceWritable};
    use crate::noname_types::{NoNameMode, NoNameRole};
    use serde_json::json;

    #[test]
    fn agent_message_can_record_to_trace() {
        let message = NoNameAgentMessage {
            header: NoNameProtocolHeader::new("trace-1", "session-1"),
            from: NoNameAgentAddress {
                agent_id: "director".to_string(),
                role: NoNameRole::Director,
                runtime: "local".to_string(),
            },
            to: NoNameAgentAddress {
                agent_id: "world-curator".to_string(),
                role: NoNameRole::WorldCurator,
                runtime: "local".to_string(),
            },
            kind: NoNameAgentMessageKind::Delegation,
            lifecycle: NoNameTaskLifecycle {
                task_id: "task-1".to_string(),
                status: NoNameTaskStatus::Running,
                parent_task_id: None,
            },
            payload: json!({"goal": "补全世界事实"}),
        };
        let mut trace =
            NoNameTrace::empty("trace-1", "session-1", "turn-1", NoNameMode::ObserveOnly);

        message.record_on_trace(&mut trace, "queued");

        assert_eq!(trace.capability_calls.len(), 1);
        assert_eq!(trace.capability_calls[0].call_kind, "agentMessage");
    }
}
