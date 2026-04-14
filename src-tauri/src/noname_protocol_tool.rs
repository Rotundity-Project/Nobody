use crate::noname_protocol_types::{NoNameProtocolHeader, NoNameTraceWritable};
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

#[cfg(test)]
mod tests {
    use super::*;
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
}
