use crate::models::CharacterStats;
use crate::script::{Location, Script};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 包含所有游戏数据的主游戏状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameState {
    pub script: Script,
    pub player: Character,
    pub world_state: WorldState,
    pub game_time: GameTime,
}

/// 角色数据结构
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Character {
    pub id: String,
    pub name: String,
    pub stats: CharacterStats,
    pub inventory: Vec<Item>,
    pub location: String,
}

/// 角色背包中的物品
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub description: String,
    pub item_type: ItemType,
}

/// 物品类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ItemType {
    Technique,
    Artifact,
    Medicine,
    Material,
}

/// 包含地点和全局事件的世界状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorldState {
    pub locations: HashMap<String, Location>,
    pub global_events: Vec<GlobalEvent>,
}

/// 影响世界的全局事件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlobalEvent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub timestamp: u64,
}

/// 游戏时间追踪
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameTime {
    pub year: u32,
    pub month: u32,
    pub day: u32,
    pub total_days: u32,
}

impl GameTime {
    pub fn new(year: u32, month: u32, day: u32) -> Self {
        let total_days = (year - 1) * 360 + (month - 1) * 30 + day;
        Self {
            year,
            month,
            day,
            total_days,
        }
    }

    pub fn advance_days(&mut self, days: u32) {
        self.total_days += days;
        self.day += days;

        while self.day > 30 {
            self.day -= 30;
            self.month += 1;
        }

        while self.month > 12 {
            self.month -= 12;
            self.year += 1;
        }
    }
}

impl Character {
    pub fn new(
        id: String,
        name: String,
        stats: CharacterStats,
        location: String,
    ) -> Self {
        Self {
            id,
            name,
            stats,
            inventory: Vec::new(),
            location,
        }
    }
}

impl WorldState {
    pub fn new() -> Self {
        Self {
            locations: HashMap::new(),
            global_events: Vec::new(),
        }
    }

    pub fn from_script(script: &Script) -> Self {
        let mut locations = HashMap::new();
        for location in &script.world_setting.locations {
            locations.insert(location.id.clone(), location.clone());
        }

        Self {
            locations,
            global_events: Vec::new(),
        }
    }
}

impl Default for WorldState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CultivationRealm, Element, Grade, Lifespan, SpiritualRoot};
    use crate::script::{InitialState, ScriptType, WorldSetting};

    fn create_test_character() -> Character {
        let stats = CharacterStats {
            spiritual_root: SpiritualRoot {
                element: Element::Fire,
                grade: Grade::Heavenly,
                affinity: 0.8,
            },
            cultivation_realm: CultivationRealm::new("Qi Condensation".to_string(), 1, 0, 1.0),
            techniques: Vec::new(),
            lifespan: Lifespan {
                current_age: 16,
                max_age: 100,
                realm_bonus: 0,
            },
            combat_power: 100,
        };

        Character::new(
            "player".to_string(),
            "Test Player".to_string(),
            stats,
            "sect".to_string(),
        )
    }

    #[test]
    fn test_game_time_creation() {
        let time = GameTime::new(1, 1, 1);
        assert_eq!(time.year, 1);
        assert_eq!(time.month, 1);
        assert_eq!(time.day, 1);
        assert_eq!(time.total_days, 1);
    }

    #[test]
    fn test_game_time_advance() {
        let mut time = GameTime::new(1, 1, 1);
        time.advance_days(10);
        assert_eq!(time.day, 11);
        assert_eq!(time.total_days, 11);
    }

    #[test]
    fn test_game_time_advance_month() {
        let mut time = GameTime::new(1, 1, 1);
        time.advance_days(30);
        assert_eq!(time.month, 2);
        assert_eq!(time.day, 1);
    }

    #[test]
    fn test_game_time_advance_year() {
        let mut time = GameTime::new(1, 1, 1);
        time.advance_days(360);
        assert_eq!(time.year, 2);
        assert_eq!(time.month, 1);
        assert_eq!(time.day, 1);
    }

    #[test]
    fn test_character_creation() {
        let character = create_test_character();
        assert_eq!(character.id, "player");
        assert_eq!(character.name, "Test Player");
        assert_eq!(character.location, "sect");
        assert!(character.inventory.is_empty());
    }

    #[test]
    fn test_world_state_from_script() {
        let mut world_setting = WorldSetting::new();
        world_setting.locations = vec![
            Location {
                id: "sect".to_string(),
                name: "Azure Cloud Sect".to_string(),
                description: "A peaceful cultivation sect".to_string(),
                spiritual_energy: 1.0,
            },
            Location {
                id: "city".to_string(),
                name: "Mortal City".to_string(),
                description: "A bustling mortal city".to_string(),
                spiritual_energy: 0.1,
            },
        ];

        let initial_state = InitialState {
            player_name: "Test".to_string(),
            player_spiritual_root: SpiritualRoot {
                element: Element::Fire,
                grade: Grade::Heavenly,
                affinity: 0.8,
            },
            starting_location: "sect".to_string(),
            starting_age: 16,
        };

        let script = Script::new(
            "test".to_string(),
            "Test Script".to_string(),
            ScriptType::Custom,
            world_setting,
            initial_state,
        );

        let world_state = WorldState::from_script(&script);
        assert_eq!(world_state.locations.len(), 2);
        assert!(world_state.locations.contains_key("sect"));
        assert!(world_state.locations.contains_key("city"));
    }

    #[test]
    fn test_game_state_serialization() {
        let character = create_test_character();
        let world_state = WorldState::new();
        let game_time = GameTime::new(1, 1, 1);

        let mut world_setting = WorldSetting::new();
        world_setting.cultivation_realms = vec![
            CultivationRealm::new("Qi Condensation".to_string(), 1, 0, 1.0),
        ];
        world_setting.locations = vec![Location {
            id: "sect".to_string(),
            name: "Azure Cloud Sect".to_string(),
            description: "A peaceful cultivation sect".to_string(),
            spiritual_energy: 1.0,
        }];

        let initial_state = InitialState {
            player_name: "Test".to_string(),
            player_spiritual_root: SpiritualRoot {
                element: Element::Fire,
                grade: Grade::Heavenly,
                affinity: 0.8,
            },
            starting_location: "sect".to_string(),
            starting_age: 16,
        };

        let script = Script::new(
            "test".to_string(),
            "Test Script".to_string(),
            ScriptType::Custom,
            world_setting,
            initial_state,
        );

        let game_state = GameState {
            script,
            player: character,
            world_state,
            game_time,
        };

        // 测试序列化
        let json = serde_json::to_string(&game_state).unwrap();
        assert!(!json.is_empty());

        // 测试反序列化
        let deserialized: GameState = serde_json::from_str(&json).unwrap();
        assert_eq!(game_state, deserialized);
    }
}
