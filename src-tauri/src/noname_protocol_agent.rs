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
    Cancel,
    Timeout,
    Error,
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

impl NoNameAgentMessage {
    pub fn new(
        header: NoNameProtocolHeader,
        from: NoNameAgentAddress,
        to: NoNameAgentAddress,
        kind: NoNameAgentMessageKind,
        lifecycle: NoNameTaskLifecycle,
        payload: Value,
    ) -> Self {
        Self {
            header,
            from,
            to,
            kind,
            lifecycle,
            payload,
        }
    }

    pub fn for_status(
        header: NoNameProtocolHeader,
        from: NoNameAgentAddress,
        to: NoNameAgentAddress,
        lifecycle: NoNameTaskLifecycle,
        payload: Value,
    ) -> Self {
        Self::new(
            header,
            from,
            to,
            NoNameAgentMessageKind::Status,
            lifecycle,
            payload,
        )
    }
}

impl NoNameTraceWritable for NoNameAgentMessage {
    fn record_on_trace(&self, trace: &mut NoNameTrace, status: &str) {
        trace.record_capability_call(
            format!("agent:{}->{}", self.from.agent_id, self.to.agent_id),
            "agentMessage",
            format!("{}:{}", self.kind_label(), status),
        );
        trace.record_protocol_event(
            "agent",
            Some(self.from.agent_id.clone()),
            Some(self.to.agent_id.clone()),
            self.kind_label(),
            self.lifecycle.task_id.clone(),
            status,
            self.lifecycle.parent_task_id.clone(),
        );
    }
}

impl NoNameAgentMessage {
    fn kind_label(&self) -> &'static str {
        match self.kind {
            NoNameAgentMessageKind::TaskRequest => "task_request",
            NoNameAgentMessageKind::Delegation => "delegation",
            NoNameAgentMessageKind::Status => "status",
            NoNameAgentMessageKind::Result => "result",
            NoNameAgentMessageKind::Cancel => "cancel",
            NoNameAgentMessageKind::Timeout => "timeout",
            NoNameAgentMessageKind::Error => "error",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noname_protocol_types::{NoNameTaskLifecycle, NoNameTaskStatus, NoNameTraceWritable};
    use crate::noname_types::{NoNameMode, NoNameRole};
    use serde_json::json;

    fn address(agent_id: &str, role: NoNameRole) -> NoNameAgentAddress {
        NoNameAgentAddress {
            agent_id: agent_id.to_string(),
            role,
            runtime: "local".to_string(),
        }
    }

    #[test]
    fn agent_message_can_record_to_trace() {
        let message = NoNameAgentMessage::new(
            NoNameProtocolHeader::new("trace-1", "session-1"),
            address("director", NoNameRole::Director),
            address("world-curator", NoNameRole::WorldCurator),
            NoNameAgentMessageKind::Delegation,
            NoNameTaskLifecycle {
                task_id: "task-1".to_string(),
                status: NoNameTaskStatus::Running,
                parent_task_id: None,
                attempt: 1,
                timeout_ms: Some(800),
                last_error: None,
                cancellation_reason: None,
            },
            json!({"goal": "补全世界事实"}),
        );
        let mut trace =
            NoNameTrace::empty("trace-1", "session-1", "turn-1", NoNameMode::ObserveOnly);

        message.record_on_trace(&mut trace, "queued");

        assert_eq!(trace.capability_calls.len(), 1);
        assert_eq!(trace.capability_calls[0].call_kind, "agentMessage");
        assert!(trace.capability_calls[0].status.contains("delegation"));
        assert_eq!(trace.protocol_events.len(), 1);
        assert_eq!(trace.protocol_events[0].channel, "agent");
        assert_eq!(trace.protocol_events[0].kind, "delegation");
        assert_eq!(trace.protocol_events[0].status, "queued");
    }

    #[test]
    fn status_constructor_uses_status_kind() {
        let message = NoNameAgentMessage::for_status(
            NoNameProtocolHeader::new("trace-2", "session-1"),
            address("director", NoNameRole::Director),
            address("npc-intent", NoNameRole::NpcIntent),
            NoNameTaskLifecycle::new("task-2"),
            json!({"status": "queued"}),
        );

        assert_eq!(message.kind, NoNameAgentMessageKind::Status);
    }
}
