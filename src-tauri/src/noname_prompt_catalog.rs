use crate::noname_errors::NoNameProtocolError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNamePromptTemplate {
    pub prompt_id: String,
    pub template: String,
    #[serde(default)]
    pub required_variables: Vec<String>,
}

impl NoNamePromptTemplate {
    pub fn resolve(
        &self,
        variables: &BTreeMap<String, String>,
    ) -> Result<String, NoNameProtocolError> {
        let mut resolved = self.template.clone();
        for variable in &self.required_variables {
            let value = variables.get(variable).ok_or_else(|| {
                NoNameProtocolError::new(
                    format!("missing prompt variable: {}", variable),
                    "noname.protocol.prompt_missing_variable",
                    true,
                )
            })?;
            resolved = resolved.replace(&format!("{{{{{}}}}}", variable), value);
        }
        Ok(resolved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_template_resolves_variables() {
        let template = NoNamePromptTemplate {
            prompt_id: "prompt.director.plan".to_string(),
            template: "请围绕{{goal}}规划下一步".to_string(),
            required_variables: vec!["goal".to_string()],
        };

        let mut vars = BTreeMap::new();
        vars.insert("goal".to_string(), "主线推进".to_string());

        let resolved = template.resolve(&vars).expect("prompt should resolve");
        assert!(resolved.contains("主线推进"));
    }
}
