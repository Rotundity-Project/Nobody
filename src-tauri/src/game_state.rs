use crate::event_log::GameEvent;
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
    pub event_history: Vec<GameEvent>,
}

/// 角色数据结构
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Character {
    pub id: String,
    pub name: String,
    pub stats: CharacterStats,
    pub inventory: Vec<Item>,
    pub location: String,
    #[serde(default)]
    pub combat_status: CombatAftermathStatus,
    #[serde(default)]
    pub growth_log: Vec<String>,
    #[serde(default)]
    pub social_profile: SocialProfile,
    #[serde(default)]
    pub personality_tags: Vec<String>,
    #[serde(default)]
    pub technique_tree: Vec<TechniqueNode>,
    #[serde(default)]
    pub equipment_slots: EquipmentSlots,
    #[serde(default = "default_combat_tendency")]
    pub combat_tendency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CombatAftermathStatus {
    pub injury_level: u8,
    pub reputation: i32,
    pub enmity: i32,
    #[serde(default)]
    pub qi_deviation: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SocialProfile {
    pub sect_affinity: i32,
    pub mentor_bond: i32,
    pub vendetta: i32,
    pub favor: i32,
    pub camp_stance: String,
}

impl Default for SocialProfile {
    fn default() -> Self {
        Self {
            sect_affinity: 0,
            mentor_bond: 0,
            vendetta: 0,
            favor: 0,
            camp_stance: "neutral".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TechniqueNode {
    pub name: String,
    pub style: String,
    pub mastery: u8,
    pub risk_level: u8,
    pub unlocked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct EquipmentSlots {
    pub weapon: Option<String>,
    pub armor: Option<String>,
    pub accessory: Option<String>,
    pub artifact: Option<String>,
}

fn default_combat_tendency() -> String {
    "balanced".to_string()
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
    fn derive_personality_tags(stats: &CharacterStats) -> Vec<String> {
        let mut tags = Vec::new();
        if stats.spiritual_root.affinity >= 0.75 {
            tags.push("悟性敏锐".to_string());
        } else if stats.spiritual_root.affinity <= 0.35 {
            tags.push("根基薄弱".to_string());
        }
        match stats.spiritual_root.grade {
            crate::models::Grade::Heavenly => tags.push("天资卓绝".to_string()),
            crate::models::Grade::Pseudo => tags.push("逆境求存".to_string()),
            crate::models::Grade::Double => tags.push("资质均衡".to_string()),
            crate::models::Grade::Triple => tags.push("稳扎稳打".to_string()),
        }
        if stats.techniques.is_empty() {
            tags.push("功法待定型".to_string());
        }
        tags.truncate(4);
        tags
    }

    fn classify_technique_style(name: &str) -> String {
        let lower = name.to_lowercase();
        if lower.contains("剑") || lower.contains("sword") {
            "sword".to_string()
        } else if lower.contains("刀") || lower.contains("blade") {
            "blade".to_string()
        } else if lower.contains("拳") || lower.contains("体") || lower.contains("body") {
            "body".to_string()
        } else if lower.contains("符")
            || lower.contains("阵")
            || lower.contains("array")
            || lower.contains("talisman")
        {
            "talisman".to_string()
        } else {
            "misc".to_string()
        }
    }

    fn estimate_technique_risk(name: &str) -> u8 {
        let lower = name.to_lowercase();
        if lower.contains("禁")
            || lower.contains("爆")
            || lower.contains("噬")
            || lower.contains("逆")
            || lower.contains("魔")
            || lower.contains("forbidden")
            || lower.contains("berserk")
        {
            3
        } else if lower.contains("雷") || lower.contains("thunder") {
            2
        } else {
            1
        }
    }

    fn build_technique_tree(stats: &CharacterStats) -> Vec<TechniqueNode> {
        stats
            .techniques
            .iter()
            .enumerate()
            .map(|(idx, tech)| TechniqueNode {
                name: tech.clone(),
                style: Self::classify_technique_style(tech),
                mastery: ((stats.cultivation_realm.level + stats.cultivation_realm.sub_level) * 8)
                    .min(100) as u8,
                risk_level: Self::estimate_technique_risk(tech),
                unlocked: idx < stats.techniques.len(),
            })
            .collect::<Vec<_>>()
    }

    pub fn refresh_profile_views(&mut self) {
        self.technique_tree = Self::build_technique_tree(&self.stats);
        self.personality_tags = Self::derive_personality_tags(&self.stats);
    }

    pub fn set_combat_tendency(&mut self, tendency: &str) -> bool {
        if tendency.trim().is_empty() {
            return false;
        }
        if self.combat_tendency == tendency {
            return false;
        }
        self.combat_tendency = tendency.to_string();
        true
    }

    pub fn new(
        id: String,
        name: String,
        stats: CharacterStats,
        location: String,
    ) -> Self {
        let personality_tags = Self::derive_personality_tags(&stats);
        let technique_tree = Self::build_technique_tree(&stats);
        Self {
            id,
            name,
            stats,
            inventory: Vec::new(),
            location,
            combat_status: CombatAftermathStatus::default(),
            growth_log: Vec::new(),
            social_profile: SocialProfile::default(),
            personality_tags,
            technique_tree,
            equipment_slots: EquipmentSlots::default(),
            combat_tendency: default_combat_tendency(),
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
            elements: Vec::new(),
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
        assert_eq!(character.combat_status.injury_level, 0);
        assert_eq!(character.combat_status.qi_deviation, 0);
        assert!(character.growth_log.is_empty());
        assert_eq!(character.social_profile.camp_stance, "neutral");
        assert!(!character.personality_tags.is_empty());
        assert!(character.technique_tree.is_empty());
        assert_eq!(character.combat_tendency, "balanced");
        assert_eq!(character.equipment_slots.weapon, None);
    }

    #[test]
    fn test_refresh_profile_views_generates_technique_tree() {
        let mut character = create_test_character();
        character.stats.techniques = vec!["青霜剑诀".to_string(), "禁术爆燃".to_string()];
        character.refresh_profile_views();
        assert_eq!(character.technique_tree.len(), 2);
        assert!(character.technique_tree.iter().any(|n| n.style == "sword"));
        assert!(character.technique_tree.iter().any(|n| n.risk_level >= 3));
    }

    #[test]
    fn test_set_combat_tendency_updates_value() {
        let mut character = create_test_character();
        assert!(character.set_combat_tendency("aggressive"));
        assert_eq!(character.combat_tendency, "aggressive");
        assert!(!character.set_combat_tendency("aggressive"));
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
            elements: Vec::new(),
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
            elements: Vec::new(),
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
            event_history: Vec::new(),
        };

        // 测试序列化
        let json = serde_json::to_string(&game_state).unwrap();
        assert!(!json.is_empty());

        // 测试反序列化
        let deserialized: GameState = serde_json::from_str(&json).unwrap();
        assert_eq!(game_state, deserialized);
    }
}


