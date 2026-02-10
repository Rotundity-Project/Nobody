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
