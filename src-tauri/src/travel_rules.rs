use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TravelRules {
    pub mobility_base: f32,
    pub mobility_per_realm: f32,
    pub mobility_per_sub_level: f32,
    pub mobility_injury_penalty: f32,
    pub mobility_qi_penalty: f32,
    pub mobility_min: f32,
    pub mobility_max: f32,
    pub max_energy_base: f32,
    pub max_energy_per_realm: f32,
    pub max_energy_injury_penalty: f32,
    pub max_energy_min: f32,
    pub max_energy_max: f32,
    pub nearby_fallback_enabled: bool,
    pub nearby_fallback_count: usize,
    pub nearby_fallback_min_location_count: usize,
    pub encounter_base_prob: f64,
    pub encounter_energy_weight: f64,
    pub encounter_enmity_weight: f64,
    pub encounter_qi_weight: f64,
    pub encounter_prob_min: f64,
    pub encounter_prob_max: f64,
}

impl Default for TravelRules {
    fn default() -> Self {
        Self {
            mobility_base: 0.22,
            mobility_per_realm: 0.09,
            mobility_per_sub_level: 0.03,
            mobility_injury_penalty: 0.025,
            mobility_qi_penalty: 0.01,
            mobility_min: 0.12,
            mobility_max: 0.95,
            max_energy_base: 0.35,
            max_energy_per_realm: 0.16,
            max_energy_injury_penalty: 0.02,
            max_energy_min: 0.35,
            max_energy_max: 1.2,
            nearby_fallback_enabled: true,
            nearby_fallback_count: 2,
            nearby_fallback_min_location_count: 4,
            encounter_base_prob: 0.18,
            encounter_energy_weight: 0.42,
            encounter_enmity_weight: 0.025,
            encounter_qi_weight: 0.02,
            encounter_prob_min: 0.1,
            encounter_prob_max: 0.92,
        }
    }
}

fn load_from_disk() -> Option<TravelRules> {
    let path = std::path::Path::new("config/travel_rules_v2.json");
    let raw = std::fs::read_to_string(path).ok()?;
    serde_json::from_str::<TravelRules>(&raw).ok()
}

pub fn rules() -> &'static TravelRules {
    static RULES: OnceLock<TravelRules> = OnceLock::new();
    RULES.get_or_init(|| {
        load_from_disk()
            .or_else(|| serde_json::from_str(include_str!("../config/travel_rules_v2.json")).ok())
            .unwrap_or_default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_config_is_valid() {
        let cfg = serde_json::from_str::<TravelRules>(include_str!("../config/travel_rules_v2.json"));
        assert!(cfg.is_ok());
    }

    #[test]
    fn rules_are_reasonable() {
        let cfg = rules();
        assert!(cfg.mobility_min <= cfg.mobility_max);
        assert!(cfg.encounter_prob_min <= cfg.encounter_prob_max);
        assert!(cfg.nearby_fallback_count <= 8);
    }
}
