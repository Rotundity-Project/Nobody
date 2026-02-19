use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleCounterRule {
    pub attacker_style: String,
    pub defender_style: String,
    pub delta_pct: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CombatStyleRules {
    pub default_same_style_penalty_pct: i32,
    pub default_unknown_delta_pct: i32,
    pub counters: Vec<StyleCounterRule>,
}

impl Default for CombatStyleRules {
    fn default() -> Self {
        Self {
            default_same_style_penalty_pct: -2,
            default_unknown_delta_pct: 0,
            counters: vec![
                StyleCounterRule {
                    attacker_style: "sword".to_string(),
                    defender_style: "body".to_string(),
                    delta_pct: 5,
                },
                StyleCounterRule {
                    attacker_style: "blade".to_string(),
                    defender_style: "talisman".to_string(),
                    delta_pct: 6,
                },
                StyleCounterRule {
                    attacker_style: "body".to_string(),
                    defender_style: "sword".to_string(),
                    delta_pct: 4,
                },
                StyleCounterRule {
                    attacker_style: "talisman".to_string(),
                    defender_style: "blade".to_string(),
                    delta_pct: 5,
                },
            ],
        }
    }
}

fn load_from_disk() -> Option<CombatStyleRules> {
    let path = std::path::Path::new("config/combat_style_rules_v2.json");
    let raw = std::fs::read_to_string(path).ok()?;
    serde_json::from_str::<CombatStyleRules>(&raw).ok()
}

pub fn rules() -> &'static CombatStyleRules {
    static RULES: OnceLock<CombatStyleRules> = OnceLock::new();
    RULES.get_or_init(|| {
        load_from_disk()
            .or_else(|| serde_json::from_str(include_str!("../config/combat_style_rules_v2.json")).ok())
            .unwrap_or_default()
    })
}

pub fn counter_delta(attacker_style: &str, defender_style: &str) -> i32 {
    let cfg = rules();
    if attacker_style == defender_style {
        return cfg.default_same_style_penalty_pct;
    }
    cfg.counters
        .iter()
        .find(|rule| rule.attacker_style == attacker_style && rule.defender_style == defender_style)
        .map(|rule| rule.delta_pct)
        .unwrap_or(cfg.default_unknown_delta_pct)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_config_is_valid() {
        let cfg =
            serde_json::from_str::<CombatStyleRules>(include_str!("../config/combat_style_rules_v2.json"));
        assert!(cfg.is_ok());
    }

    #[test]
    fn rules_are_reasonable() {
        let cfg = rules();
        assert!(!cfg.counters.is_empty());
        assert!(cfg.default_same_style_penalty_pct <= 0);
    }

    #[test]
    fn configured_counter_applies() {
        let delta = counter_delta("sword", "body");
        assert!(delta > 0);
    }
}
