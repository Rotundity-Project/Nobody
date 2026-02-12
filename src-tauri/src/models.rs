use serde::{Deserialize, Serialize};

/// 灵根元素类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Element {
    Metal,    // 金
    Wood,     // 木
    Water,    // 水
    Fire,     // 火
    Earth,    // 土
    Thunder,  // 雷
    Wind,     // 风
    Ice,      // 冰
}

/// 灵根品质
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Grade {
    Heavenly,      // 天灵根（单灵根）
    Pseudo,        // 伪灵根（四灵根或五灵根）
    Triple,        // 三灵根
    Double,        // 双灵根
}

/// 灵根
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpiritualRoot {
    pub element: Element,  // 元素
    pub grade: Grade,      // 品质
    pub affinity: f32,     // 亲和度 (0.0-1.0)
}

/// 修炼境界
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CultivationRealm {
    pub name: String,              // 境界名称
    pub level: u32,                // 境界等级
    pub sub_level: u32,            // 子等级 (0=初期, 1=中期, 2=后期, 3=圆满期)
    pub power_multiplier: f32,     // 战力倍数
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
            0 => "初期",
            1 => "中期",
            2 => "后期",
            3 => "圆满期",
            _ => "未知",
        }
    }
}

/// 寿元
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lifespan {
    pub current_age: u32,    // 当前年龄
    pub max_age: u32,        // 基础寿元
    pub realm_bonus: u32,    // 境界增长
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

/// 角色属性
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterStats {
    pub spiritual_root: SpiritualRoot,       // 灵根
    pub cultivation_realm: CultivationRealm, // 修炼境界
    pub techniques: Vec<String>,             // 已学功法
    pub lifespan: Lifespan,                  // 寿元
    pub combat_power: u64,                   // 战力
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
            Grade::Heavenly => 3.0,  // 天灵根
            Grade::Double => 2.0,    // 双灵根
            Grade::Triple => 1.5,    // 三灵根
            Grade::Pseudo => 1.0,    // 伪灵根
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
        let realm = CultivationRealm::new("练气".to_string(), 1, 0, 1.0);
        assert_eq!(realm.sub_level_name(), "初期");

        let realm = CultivationRealm::new("练气".to_string(), 1, 3, 1.0);
        assert_eq!(realm.sub_level_name(), "圆满期");
    }

    #[test]
    fn test_character_stats_combat_power() {
        let spiritual_root = SpiritualRoot {
            element: Element::Fire,
            grade: Grade::Heavenly,
            affinity: 0.8,
        };
        let realm = CultivationRealm::new("练气".to_string(), 1, 0, 1.0);
        let lifespan = Lifespan::new(20, 100, 0);

        let stats = CharacterStats::new(spiritual_root, realm, lifespan);
        assert!(stats.combat_power > 0);
    }
}

// 数据模型的属性测试
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // 属性测试的生成器
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
            Just(Grade::Pseudo),
            Just(Grade::Triple),
            Just(Grade::Double),
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

    // 任务 4.2: 属性 26 - 序列化反序列化一致性
    // 功能: Nobody, 属性 26: 序列化反序列化一致性
    // 对任何游戏状态，经过序列化和反序列化后，
    // 恢复的状态应该与原始状态等价
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_property_26_spiritual_root_serialization_roundtrip(
            spiritual_root in arb_spiritual_root()
        ) {
            // 序列化为 JSON
            let json = serde_json::to_string(&spiritual_root).unwrap();
            
            // 反序列化回来
            let restored: SpiritualRoot = serde_json::from_str(&json).unwrap();
            
            // 应该等于原始值
            prop_assert_eq!(spiritual_root, restored);
        }

        #[test]
        fn test_property_26_cultivation_realm_serialization_roundtrip(
            realm in arb_cultivation_realm()
        ) {
            // 序列化为 JSON
            let json = serde_json::to_string(&realm).unwrap();
            
            // 反序列化回来
            let restored: CultivationRealm = serde_json::from_str(&json).unwrap();
            
            // 应该等于原始值
            prop_assert_eq!(realm, restored);
        }

        #[test]
        fn test_property_26_lifespan_serialization_roundtrip(
            lifespan in arb_lifespan()
        ) {
            // 序列化为 JSON
            let json = serde_json::to_string(&lifespan).unwrap();
            
            // 反序列化回来
            let restored: Lifespan = serde_json::from_str(&json).unwrap();
            
            // 应该等于原始值
            prop_assert_eq!(lifespan, restored);
        }

        #[test]
        fn test_property_26_character_stats_serialization_roundtrip(
            stats in arb_character_stats()
        ) {
            // 序列化为 JSON
            let json = serde_json::to_string(&stats).unwrap();
            
            // 反序列化回来
            let restored: CharacterStats = serde_json::from_str(&json).unwrap();
            
            // 应该等于原始值
            prop_assert_eq!(stats, restored);
        }
    }

    // 序列化特性的额外单元测试
    #[test]
    fn test_serialization_with_empty_techniques() {
        let stats = CharacterStats::new(
            SpiritualRoot {
                element: Element::Fire,
                grade: Grade::Heavenly,
                affinity: 0.8,
            },
            CultivationRealm::new("练气".to_string(), 1, 0, 1.0),
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
                grade: Grade::Double,
                affinity: 0.6,
            },
            CultivationRealm::new("筑基".to_string(), 2, 2, 2.5),
            Lifespan::new(50, 150, 100),
        );

        stats.techniques.push("水龙吟".to_string());
        stats.techniques.push("冰魄术".to_string());
        stats.techniques.push("寒冰诀".to_string());

        let json = serde_json::to_string(&stats).unwrap();
        let restored: CharacterStats = serde_json::from_str(&json).unwrap();

        assert_eq!(stats, restored);
        assert_eq!(restored.techniques.len(), 3);
        assert_eq!(restored.techniques[0], "水龙吟");
        assert_eq!(restored.techniques[1], "冰魄术");
        assert_eq!(restored.techniques[2], "寒冰诀");
    }

    #[test]
    fn test_serialization_preserves_combat_power() {
        let stats = CharacterStats::new(
            SpiritualRoot {
                element: Element::Thunder,
                grade: Grade::Heavenly,
                affinity: 0.95,
            },
            CultivationRealm::new("金丹".to_string(), 3, 3, 5.0),
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
        
        // JSON 应该包含字段名
        assert!(json.contains("element"));
        assert!(json.contains("grade"));
        assert!(json.contains("affinity"));
        assert!(json.contains("Fire"));
        assert!(json.contains("Heavenly"));
    }
}
