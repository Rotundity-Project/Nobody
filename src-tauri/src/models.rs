use serde::{Deserialize, Serialize};

/// ���Ԫ������
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Element {
    Metal,    // ��
    Wood,     // ľ
    Water,    // ˮ
    Fire,     // ��
    Earth,    // ��
    Thunder,  // ��
    Wind,     // ��
    Ice,      // ��
}

/// ���Ʒ��
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Grade {
    Heavenly,      // ��������������
    Pseudo,        // α�������������������
    Triple,        // �����
    Double,        // �����
}

/// ���
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpiritualRoot {
    pub element: Element,  // Ԫ��
    pub grade: Grade,      // Ʒ��
    pub affinity: f32,     // �׺Ͷ� (0.0-1.0)
}

/// ��������
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CultivationRealm {
    pub name: String,              // ��������
    pub level: u32,                // ����ȼ�
    pub sub_level: u32,            // �ӵȼ� (0=����, 1=����, 2=����, 3=��Բ��)
    pub power_multiplier: f32,     // ս������
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
            0 => "����",
            1 => "����",
            2 => "����",
            3 => "��Բ��",
            _ => "δ֪",
        }
    }
}

/// ��Ԫ
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lifespan {
    pub current_age: u32,    // ��ǰ����
    pub max_age: u32,        // ������Ԫ
    pub realm_bonus: u32,    // ����ӳ�
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

/// ��ɫ����
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterStats {
    pub spiritual_root: SpiritualRoot,       // ���
    pub cultivation_realm: CultivationRealm, // ��������
    pub techniques: Vec<String>,             // ��ѧ����
    pub lifespan: Lifespan,                  // ��Ԫ
    pub combat_power: u64,                   // ս��
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
            Grade::Heavenly => 3.0,  // �������ǿ
            Grade::Double => 2.0,    // ˫�����֮
            Grade::Triple => 1.5,    // �����һ��
            Grade::Pseudo => 1.0,    // α�������
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
        let realm = CultivationRealm::new("����".to_string(), 1, 0, 1.0);
        assert_eq!(realm.sub_level_name(), "����");

        let realm = CultivationRealm::new("����".to_string(), 1, 3, 1.0);
        assert_eq!(realm.sub_level_name(), "��Բ��");
    }

    #[test]
    fn test_character_stats_combat_power() {
        let spiritual_root = SpiritualRoot {
            element: Element::Fire,
            grade: Grade::Heavenly,
            affinity: 0.8,
        };
        let realm = CultivationRealm::new("����".to_string(), 1, 0, 1.0);
        let lifespan = Lifespan::new(20, 100, 0);

        let stats = CharacterStats::new(spiritual_root, realm, lifespan);
        assert!(stats.combat_power > 0);
    }
}

// ����ģ�͵����Բ���
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // ���Բ��Ե�����ֵ������
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

    // ���� 4.2: ���� 26 - �����������һ����
    // ����: Nobody, ���� 26: �����������һ����
    // �����κ���Ϸ״̬��������������أ�
    // �ָ���״̬Ӧ����ԭʼ״̬�ȼ�
    proptest! {
        #[test]
        fn test_property_26_spiritual_root_serialization_roundtrip(
            spiritual_root in arb_spiritual_root()
        ) {
            // ���л�Ϊ JSON
            let json = serde_json::to_string(&spiritual_root).unwrap();
            
            // �����л�����
            let restored: SpiritualRoot = serde_json::from_str(&json).unwrap();
            
            // Ӧ�õ���ԭʼֵ
            prop_assert_eq!(spiritual_root, restored);
        }

        #[test]
        fn test_property_26_cultivation_realm_serialization_roundtrip(
            realm in arb_cultivation_realm()
        ) {
            // ���л�Ϊ JSON
            let json = serde_json::to_string(&realm).unwrap();
            
            // �����л�����
            let restored: CultivationRealm = serde_json::from_str(&json).unwrap();
            
            // Ӧ�õ���ԭʼֵ
            prop_assert_eq!(realm, restored);
        }

        #[test]
        fn test_property_26_lifespan_serialization_roundtrip(
            lifespan in arb_lifespan()
        ) {
            // ���л�Ϊ JSON
            let json = serde_json::to_string(&lifespan).unwrap();
            
            // �����л�����
            let restored: Lifespan = serde_json::from_str(&json).unwrap();
            
            // Ӧ�õ���ԭʼֵ
            prop_assert_eq!(lifespan, restored);
        }

        #[test]
        fn test_property_26_character_stats_serialization_roundtrip(
            stats in arb_character_stats()
        ) {
            // ���л�Ϊ JSON
            let json = serde_json::to_string(&stats).unwrap();
            
            // �����л�����
            let restored: CharacterStats = serde_json::from_str(&json).unwrap();
            
            // Ӧ�õ���ԭʼֵ
            prop_assert_eq!(stats, restored);
        }
    }

    // ���л���Ե����Ķ��ⵥԪ����
    #[test]
    fn test_serialization_with_empty_techniques() {
        let stats = CharacterStats::new(
            SpiritualRoot {
                element: Element::Fire,
                grade: Grade::Heavenly,
                affinity: 0.8,
            },
            CultivationRealm::new("����".to_string(), 1, 0, 1.0),
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
            CultivationRealm::new("����".to_string(), 2, 2, 2.5),
            Lifespan::new(50, 150, 100),
        );

        stats.techniques.push("ˮ����".to_string());
        stats.techniques.push("��ì��".to_string());
        stats.techniques.push("������".to_string());

        let json = serde_json::to_string(&stats).unwrap();
        let restored: CharacterStats = serde_json::from_str(&json).unwrap();

        assert_eq!(stats, restored);
        assert_eq!(restored.techniques.len(), 3);
        assert_eq!(restored.techniques[0], "ˮ����");
        assert_eq!(restored.techniques[1], "��ì��");
        assert_eq!(restored.techniques[2], "������");
    }

    #[test]
    fn test_serialization_preserves_combat_power() {
        let stats = CharacterStats::new(
            SpiritualRoot {
                element: Element::Thunder,
                grade: Grade::Heavenly,
                affinity: 0.95,
            },
            CultivationRealm::new("��".to_string(), 3, 3, 5.0),
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
        
        // JSON Ӧ�ð����ֶ���
        assert!(json.contains("element"));
        assert!(json.contains("grade"));
        assert!(json.contains("affinity"));
        assert!(json.contains("Fire"));
        assert!(json.contains("Heavenly"));
    }
}
