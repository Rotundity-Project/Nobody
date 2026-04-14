use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameResourceDocument {
    pub resource_id: String,
    pub mime_type: String,
    pub content: Value,
}

impl NoNameResourceDocument {
    pub fn new(
        resource_id: impl Into<String>,
        mime_type: impl Into<String>,
        content: Value,
    ) -> Self {
        Self {
            resource_id: resource_id.into(),
            mime_type: mime_type.into(),
            content,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn resource_document_stores_content() {
        let doc = NoNameResourceDocument::new(
            "resource.world",
            "application/json",
            json!({"scene": "山门"}),
        );
        assert_eq!(doc.resource_id, "resource.world");
        assert_eq!(doc.content["scene"], "山门");
    }
}
