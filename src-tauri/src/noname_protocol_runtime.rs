use crate::noname_capability_registry::NoNameCapabilityRegistry;
use crate::noname_errors::{NoNameError, NoNameErrorKind};
use crate::noname_protocol_agent::{NoNameAgentMessage, NoNameAgentMessageKind};
use crate::noname_protocol_tool::{NoNameToolCall, NoNameToolEnvelope};
use crate::noname_protocol_types::{NoNameTaskError, NoNameTaskLifecycle};
use std::collections::BTreeMap;

#[derive(Debug, Default, Clone)]
pub struct NoNameProtocolRuntime {
    task_states: BTreeMap<String, NoNameTaskLifecycle>,
    agent_history: Vec<NoNameAgentMessage>,
    tool_history: Vec<NoNameToolEnvelope>,
}

impl NoNameProtocolRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn task_state(&self, task_id: &str) -> Option<&NoNameTaskLifecycle> {
        self.task_states.get(task_id)
    }

    pub fn agent_history(&self) -> &[NoNameAgentMessage] {
        &self.agent_history
    }

    pub fn tool_history(&self) -> &[NoNameToolEnvelope] {
        &self.tool_history
    }

    pub fn clear(&mut self) {
        self.task_states.clear();
        self.agent_history.clear();
        self.tool_history.clear();
    }

    pub fn submit_agent_message(
        &mut self,
        message: NoNameAgentMessage,
    ) -> Result<NoNameTaskLifecycle, NoNameError> {
        let next = self.resolve_next_lifecycle(&message.lifecycle, message.kind)?;
        self.task_states.insert(next.task_id.clone(), next.clone());
        self.agent_history.push(message);
        Ok(next)
    }

    pub fn execute_local_tool(
        &mut self,
        registry: &NoNameCapabilityRegistry,
        lifecycle: NoNameTaskLifecycle,
        call: NoNameToolCall,
    ) -> Result<NoNameToolEnvelope, NoNameError> {
        let queued = self.merge_or_use(lifecycle).queued()?;
        self.task_states
            .insert(queued.task_id.clone(), queued.clone());
        let request = NoNameToolEnvelope::request(queued.clone(), call.clone());
        self.tool_history.push(request);

        let running = queued.running()?;
        self.task_states
            .insert(running.task_id.clone(), running.clone());

        match registry.invoke_tool(&call) {
            Ok(result) => {
                let completed = running.completed()?;
                self.task_states
                    .insert(completed.task_id.clone(), completed.clone());
                let envelope = NoNameToolEnvelope::result(completed, result);
                self.tool_history.push(envelope.clone());
                Ok(envelope)
            }
            Err(error) => {
                let failed = running.failed(error.code.clone(), error.message.clone())?;
                self.task_states
                    .insert(failed.task_id.clone(), failed.clone());
                let envelope = NoNameToolEnvelope::error(
                    call.header,
                    failed,
                    call.capability_id,
                    NoNameTaskError {
                        code: error.code,
                        message: error.message,
                    },
                );
                self.tool_history.push(envelope.clone());
                Ok(envelope)
            }
        }
    }

    pub fn cancel_task(
        &mut self,
        task_id: &str,
        reason: impl Into<String>,
    ) -> Result<NoNameTaskLifecycle, NoNameError> {
        let current = self.task_states.get(task_id).cloned().ok_or_else(|| {
            NoNameError::new(
                NoNameErrorKind::Protocol,
                format!("task not found: {task_id}"),
                "noname.protocol.task_not_found",
                true,
            )
        })?;
        let cancelled = current.cancelled(reason)?;
        self.task_states
            .insert(cancelled.task_id.clone(), cancelled.clone());
        Ok(cancelled)
    }

    pub fn timeout_task(&mut self, task_id: &str) -> Result<NoNameTaskLifecycle, NoNameError> {
        let current = self.task_states.get(task_id).cloned().ok_or_else(|| {
            NoNameError::new(
                NoNameErrorKind::Protocol,
                format!("task not found: {task_id}"),
                "noname.protocol.task_not_found",
                true,
            )
        })?;
        let timed_out = current.timed_out()?;
        self.task_states
            .insert(timed_out.task_id.clone(), timed_out.clone());
        Ok(timed_out)
    }

    fn merge_or_use(&self, incoming: NoNameTaskLifecycle) -> NoNameTaskLifecycle {
        self.task_states
            .get(&incoming.task_id)
            .cloned()
            .unwrap_or(incoming)
    }

    fn resolve_next_lifecycle(
        &self,
        incoming: &NoNameTaskLifecycle,
        kind: NoNameAgentMessageKind,
    ) -> Result<NoNameTaskLifecycle, NoNameError> {
        let base = self
            .task_states
            .get(&incoming.task_id)
            .cloned()
            .unwrap_or_else(|| incoming.clone());

        match kind {
            NoNameAgentMessageKind::TaskRequest => base.queued(),
            NoNameAgentMessageKind::Delegation => base.running(),
            NoNameAgentMessageKind::Status => Ok(incoming.clone()),
            NoNameAgentMessageKind::Result => base.completed(),
            NoNameAgentMessageKind::Cancel => base.cancelled("agent requested cancellation"),
            NoNameAgentMessageKind::Timeout => base.timed_out(),
            NoNameAgentMessageKind::Error => base.failed(
                "noname.protocol.agent_error",
                "agent message entered error state",
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noname_capability_base::{
        NoNameCapabilityDescriptor, NoNameCapabilityKind, NoNameToolCapability,
    };
    use crate::noname_capability_registry::NoNameCapabilityRegistry;
    use crate::noname_protocol_agent::NoNameAgentMessage;
    use crate::noname_protocol_types::{
        NoNameAgentAddress, NoNameProtocolHeader, NoNameTaskStatus,
    };
    use crate::noname_types::NoNameRole;
    use serde_json::json;

    fn address(agent_id: &str, role: NoNameRole) -> NoNameAgentAddress {
        NoNameAgentAddress {
            agent_id: agent_id.to_string(),
            role,
            runtime: "local".to_string(),
        }
    }

    fn build_registry() -> NoNameCapabilityRegistry {
        let mut registry = NoNameCapabilityRegistry::new();
        registry.register_tool(NoNameToolCapability {
            descriptor: NoNameCapabilityDescriptor::new(
                "tool.echo",
                "Echo Tool",
                NoNameCapabilityKind::Tool,
                "Return a canned response",
            ),
            canned_result: json!({"echo": "ok"}),
        });
        registry
    }

    #[test]
    fn agent_messages_drive_task_lifecycle_forward() {
        let header = NoNameProtocolHeader::new("trace-1", "session-1");
        let mut runtime = NoNameProtocolRuntime::new();
        let lifecycle = NoNameTaskLifecycle::new("task-1");

        let queued = runtime
            .submit_agent_message(NoNameAgentMessage::new(
                header.clone(),
                address("director", NoNameRole::Director),
                address("world-curator", NoNameRole::WorldCurator),
                NoNameAgentMessageKind::TaskRequest,
                lifecycle.clone(),
                json!({"goal": "补全世界事实"}),
            ))
            .expect("task request should queue");
        let running = runtime
            .submit_agent_message(NoNameAgentMessage::new(
                header.clone(),
                address("director", NoNameRole::Director),
                address("world-curator", NoNameRole::WorldCurator),
                NoNameAgentMessageKind::Delegation,
                queued.clone(),
                json!({"goal": "补全世界事实"}),
            ))
            .expect("delegation should run");
        let completed = runtime
            .submit_agent_message(NoNameAgentMessage::new(
                header,
                address("world-curator", NoNameRole::WorldCurator),
                address("director", NoNameRole::Director),
                NoNameAgentMessageKind::Result,
                running,
                json!({"result": "ok"}),
            ))
            .expect("result should complete");

        assert_eq!(completed.status, NoNameTaskStatus::Completed);
        assert_eq!(runtime.agent_history().len(), 3);
    }

    #[test]
    fn local_tool_execution_returns_result_envelope() {
        let mut runtime = NoNameProtocolRuntime::new();
        let registry = build_registry();

        let envelope = runtime
            .execute_local_tool(
                &registry,
                NoNameTaskLifecycle::new("tool-task-1"),
                NoNameToolCall {
                    header: NoNameProtocolHeader::new("trace-2", "session-2"),
                    capability_id: "tool.echo".to_string(),
                    args: json!({"text": "hello"}),
                },
            )
            .expect("tool execution should succeed");

        assert_eq!(
            envelope.kind,
            crate::noname_protocol_tool::NoNameToolEnvelopeKind::Result
        );
        assert_eq!(
            envelope.result.as_ref().map(|value| value["echo"].as_str()),
            Some(Some("ok"))
        );
        assert_eq!(
            runtime.task_state("tool-task-1").map(|state| state.status),
            Some(NoNameTaskStatus::Completed)
        );
        assert_eq!(runtime.tool_history().len(), 2);
    }

    #[test]
    fn running_task_can_be_cancelled_or_timed_out() {
        let mut runtime = NoNameProtocolRuntime::new();
        let lifecycle = NoNameTaskLifecycle::new("task-3")
            .queued()
            .expect("queue")
            .running()
            .expect("run");
        runtime
            .task_states
            .insert(lifecycle.task_id.clone(), lifecycle.clone());

        let cancelled = runtime
            .cancel_task("task-3", "operator stop")
            .expect("cancel should work");
        assert_eq!(cancelled.status, NoNameTaskStatus::Cancelled);

        runtime.task_states.insert(
            "task-4".to_string(),
            NoNameTaskLifecycle::new("task-4")
                .queued()
                .expect("queue")
                .running()
                .expect("run"),
        );
        let timed_out = runtime.timeout_task("task-4").expect("timeout should work");
        assert_eq!(timed_out.status, NoNameTaskStatus::TimedOut);
    }
}
