use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoNameCapabilityKind {
    Tool,
    Resource,
    Prompt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameCapabilityDescriptor {
    pub id: String,
    pub name: String,
    pub kind: NoNameCapabilityKind,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameToolCapability {
    pub descriptor: NoNameCapabilityDescriptor,
    pub canned_result: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameResourceCapability {
    pub descriptor: NoNameCapabilityDescriptor,
    pub resource_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNamePromptCapability {
    pub descriptor: NoNameCapabilityDescriptor,
    pub prompt_id: String,
}

impl NoNameCapabilityDescriptor {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        kind: NoNameCapabilityKind,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            kind,
            description: description.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn tool_capability_keeps_descriptor_and_result() {
        let capability = NoNameToolCapability {
            descriptor: NoNameCapabilityDescriptor::new(
                "tool.echo",
                "Echo Tool",
                NoNameCapabilityKind::Tool,
                "Return a canned payload",
            ),
            canned_result: json!({ "ok": true }),
        };

        assert_eq!(capability.descriptor.kind, NoNameCapabilityKind::Tool);
        assert_eq!(capability.canned_result["ok"], true);
    }
}
