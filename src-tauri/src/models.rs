use serde::{Deserialize, Serialize};

/// Spiritual root element type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Element {
    Metal,
    Wood,
    Water,
    Fire,
    Earth,
    Thunder,
    Wind,
    Ice,
}

/// Spiritual root grade
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Grade {
    Heavenly,
    Earth,
    Human,
    Mortal,
}

/// Spiritual root
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpiritualRoot {
    pub element: Element,
    pub grade: Grade,
    pub affinity: f32,
}

/// Cultivation realm
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CultivationRealm {
    pub name: String,
    pub level: u32,
    pub sub_level: u32,
    pub power_multiplier: f32,
}

impl CultivationRealm {
    pub fn new(name: String, level: u32, sub_level: u32, power_multiplier: f32) -> Self {
        Self {
            name,
            level,
            sub_level,
            power_multiplier,
        }
    }

    pub fn sub_level_name(&self) -> &str {
        match self.sub_level {
            0 => "Early",
            1 => "Middle",
            2 => "Late",
            3 => "Peak",
            _ => "Unknown",
        }
    }
}

/// Lifespan
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lifespan {
    pub current_age: u32,
    pub max_age: u32,
    pub realm_bonus: u32,
}

impl Lifespan {
    pub fn new(current_age: u32, max_age: u32, realm_bonus: u32) -> Self {
        Self {
            current_age,
            max_age,
            realm_bonus,
        }
    }

    pub fn total_max_age(&self) -> u32 {
        self.max_age + self.realm_bonus
    }

    pub fn is_alive(&self) -> bool {
        self.current_age < self.total_max_age()
    }

    pub fn remaining_years(&self) -> u32 {
        self.total_max_age().saturating_sub(self.current_age)
    }
}

/// Character stats
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterStats {
    pub spiritual_root: SpiritualRoot,
    pub cultivation_realm: CultivationRealm,
    pub techniques: Vec<String>,
    pub lifespan: Lifespan,
    pub combat_power: u64,
}

impl CharacterStats {
    pub fn new(
        spiritual_root: SpiritualRoot,
        cultivation_realm: CultivationRealm,
        lifespan: Lifespan,
    ) -> Self {
        let combat_power = Self::calculate_base_combat_power(&spiritual_root, &cultivation_realm);
        Self {
            spiritual_root,
            cultivation_realm,
            techniques: Vec::new(),
            lifespan,
            combat_power,
        }
    }

    fn calculate_base_combat_power(
        spiritual_root: &SpiritualRoot,
        realm: &CultivationRealm,
    ) -> u64 {
        let base = 100u64;
        let grade_multiplier = match spiritual_root.grade {
            Grade::Heavenly => 2.0,
            Grade::Earth => 1.5,
            Grade::Human => 1.2,
            Grade::Mortal => 1.0,
        };
        let affinity_bonus = 1.0 + spiritual_root.affinity;
        let realm_power = realm.power_multiplier;

        (base as f32 * grade_multiplier * affinity_bonus * realm_power) as u64
    }

    pub fn update_combat_power(&mut self) {
        self.combat_power =
            Self::calculate_base_combat_power(&self.spiritual_root, &self.cultivation_realm);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifespan_is_alive() {
        let lifespan = Lifespan::new(50, 100, 50);
        assert!(lifespan.is_alive());
        assert_eq!(lifespan.total_max_age(), 150);
        assert_eq!(lifespan.remaining_years(), 100);
    }

    #[test]
    fn test_lifespan_dead() {
        let lifespan = Lifespan::new(150, 100, 50);
        assert!(!lifespan.is_alive());
        assert_eq!(lifespan.remaining_years(), 0);
    }

    #[test]
    fn test_cultivation_realm_sub_level_name() {
        let realm = CultivationRealm::new("Qi Condensation".to_string(), 1, 0, 1.0);
        assert_eq!(realm.sub_level_name(), "Early");

        let realm = CultivationRealm::new("Qi Condensation".to_string(), 1, 3, 1.0);
        assert_eq!(realm.sub_level_name(), "Peak");
    }

    #[test]
    fn test_character_stats_combat_power() {
        let spiritual_root = SpiritualRoot {
            element: Element::Fire,
            grade: Grade::Heavenly,
            affinity: 0.8,
        };
        let realm = CultivationRealm::new("Qi Condensation".to_string(), 1, 0, 1.0);
        let lifespan = Lifespan::new(20, 100, 0);

        let stats = CharacterStats::new(spiritual_root, realm, lifespan);
        assert!(stats.combat_power > 0);
    }
}

// Property-based tests for data models
#[cfg(test)]
mod property_tests {
    use super::*;
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

    // Task 4.2: Property 26 - Save/Load roundtrip consistency
    // Feature: Nobody, Property 26: Save/Load roundtrip consistency
    // For any game state, after saving and then loading, 
    // the restored state should be equivalent to the original state
    proptest! {
        #[test]
        fn test_property_26_spiritual_root_serialization_roundtrip(
            spiritual_root in arb_spiritual_root()
        ) {
            // Serialize to JSON
            let json = serde_json::to_string(&spiritual_root).unwrap();
            
            // Deserialize back
            let restored: SpiritualRoot = serde_json::from_str(&json).unwrap();
            
            // Should be equal to original
            prop_assert_eq!(spiritual_root, restored);
        }

        #[test]
        fn test_property_26_cultivation_realm_serialization_roundtrip(
            realm in arb_cultivation_realm()
        ) {
            // Serialize to JSON
            let json = serde_json::to_string(&realm).unwrap();
            
            // Deserialize back
            let restored: CultivationRealm = serde_json::from_str(&json).unwrap();
            
            // Should be equal to original
            prop_assert_eq!(realm, restored);
        }

        #[test]
        fn test_property_26_lifespan_serialization_roundtrip(
            lifespan in arb_lifespan()
        ) {
            // Serialize to JSON
            let json = serde_json::to_string(&lifespan).unwrap();
            
            // Deserialize back
            let restored: Lifespan = serde_json::from_str(&json).unwrap();
            
            // Should be equal to original
            prop_assert_eq!(lifespan, restored);
        }

        #[test]
        fn test_property_26_character_stats_serialization_roundtrip(
            stats in arb_character_stats()
        ) {
            // Serialize to JSON
            let json = serde_json::to_string(&stats).unwrap();
            
            // Deserialize back
            let restored: CharacterStats = serde_json::from_str(&json).unwrap();
            
            // Should be equal to original
            prop_assert_eq!(stats, restored);
        }
    }

    // Additional unit tests for serialization edge cases
    #[test]
    fn test_serialization_with_empty_techniques() {
        let stats = CharacterStats::new(
            SpiritualRoot {
                element: Element::Fire,
                grade: Grade::Heavenly,
                affinity: 0.8,
            },
            CultivationRealm::new("Qi Condensation".to_string(), 1, 0, 1.0),
            Lifespan::new(20, 100, 50),
        );

        let json = serde_json::to_string(&stats).unwrap();
        let restored: CharacterStats = serde_json::from_str(&json).unwrap();

        assert_eq!(stats, restored);
        assert_eq!(restored.techniques.len(), 0);
    }

    #[test]
    fn test_serialization_with_multiple_techniques() {
        let mut stats = CharacterStats::new(
            SpiritualRoot {
                element: Element::Water,
                grade: Grade::Earth,
                affinity: 0.6,
            },
            CultivationRealm::new("Foundation".to_string(), 2, 2, 2.5),
            Lifespan::new(50, 150, 100),
        );

        stats.techniques.push("Water Shield".to_string());
        stats.techniques.push("Ice Spear".to_string());
        stats.techniques.push("Healing Wave".to_string());

        let json = serde_json::to_string(&stats).unwrap();
        let restored: CharacterStats = serde_json::from_str(&json).unwrap();

        assert_eq!(stats, restored);
        assert_eq!(restored.techniques.len(), 3);
        assert_eq!(restored.techniques[0], "Water Shield");
        assert_eq!(restored.techniques[1], "Ice Spear");
        assert_eq!(restored.techniques[2], "Healing Wave");
    }

    #[test]
    fn test_serialization_preserves_combat_power() {
        let stats = CharacterStats::new(
            SpiritualRoot {
                element: Element::Thunder,
                grade: Grade::Heavenly,
                affinity: 0.95,
            },
            CultivationRealm::new("Core Formation".to_string(), 3, 3, 5.0),
            Lifespan::new(100, 200, 300),
        );

        let original_power = stats.combat_power;
        
        let json = serde_json::to_string(&stats).unwrap();
        let restored: CharacterStats = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.combat_power, original_power);
    }

    #[test]
    fn test_json_format_is_readable() {
        let spiritual_root = SpiritualRoot {
            element: Element::Fire,
            grade: Grade::Heavenly,
            affinity: 0.8,
        };

        let json = serde_json::to_string_pretty(&spiritual_root).unwrap();
        
        // JSON should contain the field names
        assert!(json.contains("element"));
        assert!(json.contains("grade"));
        assert!(json.contains("affinity"));
        assert!(json.contains("Fire"));
        assert!(json.contains("Heavenly"));
    }
}
