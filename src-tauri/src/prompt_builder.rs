use serde::{Deserialize, Serialize};

pub const DEFAULT_MAX_HISTORY_ITEMS: usize = 12;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromptTemplate {
    ScriptGeneration,
    OptionGeneration,
    NpcDecision,
    PlotGeneration,
}

impl PromptTemplate {
    fn instruction(&self) -> &'static str {
        match self {
            PromptTemplate::ScriptGeneration => {
                "Generate a complete cultivation world script with coherent settings."
            }
            PromptTemplate::OptionGeneration => {
                "Generate 2 to 5 actionable player options for the current scene."
            }
            PromptTemplate::NpcDecision => {
                "Generate an NPC decision consistent with personality and memory."
            }
            PromptTemplate::PlotGeneration => {
                "Generate novel-style plot text that follows from the latest events."
            }
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptContext {
    pub scene: Option<String>,
    pub location: Option<String>,
    pub actor_name: Option<String>,
    pub actor_realm: Option<String>,
    pub actor_combat_power: Option<u64>,
    pub history_events: Vec<String>,
    pub world_setting_summary: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptConstraints {
    pub numerical_rules: Vec<String>,
    pub world_rules: Vec<String>,
    pub output_schema_hint: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PromptBuilder {
    max_history_items: usize,
}

impl PromptBuilder {
    pub fn new(max_history_items: usize) -> Self {
        Self {
            max_history_items: max_history_items.max(1),
        }
    }

    pub fn build_prompt(
        &self,
        template: PromptTemplate,
        context: &PromptContext,
        constraints: &PromptConstraints,
    ) -> String {
        self.build_prompt_with_token_limit(template, context, constraints, u32::MAX)
    }

    pub fn build_prompt_with_token_limit(
        &self,
        template: PromptTemplate,
        context: &PromptContext,
        constraints: &PromptConstraints,
        max_prompt_tokens: u32,
    ) -> String {
        let token_limit = max_prompt_tokens.max(1);
        let mut history_count = context.history_events.len().min(self.max_history_items);
        let mut text_limit = usize::MAX;

        loop {
            let prompt = self.render_prompt(template.clone(), context, constraints, history_count, text_limit);
            if self.estimate_prompt_tokens(&prompt) <= token_limit {
                return prompt;
            }

            if history_count > 0 {
                history_count -= 1;
                continue;
            }

            if text_limit > 256 {
                text_limit = 256;
                continue;
            }

            if text_limit > 128 {
                text_limit = 128;
                continue;
            }

            // Last resort: hard truncate to requested budget.
            let max_chars = usize::try_from(token_limit)
                .unwrap_or(usize::MAX)
                .saturating_mul(4);
            return prompt.chars().take(max_chars).collect();
        }
    }

    pub fn estimate_prompt_tokens(&self, prompt: &str) -> u32 {
        estimate_token_count(prompt)
    }

    fn render_prompt(
        &self,
        template: PromptTemplate,
        context: &PromptContext,
        constraints: &PromptConstraints,
        history_count: usize,
        text_limit: usize,
    ) -> String {
        let mut prompt = String::new();

        prompt.push_str("[Task]\n");
        prompt.push_str(template.instruction());
        prompt.push_str("\n\n");

        prompt.push_str("[Context]\n");
        if let Some(scene) = &context.scene {
            prompt.push_str(&format!("Scene: {}\n", truncate_text(scene, text_limit)));
        }
        if let Some(location) = &context.location {
            prompt.push_str(&format!("Location: {}\n", truncate_text(location, text_limit)));
        }
        if let Some(actor_name) = &context.actor_name {
            prompt.push_str(&format!("Actor: {}\n", truncate_text(actor_name, text_limit)));
        }
        if let Some(actor_realm) = &context.actor_realm {
            prompt.push_str(&format!("Realm: {}\n", truncate_text(actor_realm, text_limit)));
        }
        if let Some(power) = context.actor_combat_power {
            prompt.push_str(&format!("CombatPower: {power}\n"));
        }
        if let Some(summary) = &context.world_setting_summary {
            prompt.push_str(&format!(
                "WorldSetting: {}\n",
                truncate_text(summary, text_limit)
            ));
        }

        let start = context.history_events.len().saturating_sub(history_count);
        prompt.push_str("RecentHistory:\n");
        for event in &context.history_events[start..] {
            prompt.push_str("- ");
            prompt.push_str(&truncate_text(event, text_limit));
            prompt.push('\n');
        }
        if history_count == 0 {
            prompt.push_str("- none\n");
        }
        prompt.push('\n');

        prompt.push_str("[Constraints]\n");
        if constraints.numerical_rules.is_empty() {
            prompt.push_str("NumericalRules:\n- none\n");
        } else {
            prompt.push_str("NumericalRules:\n");
            for rule in &constraints.numerical_rules {
                prompt.push_str("- ");
                prompt.push_str(rule);
                prompt.push('\n');
            }
        }

        if constraints.world_rules.is_empty() {
            prompt.push_str("WorldRules:\n- none\n");
        } else {
            prompt.push_str("WorldRules:\n");
            for rule in &constraints.world_rules {
                prompt.push_str("- ");
                prompt.push_str(rule);
                prompt.push('\n');
            }
        }

        prompt.push_str("\n[OutputRequirements]\n");
        if let Some(schema_hint) = &constraints.output_schema_hint {
            prompt.push_str(schema_hint);
            prompt.push('\n');
        } else {
            prompt.push_str("Return valid JSON with deterministic fields when possible.\n");
        }
        prompt.push_str("Do not violate any numerical or world constraints.\n");

        prompt
    }
}

pub fn estimate_token_count(text: &str) -> u32 {
    let words = text.split_whitespace().count();
    if words > 0 {
        return u32::try_from(words).unwrap_or(u32::MAX);
    }

    let chars = text.chars().count();
    let approx = chars.saturating_add(3) / 4;
    u32::try_from(approx).unwrap_or(u32::MAX)
}

fn truncate_text(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }

    let mut out: String = text.chars().take(limit).collect();
    out.push_str("...");
    out
}

impl Default for PromptBuilder {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_HISTORY_ITEMS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn full_context() -> PromptContext {
        PromptContext {
            scene: Some("A tense breakthrough attempt in the sect hall".to_string()),
            location: Some("Azure Cloud Sect".to_string()),
            actor_name: Some("Lin Mo".to_string()),
            actor_realm: Some("Qi Condensation - Late".to_string()),
            actor_combat_power: Some(356),
            history_events: vec![
                "Defeated a rogue cultivator".to_string(),
                "Consumed a spirit pill".to_string(),
            ],
            world_setting_summary: Some("Five-element cultivation world with strict sect laws".to_string()),
        }
    }

    fn strict_constraints() -> PromptConstraints {
        PromptConstraints {
            numerical_rules: vec![
                "No realm jump larger than one major realm per event".to_string(),
                "Combat outcomes must respect combat power delta".to_string(),
            ],
            world_rules: vec![
                "The sect forbids lethal combat inside the mountain gate".to_string(),
            ],
            output_schema_hint: Some("Return JSON: {\"text\": string, \"events\": string[]}".to_string()),
        }
    }

    #[test]
    fn test_build_prompt_includes_template_instruction() {
        let builder = PromptBuilder::default();
        let prompt = builder.build_prompt(
            PromptTemplate::NpcDecision,
            &PromptContext::default(),
            &PromptConstraints::default(),
        );

        assert!(prompt.contains("Generate an NPC decision"));
        assert!(prompt.contains("[Task]"));
    }

    #[test]
    fn test_build_prompt_includes_context_and_constraints() {
        let builder = PromptBuilder::default();
        let prompt = builder.build_prompt(
            PromptTemplate::PlotGeneration,
            &full_context(),
            &strict_constraints(),
        );

        assert!(prompt.contains("Scene: A tense breakthrough attempt in the sect hall"));
        assert!(prompt.contains("Location: Azure Cloud Sect"));
        assert!(prompt.contains("Actor: Lin Mo"));
        assert!(prompt.contains("Realm: Qi Condensation - Late"));
        assert!(prompt.contains("CombatPower: 356"));
        assert!(prompt.contains("WorldSetting: Five-element cultivation world"));
        assert!(prompt.contains("No realm jump larger than one major realm per event"));
        assert!(prompt.contains("The sect forbids lethal combat inside the mountain gate"));
        assert!(prompt.contains("Return JSON"));
    }

    #[test]
    fn test_build_prompt_limits_history_size() {
        let builder = PromptBuilder::new(2);
        let context = PromptContext {
            history_events: vec![
                "event-1".to_string(),
                "event-2".to_string(),
                "event-3".to_string(),
            ],
            ..PromptContext::default()
        };

        let prompt = builder.build_prompt(
            PromptTemplate::OptionGeneration,
            &context,
            &PromptConstraints::default(),
        );

        assert!(!prompt.contains("- event-1"));
        assert!(prompt.contains("- event-2"));
        assert!(prompt.contains("- event-3"));
    }

    #[test]
    fn test_build_prompt_with_token_limit_truncates_history() {
        let builder = PromptBuilder::new(10);
        let context = PromptContext {
            history_events: vec![
                "long history event one".to_string(),
                "long history event two".to_string(),
                "long history event three".to_string(),
            ],
            ..PromptContext::default()
        };

        let prompt = builder.build_prompt_with_token_limit(
            PromptTemplate::PlotGeneration,
            &context,
            &PromptConstraints::default(),
            20,
        );

        assert!(builder.estimate_prompt_tokens(&prompt) <= 20);
    }

    #[test]
    fn test_estimate_token_count_non_empty() {
        let tokens = estimate_token_count("alpha beta gamma");
        assert_eq!(tokens, 3);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        // Feature: Nobody, Property 29: LLM prompt context completeness
        #[test]
        fn prop_prompt_contains_all_required_context_and_constraints(
            scene in "[A-Za-z0-9 ,._-]{1,40}",
            location in "[A-Za-z0-9 ,._-]{1,40}",
            actor in "[A-Za-z0-9 ,._-]{1,30}",
            realm in "[A-Za-z0-9 ,._-]{1,30}",
            history in prop::collection::vec("[A-Za-z0-9 ,._-]{1,30}", 1..6),
            numerical in prop::collection::vec("[A-Za-z0-9 ,._-]{1,30}", 1..4),
            world in prop::collection::vec("[A-Za-z0-9 ,._-]{1,30}", 1..4)
        ) {
            let builder = PromptBuilder::default();
            let context = PromptContext {
                scene: Some(scene.clone()),
                location: Some(location.clone()),
                actor_name: Some(actor.clone()),
                actor_realm: Some(realm.clone()),
                actor_combat_power: Some(123),
                history_events: history.clone(),
                world_setting_summary: Some("world-summary".to_string()),
            };

            let constraints = PromptConstraints {
                numerical_rules: numerical.clone(),
                world_rules: world.clone(),
                output_schema_hint: Some("json-output".to_string()),
            };

            let prompt = builder.build_prompt(
                PromptTemplate::PlotGeneration,
                &context,
                &constraints,
            );

            let expected_scene = format!("Scene: {}", scene);
            let expected_location = format!("Location: {}", location);
            let expected_actor = format!("Actor: {}", actor);
            let expected_realm = format!("Realm: {}", realm);

            prop_assert!(prompt.contains(&expected_scene));
            prop_assert!(prompt.contains(&expected_location));
            prop_assert!(prompt.contains(&expected_actor));
            prop_assert!(prompt.contains(&expected_realm));
            prop_assert!(prompt.contains("CombatPower: 123"));
            prop_assert!(prompt.contains("WorldSetting: world-summary"));

            for event in history {
                let expected_event = format!("- {}", event);
                prop_assert!(prompt.contains(&expected_event));
            }

            for rule in numerical {
                let expected_numerical_rule = format!("- {}", rule);
                prop_assert!(prompt.contains(&expected_numerical_rule));
            }

            for rule in world {
                let expected_world_rule = format!("- {}", rule);
                prop_assert!(prompt.contains(&expected_world_rule));
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        // Feature: Nobody, Property 33: LLM token limit compliance
        #[test]
        fn prop_prompt_builder_respects_token_limit(
            history in prop::collection::vec("[A-Za-z0-9 ]{5,30}", 0..20),
            token_limit in 10u32..=120
        ) {
            let builder = PromptBuilder::new(20);
            let context = PromptContext {
                scene: Some("Test Scene".to_string()),
                location: Some("Sect".to_string()),
                actor_name: Some("Player".to_string()),
                actor_realm: Some("Qi Condensation".to_string()),
                actor_combat_power: Some(100),
                history_events: history,
                world_setting_summary: Some("Cultivation world".to_string()),
            };

            let prompt = builder.build_prompt_with_token_limit(
                PromptTemplate::PlotGeneration,
                &context,
                &PromptConstraints::default(),
                token_limit,
            );

            prop_assert!(builder.estimate_prompt_tokens(&prompt) <= token_limit);
        }
    }
}
