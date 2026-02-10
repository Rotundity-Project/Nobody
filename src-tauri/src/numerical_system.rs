use crate::models::{CharacterStats, CultivationRealm};
use serde::{Deserialize, Serialize};

/// Action types that characters can perform
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Action {
    Cultivate,
    Combat { target_id: String },
    Breakthrough,
    Rest,
    Custom { description: String },
}

/// Context for action execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Context {
    pub location: String,
    pub time_of_day: String,
    pub weather: Option<String>,
}

/// Result of an action
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionResult {
    pub success: bool,
    pub description: String,
    pub stat_changes: Vec<StatChange>,
    pub events: Vec<String>,
}

/// Stat change record
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatChange {
    pub stat_name: String,
    pub old_value: String,
    pub new_value: String,
}

/// Combat result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatResult {
    pub winner_id: String,
    pub loser_id: String,
    pub damage_dealt: u32,
    pub description: String,
}

/// Numerical system for game mechanics
pub struct NumericalSystem {
    realm_rules: RealmRules,
}

struct RealmRules {
    breakthrough_difficulty: f32,
}

impl Default for NumericalSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl NumericalSystem {
    pub fn new() -> Self {
        Self {
            realm_rules: RealmRules {
                breakthrough_difficulty: 0.5,
            },
        }
    }

    /// Calculate action result based on character stats and action
    pub fn calculate_action_result(
        &self,
        actor: &CharacterStats,
        action: &Action,
        context: &Context,
    ) -> ActionResult {
        match action {
            Action::Cultivate => self.calculate_cultivation_result(actor, context),
            Action::Combat { target_id } => {
                ActionResult {
                    success: true,
                    description: format!("Combat initiated with {}", target_id),
                    stat_changes: vec![],
                    events: vec![format!("Combat started at {}", context.location)],
                }
            }
            Action::Breakthrough => self.calculate_breakthrough_result(actor),
            Action::Rest => ActionResult {
                success: true,
                description: "Character rested and recovered energy".to_string(),
                stat_changes: vec![],
                events: vec![],
            },
            Action::Custom { description } => ActionResult {
                success: true,
                description: description.clone(),
                stat_changes: vec![],
                events: vec![],
            },
        }
    }

    fn calculate_cultivation_result(
        &self,
        actor: &CharacterStats,
        _context: &Context,
    ) -> ActionResult {
        let progress = actor.spiritual_root.affinity * 10.0;
        ActionResult {
            success: true,
            description: format!(
                "Cultivated successfully. Progress: {:.1}%",
                progress
            ),
            stat_changes: vec![],
            events: vec!["Cultivation session completed".to_string()],
        }
    }

    fn calculate_breakthrough_result(&self, actor: &CharacterStats) -> ActionResult {
        let success_chance = actor.spiritual_root.affinity * (1.0 - self.realm_rules.breakthrough_difficulty);
        let success = success_chance > 0.3;

        ActionResult {
            success,
            description: if success {
                format!(
                    "Successfully broke through to the next sub-level of {}!",
                    actor.cultivation_realm.name
                )
            } else {
                "Breakthrough attempt failed. Need more preparation.".to_string()
            },
            stat_changes: vec![],
            events: if success {
                vec!["Breakthrough successful".to_string()]
            } else {
                vec!["Breakthrough failed".to_string()]
            },
        }
    }

    /// Validate if a realm breakthrough is possible
    pub fn validate_realm_breakthrough(
        &self,
        character: &CharacterStats,
        target_realm: &CultivationRealm,
    ) -> bool {
        // Can only breakthrough to next level or next sub-level
        let current = &character.cultivation_realm;
        
        if target_realm.level == current.level {
            // Same level, check sub-level
            target_realm.sub_level == current.sub_level + 1 && target_realm.sub_level <= 3
        } else if target_realm.level == current.level + 1 {
            // Next level, must be at peak of current level
            current.sub_level == 3 && target_realm.sub_level == 0
        } else {
            false
        }
    }

    /// Calculate combat outcome between two characters
    pub fn calculate_combat_outcome(
        &self,
        attacker: &CharacterStats,
        defender: &CharacterStats,
    ) -> CombatResult {
        let power_diff = attacker.combat_power as i64 - defender.combat_power as i64;
        
        let (winner_id, loser_id, damage) = if power_diff > 0 {
            ("attacker".to_string(), "defender".to_string(), (power_diff / 10) as u32)
        } else {
            ("defender".to_string(), "attacker".to_string(), (power_diff.abs() / 10) as u32)
        };

        CombatResult {
            winner_id: winner_id.clone(),
            loser_id: loser_id.clone(),
            damage_dealt: damage.max(1),
            description: format!(
                "{} defeated {} with {} damage",
                winner_id, loser_id, damage
            ),
        }
    }

    /// Update character lifespan
    pub fn update_lifespan(&self, character: &mut CharacterStats, time_passed: u32) {
        character.lifespan.current_age += time_passed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Element, Grade, SpiritualRoot};

    fn create_test_character() -> CharacterStats {
        let spiritual_root = SpiritualRoot {
            element: Element::Fire,
            grade: Grade::Heavenly,
            affinity: 0.8,
        };
        let realm = CultivationRealm::new("Qi Condensation".to_string(), 1, 0, 1.0);
        let lifespan = Lifespan::new(20, 100, 50);
        CharacterStats::new(spiritual_root, realm, lifespan)
    }

    #[test]
    fn test_calculate_action_result_cultivate() {
        let system = NumericalSystem::new();
        let character = create_test_character();
        let context = Context {
            location: "Cave".to_string(),
            time_of_day: "Night".to_string(),
            weather: None,
        };

        let result = system.calculate_action_result(&character, &Action::Cultivate, &context);
        assert!(result.success);
        assert!(result.description.contains("Cultivated successfully"));
    }

    #[test]
    fn test_validate_realm_breakthrough_same_level() {
        let system = NumericalSystem::new();
        let mut character = create_test_character();
        character.cultivation_realm.sub_level = 0;

        let target = CultivationRealm::new("Qi Condensation".to_string(), 1, 1, 1.2);
        assert!(system.validate_realm_breakthrough(&character, &target));

        let invalid_target = CultivationRealm::new("Qi Condensation".to_string(), 1, 2, 1.4);
        assert!(!system.validate_realm_breakthrough(&character, &invalid_target));
    }

    #[test]
    fn test_validate_realm_breakthrough_next_level() {
        let system = NumericalSystem::new();
        let mut character = create_test_character();
        character.cultivation_realm.sub_level = 3; // Peak

        let target = CultivationRealm::new("Foundation Establishment".to_string(), 2, 0, 2.0);
        assert!(system.validate_realm_breakthrough(&character, &target));
    }

    #[test]
    fn test_calculate_combat_outcome() {
        let system = NumericalSystem::new();
        let attacker = create_test_character();
        let mut defender = create_test_character();
        defender.combat_power = 100; // Weaker

        let result = system.calculate_combat_outcome(&attacker, &defender);
        assert_eq!(result.winner_id, "attacker");
        assert!(result.damage_dealt > 0);
    }

    #[test]
    fn test_update_lifespan() {
        let system = NumericalSystem::new();
        let mut character = create_test_character();
        let initial_age = character.lifespan.current_age;

        system.update_lifespan(&mut character, 10);
        assert_eq!(character.lifespan.current_age, initial_age + 10);
    }
}
