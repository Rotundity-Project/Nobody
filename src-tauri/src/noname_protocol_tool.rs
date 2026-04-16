use crate::noname_protocol_types::{
    NoNameProtocolHeader, NoNameTaskError, NoNameTaskLifecycle, NoNameTraceWritable,
};
use crate::noname_trace::NoNameTrace;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameToolCall {
    pub header: NoNameProtocolHeader,
    pub capability_id: String,
    pub args: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameToolResult {
    pub header: NoNameProtocolHeader,
    pub capability_id: String,
    pub status: String,
    pub content: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameResourceRead {
    pub header: NoNameProtocolHeader,
    pub resource_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameResourceReadResult {
    pub header: NoNameProtocolHeader,
    pub resource_id: String,
    pub content: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNamePromptResolve {
    pub header: NoNameProtocolHeader,
    pub prompt_id: String,
    #[serde(default)]
    pub variables: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNamePromptResolveResult {
    pub header: NoNameProtocolHeader,
    pub prompt_id: String,
    pub resolved_prompt: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoNameToolEnvelopeKind {
    Request,
    Result,
    Error,
    Timeout,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameToolEnvelope {
    pub header: NoNameProtocolHeader,
    pub lifecycle: NoNameTaskLifecycle,
    pub kind: NoNameToolEnvelopeKind,
    pub capability_id: String,
    pub args: Option<Value>,
    pub result: Option<Value>,
    pub error: Option<NoNameTaskError>,
}

impl NoNameToolEnvelope {
    pub fn request(lifecycle: NoNameTaskLifecycle, call: NoNameToolCall) -> Self {
        Self {
            header: call.header.clone(),
            lifecycle,
            kind: NoNameToolEnvelopeKind::Request,
            capability_id: call.capability_id,
            args: Some(call.args),
            result: None,
            error: None,
        }
    }

    pub fn result(lifecycle: NoNameTaskLifecycle, tool_result: NoNameToolResult) -> Self {
        Self {
            header: tool_result.header.clone(),
            lifecycle,
            kind: NoNameToolEnvelopeKind::Result,
            capability_id: tool_result.capability_id,
            args: None,
            result: Some(tool_result.content),
            error: None,
        }
    }

    pub fn error(
        header: NoNameProtocolHeader,
        lifecycle: NoNameTaskLifecycle,
        capability_id: impl Into<String>,
        error: NoNameTaskError,
    ) -> Self {
        Self {
            header,
            lifecycle,
            kind: NoNameToolEnvelopeKind::Error,
            capability_id: capability_id.into(),
            args: None,
            result: None,
            error: Some(error),
        }
    }
}

impl NoNameTraceWritable for NoNameToolCall {
    fn record_on_trace(&self, trace: &mut NoNameTrace, status: &str) {
        trace.record_capability_call(self.capability_id.clone(), "tool", status.to_string());
    }
}

impl NoNameTraceWritable for NoNameResourceRead {
    fn record_on_trace(&self, trace: &mut NoNameTrace, status: &str) {
        trace.record_capability_call(self.resource_id.clone(), "resource", status.to_string());
    }
}

impl NoNameTraceWritable for NoNamePromptResolve {
    fn record_on_trace(&self, trace: &mut NoNameTrace, status: &str) {
        trace.record_capability_call(self.prompt_id.clone(), "prompt", status.to_string());
    }
}

impl NoNameTraceWritable for NoNameToolEnvelope {
    fn record_on_trace(&self, trace: &mut NoNameTrace, status: &str) {
        let kind = match self.kind {
            NoNameToolEnvelopeKind::Request => "toolRequest",
            NoNameToolEnvelopeKind::Result => "toolResult",
            NoNameToolEnvelopeKind::Error => "toolError",
            NoNameToolEnvelopeKind::Timeout => "toolTimeout",
        };
        trace.record_capability_call(self.capability_id.clone(), kind, status.to_string());
        trace.record_protocol_event(
            "tool",
            None,
            Some(self.capability_id.clone()),
            kind,
            self.lifecycle.task_id.clone(),
            status,
            self.lifecycle
                .last_error
                .as_ref()
                .map(|error| format!("{}:{}", error.code, error.message)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noname_protocol_types::NoNameTaskLifecycle;
    use crate::noname_types::NoNameMode;
    use serde_json::json;

    #[test]
    fn tool_call_can_record_to_trace() {
        let call = NoNameToolCall {
            header: NoNameProtocolHeader::new("trace-1", "session-1"),
            capability_id: "tool.echo".to_string(),
            args: json!({"text": "hello"}),
        };
        let mut trace =
            NoNameTrace::empty("trace-1", "session-1", "turn-1", NoNameMode::ObserveOnly);

        call.record_on_trace(&mut trace, "ok");

        assert_eq!(trace.capability_calls.len(), 1);
        assert_eq!(trace.capability_calls[0].call_kind, "tool");
    }

    #[test]
    fn tool_request_and_result_envelopes_keep_capability_id() {
        let lifecycle = NoNameTaskLifecycle::new("task-1");
        let request = NoNameToolEnvelope::request(
            lifecycle.clone(),
            NoNameToolCall {
                header: NoNameProtocolHeader::new("trace-2", "session-2"),
                capability_id: "tool.echo".to_string(),
                args: json!({"text": "hello"}),
            },
        );
        let result = NoNameToolEnvelope::result(
            lifecycle,
            NoNameToolResult {
                header: NoNameProtocolHeader::new("trace-2", "session-2"),
                capability_id: "tool.echo".to_string(),
                status: "ok".to_string(),
                content: json!({"echo": "hello"}),
            },
        );

        assert_eq!(request.kind, NoNameToolEnvelopeKind::Request);
        assert_eq!(result.kind, NoNameToolEnvelopeKind::Result);
        assert_eq!(result.capability_id, "tool.echo");
    }

    #[test]
    fn tool_envelope_can_record_protocol_event() {
        let envelope = NoNameToolEnvelope::request(
            NoNameTaskLifecycle::new("tool-task-1"),
            NoNameToolCall {
                header: NoNameProtocolHeader::new("trace-3", "session-3"),
                capability_id: "tool.echo".to_string(),
                args: json!({"text": "hello"}),
            },
        );
        let mut trace =
            NoNameTrace::empty("trace-3", "session-3", "turn-3", NoNameMode::ObserveOnly);

        envelope.record_on_trace(&mut trace, "queued");

        assert_eq!(trace.protocol_events.len(), 1);
        assert_eq!(trace.protocol_events[0].channel, "tool");
        assert_eq!(trace.protocol_events[0].kind, "toolRequest");
    }
}
