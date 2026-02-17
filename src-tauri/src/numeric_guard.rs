use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RealmPowerBand {
    pub realm_requirement: u32,
    pub min_power: f64,
    pub max_power: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NumericRules {
    pub technique_power_bands: Vec<RealmPowerBand>,
    pub map_danger_min: u8,
    pub map_danger_max: u8,
    pub aura_density_min: f64,
    pub aura_density_max: f64,
}

#[derive(Debug, Clone)]
pub struct NumericCheckResult {
    pub accepted: bool,
    pub normalized: bool,
    pub normalized_value: Option<f64>,
    pub reason: Option<String>,
}

static RULES: OnceLock<NumericRules> = OnceLock::new();

pub fn rules() -> &'static NumericRules {
    RULES.get_or_init(|| {
        serde_json::from_str(include_str!("../config/numeric_rules_v2.json"))
            .expect("numeric_rules_v2.json must be valid")
    })
}

pub fn validate_technique_power(realm_requirement: u32, base_power: f64) -> NumericCheckResult {
    let rules = rules();
    if let Some(band) = rules
        .technique_power_bands
        .iter()
        .find(|b| b.realm_requirement == realm_requirement)
    {
        if base_power < band.min_power {
            return NumericCheckResult {
                accepted: true,
                normalized: true,
                normalized_value: Some(band.min_power),
                reason: Some(format!(
                    "base_power below minimum for realm {}, normalized to {}",
                    realm_requirement, band.min_power
                )),
            };
        }
        if base_power > band.max_power {
            return NumericCheckResult {
                accepted: true,
                normalized: true,
                normalized_value: Some(band.max_power),
                reason: Some(format!(
                    "base_power above maximum for realm {}, normalized to {}",
                    realm_requirement, band.max_power
                )),
            };
        }
        return NumericCheckResult {
            accepted: true,
            normalized: false,
            normalized_value: None,
            reason: None,
        };
    }

    NumericCheckResult {
        accepted: false,
        normalized: false,
        normalized_value: None,
        reason: Some(format!(
            "unknown realm requirement {} for technique power validation",
            realm_requirement
        )),
    }
}

pub fn validate_map_numbers(danger_tier: u8, aura_density: f64) -> NumericCheckResult {
    let rules = rules();
    if danger_tier < rules.map_danger_min || danger_tier > rules.map_danger_max {
        return NumericCheckResult {
            accepted: false,
            normalized: false,
            normalized_value: None,
            reason: Some(format!(
                "danger_tier {} out of range {}..{}",
                danger_tier, rules.map_danger_min, rules.map_danger_max
            )),
        };
    }

    if aura_density < rules.aura_density_min {
        return NumericCheckResult {
            accepted: true,
            normalized: true,
            normalized_value: Some(rules.aura_density_min),
            reason: Some("aura_density too low, normalized".to_string()),
        };
    }
    if aura_density > rules.aura_density_max {
        return NumericCheckResult {
            accepted: true,
            normalized: true,
            normalized_value: Some(rules.aura_density_max),
            reason: Some("aura_density too high, normalized".to_string()),
        };
    }

    NumericCheckResult {
        accepted: true,
        normalized: false,
        normalized_value: None,
        reason: None,
    }
}

pub fn validate_character_combat_power(realm_level: u32, combat_power: u64) -> NumericCheckResult {
    let rules = rules();
    let power = combat_power as f64;
    if let Some(band) = rules
        .technique_power_bands
        .iter()
        .find(|b| b.realm_requirement == realm_level)
    {
        let max_for_character = band.max_power * 2.0;
        if power < band.min_power {
            return NumericCheckResult {
                accepted: true,
                normalized: true,
                normalized_value: Some(band.min_power),
                reason: Some(format!(
                    "combat_power below minimum for realm {}, normalized to {}",
                    realm_level, band.min_power
                )),
            };
        }
        if power > max_for_character {
            return NumericCheckResult {
                accepted: true,
                normalized: true,
                normalized_value: Some(max_for_character),
                reason: Some(format!(
                    "combat_power above maximum for realm {}, normalized to {}",
                    realm_level, max_for_character
                )),
            };
        }
        return NumericCheckResult {
            accepted: true,
            normalized: false,
            normalized_value: None,
            reason: None,
        };
    }

    NumericCheckResult {
        accepted: false,
        normalized: false,
        normalized_value: None,
        reason: Some(format!(
            "unknown realm level {} for combat power validation",
            realm_level
        )),
    }
}
