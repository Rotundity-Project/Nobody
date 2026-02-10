use crate::models::{CultivationRealm, Element, Grade, SpiritualRoot};
use serde::{Deserialize, Serialize};

/// 剧本类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScriptType {
    ExistingNovel,
    RandomGenerated,
    Custom,
}

/// 世界中的地点
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub id: String,
    pub name: String,
    pub description: String,
    pub spiritual_energy: f32,
}

/// 势力/宗门
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Faction {
    pub id: String,
    pub name: String,
    pub description: String,
    pub power_level: u32,
}

/// 功法/技能
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Technique {
    pub id: String,
    pub name: String,
    pub description: String,
    pub required_realm_level: u32,
    pub element: Option<Element>,
}

/// 世界设定
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldSetting {
    pub cultivation_realms: Vec<CultivationRealm>,
    pub spiritual_roots: Vec<SpiritualRoot>,
    pub techniques: Vec<Technique>,
    pub locations: Vec<Location>,
    pub factions: Vec<Faction>,
}

impl WorldSetting {
    pub fn new() -> Self {
        Self {
            cultivation_realms: Vec::new(),
            spiritual_roots: Vec::new(),
            techniques: Vec::new(),
            locations: Vec::new(),
            factions: Vec::new(),
        }
    }

    pub fn with_default_realms() -> Self {
        let mut setting = Self::new();
        setting.cultivation_realms = vec![
            CultivationRealm::new("练气".to_string(), 1, 0, 1.0),
            CultivationRealm::new("筑基".to_string(), 2, 0, 2.0),
            CultivationRealm::new("金丹".to_string(), 3, 0, 4.0),
            CultivationRealm::new("元婴".to_string(), 4, 0, 8.0),
        ];
        setting
    }

    pub fn with_default_spiritual_roots() -> Self {
        let mut setting = Self::new();
        setting.spiritual_roots = vec![
            SpiritualRoot {
                element: Element::Fire,
                grade: Grade::Heavenly,
                affinity: 0.9,
            },
            SpiritualRoot {
                element: Element::Water,
                grade: Grade::Double,
                affinity: 0.7,
            },
            SpiritualRoot {
                element: Element::Metal,
                grade: Grade::Triple,
                affinity: 0.5,
            },
        ];
        setting
    }
}

impl Default for WorldSetting {
    fn default() -> Self {
        Self::new()
    }
}

/// 游戏的初始状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InitialState {
    pub player_name: String,
    pub player_spiritual_root: SpiritualRoot,
    pub starting_location: String,
    pub starting_age: u32,
}

/// 剧本定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Script {
    pub id: String,
    pub name: String,
    pub script_type: ScriptType,
    pub world_setting: WorldSetting,
    pub initial_state: InitialState,
}

impl Script {
    pub fn new(
        id: String,
        name: String,
        script_type: ScriptType,
        world_setting: WorldSetting,
        initial_state: InitialState,
    ) -> Self {
        Self {
            id,
            name,
            script_type,
            world_setting,
            initial_state,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_setting_creation() {
        let setting = WorldSetting::new();
        assert_eq!(setting.cultivation_realms.len(), 0);
        assert_eq!(setting.locations.len(), 0);
    }

    #[test]
    fn test_world_setting_with_default_realms() {
        let setting = WorldSetting::with_default_realms();
        assert_eq!(setting.cultivation_realms.len(), 4);
        assert_eq!(setting.cultivation_realms[0].name, "练气");
    }

    #[test]
    fn test_script_creation() {
        let world_setting = WorldSetting::with_default_realms();
        let initial_state = InitialState {
            player_name: "Test Player".to_string(),
            player_spiritual_root: SpiritualRoot {
                element: Element::Fire,
                grade: Grade::Heavenly,
                affinity: 0.8,
            },
            starting_location: "Sect".to_string(),
            starting_age: 16,
        };

        let script = Script::new(
            "test_script".to_string(),
            "Test Script".to_string(),
            ScriptType::Custom,
            world_setting,
            initial_state,
        );

        assert_eq!(script.id, "test_script");
        assert_eq!(script.name, "Test Script");
        assert_eq!(script.script_type, ScriptType::Custom);
    }
}
