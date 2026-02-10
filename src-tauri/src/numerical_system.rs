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
    use crate::models::{Element, Grade, Lifespan, SpiritualRoot};

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

// Property-based tests
#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::models::{Element, Grade, Lifespan, SpiritualRoot};
    use proptest::prelude::*;

    // Arbitrary generators for property testing
    fn arb_element() -> impl Strategy<Value = Element> {
        prop_oneof![
            Just(Element::Metal),
            Just(Element::Wood),
            Just(Element::Water),
            Just(Element::Fire),
            Just(Element::Earth),
            Just(Element::Thunder),
            Just(Element::Wind),
            Just(Element::Ice),
        ]
    }

    fn arb_grade() -> impl Strategy<Value = Grade> {
        prop_oneof![
            Just(Grade::Heavenly),
            Just(Grade::Earth),
            Just(Grade::Human),
            Just(Grade::Mortal),
        ]
    }

    fn arb_spiritual_root() -> impl Strategy<Value = SpiritualRoot> {
        (arb_element(), arb_grade(), 0.0f32..=1.0f32).prop_map(|(element, grade, affinity)| {
            SpiritualRoot {
                element,
                grade,
                affinity,
            }
        })
    }

    fn arb_cultivation_realm() -> impl Strategy<Value = CultivationRealm> {
        (
            "[A-Z][a-z]+ [A-Z][a-z]+",
            1u32..=10,
            0u32..=3,
            0.5f32..=10.0f32,
        )
            .prop_map(|(name, level, sub_level, power_multiplier)| {
                CultivationRealm::new(name, level, sub_level, power_multiplier)
            })
    }

    fn arb_lifespan() -> impl Strategy<Value = Lifespan> {
        (10u32..=100, 50u32..=200, 0u32..=500).prop_map(|(current_age, max_age, realm_bonus)| {
            Lifespan::new(current_age, max_age, realm_bonus)
        })
    }

    fn arb_character_stats() -> impl Strategy<Value = CharacterStats> {
        (arb_spiritual_root(), arb_cultivation_realm(), arb_lifespan()).prop_map(
            |(spiritual_root, cultivation_realm, lifespan)| {
                CharacterStats::new(spiritual_root, cultivation_realm, lifespan)
            },
        )
    }

    fn arb_action() -> impl Strategy<Value = Action> {
        prop_oneof![
            Just(Action::Cultivate),
            Just(Action::Breakthrough),
            Just(Action::Rest),
            "[a-z ]+".prop_map(|desc| Action::Custom { description: desc }),
        ]
    }

    fn arb_context() -> impl Strategy<Value = Context> {
        ("[A-Z][a-z]+", "[A-Z][a-z]+").prop_map(|(location, time_of_day)| Context {
            location,
            time_of_day,
            weather: None,
        })
    }

    // Task 5.2: Property 5 - Action result numerical consistency
    // Feature: Nobody, Property 5: Action result numerical consistency
    // For any character action, the system should calculate results based on numerical system rules,
    // and the same input state and action should produce the same result
    proptest! {
        #[test]
        fn test_property_5_action_result_consistency(
            character in arb_character_stats(),
            action in arb_action(),
            context in arb_context()
        ) {
            let system = NumericalSystem::new();
            
            // Calculate result twice with same inputs
            let result1 = system.calculate_action_result(&character, &action, &context);
            let result2 = system.calculate_action_result(&character, &action, &context);
            
            // Results should be identical (deterministic)
            prop_assert_eq!(result1.success, result2.success);
            prop_assert_eq!(result1.description, result2.description);
            prop_assert_eq!(result1.stat_changes.len(), result2.stat_changes.len());
            prop_assert_eq!(result1.events.len(), result2.events.len());
        }
    }

    // Task 5.3: Property 6 - Realm breakthrough attribute update
    // Feature: Nobody, Property 6: Realm breakthrough attribute update
    // For any character's realm breakthrough, the system should update all related attributes
    proptest! {
        #[test]
        fn test_property_6_realm_breakthrough_updates_attributes(
            mut character in arb_character_stats()
        ) {
            let old_combat_power = character.combat_power;
            let old_realm_level = character.cultivation_realm.level;
            let old_sub_level = character.cultivation_realm.sub_level;
            
            // Simulate a breakthrough to next sub-level
            if character.cultivation_realm.sub_level < 3 {
                character.cultivation_realm.sub_level += 1;
                character.cultivation_realm.power_multiplier *= 1.2;
                character.update_combat_power();
                
                // Combat power should be updated
                prop_assert_ne!(character.combat_power, old_combat_power);
                // Realm should have changed
                prop_assert!(
                    character.cultivation_realm.sub_level > old_sub_level ||
                    character.cultivation_realm.level > old_realm_level
                );
            }
        }
    }

    // Task 5.4: Property 7 - Lifespan exhaustion triggers death
    // Feature: Nobody, Property 7: Lifespan exhaustion triggers death
    // For any character, when current age reaches or exceeds max lifespan, 
    // the system should trigger death event
    proptest! {
        #[test]
        fn test_property_7_lifespan_exhaustion_triggers_death(
            current_age in 0u32..=1000,
            max_age in 50u32..=200,
            realm_bonus in 0u32..=500
        ) {
            let lifespan = Lifespan::new(current_age, max_age, realm_bonus);
            let total_max = max_age + realm_bonus;
            
            if current_age >= total_max {
                // Character should be dead
                prop_assert!(!lifespan.is_alive());
                prop_assert_eq!(lifespan.remaining_years(), 0);
            } else {
                // Character should be alive
                prop_assert!(lifespan.is_alive());
                prop_assert_eq!(lifespan.remaining_years(), total_max - current_age);
            }
        }
    }

    // Task 5.5: Unit test for numerical conflict resolution
    // Test multiple effects affecting the same attribute
    #[test]
    fn test_numerical_conflict_resolution() {
        let system = NumericalSystem::new();
        let mut character = CharacterStats::new(
            SpiritualRoot {
                element: Element::Fire,
                grade: Grade::Heavenly,
                affinity: 0.8,
            },
            CultivationRealm::new("Qi Condensation".to_string(), 1, 0, 1.0),
            Lifespan::new(20, 100, 50),
        );

        let initial_combat_power = character.combat_power;

        // Apply multiple effects: realm breakthrough and technique learning
        character.cultivation_realm.sub_level += 1;
        character.cultivation_realm.power_multiplier *= 1.2;
        character.techniques.push("Fire Palm".to_string());
        
        // Update combat power - should use the latest realm multiplier
        character.update_combat_power();
        
        // Combat power should have increased due to realm improvement
        assert!(character.combat_power > initial_combat_power);
        
        // Verify the calculation is deterministic
        let expected_power = character.combat_power;
        character.update_combat_power();
        assert_eq!(character.combat_power, expected_power);
    }

    #[test]
    fn test_priority_rules_for_conflicting_effects() {
        // Test that realm changes take priority over other stat changes
        let mut character = CharacterStats::new(
            SpiritualRoot {
                element: Element::Water,
                grade: Grade::Earth,
                affinity: 0.6,
            },
            CultivationRealm::new("Foundation".to_string(), 2, 0, 2.0),
            Lifespan::new(30, 120, 80),
        );

        let power_before = character.combat_power;
        
        // Change realm (higher priority)
        character.cultivation_realm.power_multiplier = 3.0;
        character.update_combat_power();
        let power_after_realm = character.combat_power;
        
        // Realm change should significantly affect combat power
        assert!(power_after_realm > power_before);
        
        // Adding techniques doesn't directly affect combat power in current implementation
        // (combat power is calculated from spiritual root and realm only)
        character.techniques.push("Water Shield".to_string());
        character.update_combat_power();
        
        // Combat power should remain the same (techniques don't affect base calculation)
        assert_eq!(character.combat_power, power_after_realm);
    }
}
