use crate::noname_prompt_catalog::NoNamePromptTemplate;

pub const DIRECTOR_OBSERVE_PROMPT_ID: &str = "prompt.director.observe";

pub fn director_observe_prompt_template() -> NoNamePromptTemplate {
    NoNamePromptTemplate {
        prompt_id: DIRECTOR_OBSERVE_PROMPT_ID.to_string(),
        template: "你是Nobody的DirectorAgent。当前目标={{goal}}；玩家动作={{action}}；当前场景={{scene}}；请给出下一步最值得推进的冲突或关注点。".to_string(),
        required_variables: vec![
            "goal".to_string(),
            "action".to_string(),
            "scene".to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn director_prompt_contains_required_variables() {
        let template = director_observe_prompt_template();
        assert_eq!(template.prompt_id, DIRECTOR_OBSERVE_PROMPT_ID);
        assert_eq!(template.required_variables.len(), 3);
    }
}
