use crate::noname_prompt_catalog::NoNamePromptTemplate;

pub const DIRECTOR_OBSERVE_PROMPT_ID: &str = "prompt.director.observe";
pub const WORLD_CURATOR_OBSERVE_PROMPT_ID: &str = "prompt.world_curator.observe";
pub const NPC_INTENT_OBSERVE_PROMPT_ID: &str = "prompt.npc_intent.observe";
pub const COMBAT_NARRATOR_OBSERVE_PROMPT_ID: &str = "prompt.combat_narrator.observe";

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

pub fn world_curator_observe_prompt_template() -> NoNamePromptTemplate {
    NoNamePromptTemplate {
        prompt_id: WORLD_CURATOR_OBSERVE_PROMPT_ID.to_string(),
        template: "你是Nobody的WorldCuratorAgent。当前目标={{goal}}；玩家动作={{action}}；当前场景={{scene}}；请指出最需要补足或校准的世界事实、场景约束或设定缝隙。".to_string(),
        required_variables: vec![
            "goal".to_string(),
            "action".to_string(),
            "scene".to_string(),
        ],
    }
}

pub fn npc_intent_observe_prompt_template() -> NoNamePromptTemplate {
    NoNamePromptTemplate {
        prompt_id: NPC_INTENT_OBSERVE_PROMPT_ID.to_string(),
        template: "你是Nobody的NpcIntentAgent。当前目标={{goal}}；玩家动作={{action}}；当前场景={{scene}}；请判断最值得关注的NPC意图、关系变化或反应方向。".to_string(),
        required_variables: vec![
            "goal".to_string(),
            "action".to_string(),
            "scene".to_string(),
        ],
    }
}

pub fn combat_narrator_observe_prompt_template() -> NoNamePromptTemplate {
    NoNamePromptTemplate {
        prompt_id: COMBAT_NARRATOR_OBSERVE_PROMPT_ID.to_string(),
        template: "你是Nobody的CombatNarratorAgent。当前目标={{goal}}；玩家动作={{action}}；当前场景={{scene}}；请判断当前冲突节奏、战斗风险或动作表现上最值得强化的一点。".to_string(),
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

    #[test]
    fn multi_role_prompts_keep_three_shared_inputs() {
        let templates = [
            world_curator_observe_prompt_template(),
            npc_intent_observe_prompt_template(),
            combat_narrator_observe_prompt_template(),
        ];

        for template in templates {
            assert_eq!(template.required_variables.len(), 3);
            assert!(template.template.contains("当前目标"));
        }
    }
}
