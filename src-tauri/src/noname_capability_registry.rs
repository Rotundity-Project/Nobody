use crate::noname_capability_base::{
    NoNameCapabilityDescriptor, NoNameCapabilityKind, NoNamePromptCapability,
    NoNameResourceCapability, NoNameToolCapability,
};
use crate::noname_errors::{NoNameCapabilityError, NoNameError};
use crate::noname_prompt_catalog::NoNamePromptTemplate;
use crate::noname_protocol_tool::{
    NoNamePromptResolve, NoNamePromptResolveResult, NoNameResourceRead, NoNameResourceReadResult,
    NoNameToolCall, NoNameToolResult,
};
use crate::noname_resources::NoNameResourceDocument;
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct NoNameCapabilityRegistry {
    tools: BTreeMap<String, NoNameToolCapability>,
    resources: BTreeMap<String, (NoNameResourceCapability, NoNameResourceDocument)>,
    prompts: BTreeMap<String, (NoNamePromptCapability, NoNamePromptTemplate)>,
}

impl NoNameCapabilityRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_tool(&mut self, capability: NoNameToolCapability) {
        self.tools
            .insert(capability.descriptor.id.clone(), capability);
    }

    pub fn register_resource(
        &mut self,
        capability: NoNameResourceCapability,
        resource: NoNameResourceDocument,
    ) {
        self.resources
            .insert(capability.descriptor.id.clone(), (capability, resource));
    }

    pub fn register_prompt(
        &mut self,
        capability: NoNamePromptCapability,
        prompt: NoNamePromptTemplate,
    ) {
        self.prompts
            .insert(capability.descriptor.id.clone(), (capability, prompt));
    }

    pub fn list_descriptors(&self) -> Vec<NoNameCapabilityDescriptor> {
        let mut out = Vec::new();
        out.extend(self.tools.values().map(|item| item.descriptor.clone()));
        out.extend(
            self.resources
                .values()
                .map(|(capability, _)| capability.descriptor.clone()),
        );
        out.extend(
            self.prompts
                .values()
                .map(|(capability, _)| capability.descriptor.clone()),
        );
        out.sort_by(|a, b| a.id.cmp(&b.id));
        out
    }

    pub fn get_descriptor(&self, capability_id: &str) -> Option<NoNameCapabilityDescriptor> {
        self.tools
            .get(capability_id)
            .map(|item| item.descriptor.clone())
            .or_else(|| {
                self.resources
                    .get(capability_id)
                    .map(|(capability, _)| capability.descriptor.clone())
            })
            .or_else(|| {
                self.prompts
                    .get(capability_id)
                    .map(|(capability, _)| capability.descriptor.clone())
            })
    }

    pub fn invoke_tool(&self, call: &NoNameToolCall) -> Result<NoNameToolResult, NoNameError> {
        let tool = self.tools.get(&call.capability_id).ok_or_else(|| {
            NoNameCapabilityError::new(
                format!("tool capability not found: {}", call.capability_id),
                "noname.capability.tool_not_found",
                true,
            )
        })?;

        Ok(NoNameToolResult {
            header: call.header.clone(),
            capability_id: tool.descriptor.id.clone(),
            status: "ok".to_string(),
            content: tool.canned_result.clone(),
        })
    }

    pub fn read_resource(
        &self,
        read: &NoNameResourceRead,
    ) -> Result<NoNameResourceReadResult, NoNameError> {
        let (_, resource) = self.resources.get(&read.resource_id).ok_or_else(|| {
            NoNameCapabilityError::new(
                format!("resource capability not found: {}", read.resource_id),
                "noname.capability.resource_not_found",
                true,
            )
        })?;

        Ok(NoNameResourceReadResult {
            header: read.header.clone(),
            resource_id: resource.resource_id.clone(),
            content: resource.content.clone(),
        })
    }

    pub fn resolve_prompt(
        &self,
        resolve: &NoNamePromptResolve,
    ) -> Result<NoNamePromptResolveResult, NoNameError> {
        let (_, prompt) = self.prompts.get(&resolve.prompt_id).ok_or_else(|| {
            NoNameCapabilityError::new(
                format!("prompt capability not found: {}", resolve.prompt_id),
                "noname.capability.prompt_not_found",
                true,
            )
        })?;

        let resolved_prompt = prompt
            .resolve(&resolve.variables)
            .map_err(NoNameError::from)?;

        Ok(NoNamePromptResolveResult {
            header: resolve.header.clone(),
            prompt_id: prompt.prompt_id.clone(),
            resolved_prompt,
        })
    }

    pub fn list_by_kind(&self, kind: NoNameCapabilityKind) -> Vec<NoNameCapabilityDescriptor> {
        self.list_descriptors()
            .into_iter()
            .filter(|descriptor| descriptor.kind == kind)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noname_capability_base::NoNameCapabilityDescriptor;
    use crate::noname_protocol_tool::{NoNamePromptResolve, NoNameResourceRead};
    use crate::noname_protocol_types::{NoNameProtocolHeader, NoNameTraceWritable};
    use crate::noname_trace::NoNameTrace;
    use crate::noname_types::NoNameMode;
    use serde_json::json;

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
        registry.register_resource(
            NoNameResourceCapability {
                descriptor: NoNameCapabilityDescriptor::new(
                    "resource.world",
                    "World Resource",
                    NoNameCapabilityKind::Resource,
                    "Static world snapshot",
                ),
                resource_id: "resource.world".to_string(),
            },
            NoNameResourceDocument::new(
                "resource.world",
                "application/json",
                json!({"map": "qingyun"}),
            ),
        );
        registry.register_prompt(
            NoNamePromptCapability {
                descriptor: NoNameCapabilityDescriptor::new(
                    "prompt.director.plan",
                    "Director Prompt",
                    NoNameCapabilityKind::Prompt,
                    "Prompt for planning a turn",
                ),
                prompt_id: "prompt.director.plan".to_string(),
            },
            NoNamePromptTemplate {
                prompt_id: "prompt.director.plan".to_string(),
                template: "请围绕{{goal}}继续推进".to_string(),
                required_variables: vec!["goal".to_string()],
            },
        );
        registry
    }

    #[test]
    fn registry_lists_registered_descriptors() {
        let registry = build_registry();
        let descriptors = registry.list_descriptors();

        assert_eq!(descriptors.len(), 3);
        assert!(registry.get_descriptor("tool.echo").is_some());
    }

    #[test]
    fn registry_invokes_tool_reads_resource_and_resolves_prompt() {
        let registry = build_registry();
        let header = NoNameProtocolHeader::new("trace-1", "session-1");

        let tool_result = registry
            .invoke_tool(&NoNameToolCall {
                header: header.clone(),
                capability_id: "tool.echo".to_string(),
                args: json!({}),
            })
            .expect("tool should resolve");
        assert_eq!(tool_result.content["echo"], "ok");

        let resource_result = registry
            .read_resource(&NoNameResourceRead {
                header: header.clone(),
                resource_id: "resource.world".to_string(),
            })
            .expect("resource should resolve");
        assert_eq!(resource_result.content["map"], "qingyun");

        let mut variables = BTreeMap::new();
        variables.insert("goal".to_string(), "主线冲突".to_string());
        let prompt_result = registry
            .resolve_prompt(&NoNamePromptResolve {
                header,
                prompt_id: "prompt.director.plan".to_string(),
                variables,
            })
            .expect("prompt should resolve");
        assert!(prompt_result.resolved_prompt.contains("主线冲突"));
    }

    #[test]
    fn protocol_objects_can_write_to_trace() {
        let registry = build_registry();
        let header = NoNameProtocolHeader::new("trace-1", "session-1");
        let call = NoNameToolCall {
            header,
            capability_id: "tool.echo".to_string(),
            args: json!({}),
        };
        let mut trace =
            NoNameTrace::empty("trace-1", "session-1", "turn-1", NoNameMode::ObserveOnly);

        let _ = registry.invoke_tool(&call).expect("tool should resolve");
        call.record_on_trace(&mut trace, "ok");

        assert_eq!(trace.capability_calls.len(), 1);
        assert_eq!(trace.capability_calls[0].capability_id, "tool.echo");
    }
}
