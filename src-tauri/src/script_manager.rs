use crate::script::Script;
use anyhow::{anyhow, Result};
use std::path::Path;

/// �ű������������ڼ��غ���֤�ű�
pub struct ScriptManager {}

impl ScriptManager {
    pub fn new() -> Self {
        Self {}
    }

    /// ���ļ������Զ���ű�?
    pub fn load_custom_script(&self, file_path: &str) -> Result<Script> {
        let path = Path::new(file_path);
        
        if !path.exists() {
            return Err(anyhow!("δ�ҵ��ű��ļ�: {}", file_path));
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow!("��ȡ�ű��ļ�ʧ��: {}", e))?;

        let script: Script = serde_json::from_str(&content)
            .map_err(|e| anyhow!("�����ű�JSONʧ��: {}", e))?;

        self.validate_script(&script)?;

        Ok(script)
    }

    /// ��֤�ű��Ƿ�������б����ֶ�?
    pub fn validate_script(&self, script: &Script) -> Result<()> {
        // �������������Ƿ�����������
        if script.world_setting.cultivation_realms.is_empty() {
            return Err(anyhow!(
                "�ű���֤ʧ��: δ������������"
            ));
        }

        // �������������Ƿ�������һ���ص�
        if script.world_setting.locations.is_empty() {
            return Err(anyhow!(
                "�ű���֤ʧ��: δ����ص�?"
            ));
        }

        // ����ʼ״̬�Ƿ�����Ч����ʼ�ص�
        let location_exists = script
            .world_setting
            .locations
            .iter()
            .any(|loc| loc.id == script.initial_state.starting_location);

        if !location_exists {
            return Err(anyhow!(
                "�ű���֤ʧ��: ��ʼ�ص� '{}' �������������������?",
                script.initial_state.starting_location
            ));
        }

        // �����������Ƿ����?
        if script.initial_state.starting_age < 10 || script.initial_state.starting_age > 100 {
            return Err(anyhow!(
                "�ű���֤ʧ��: ��ʼ���� {} ������ (Ӧ��10-100��֮��)",
                script.initial_state.starting_age
            ));
        }

        Ok(())
    }
}

impl Default for ScriptManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CultivationRealm, Element, Grade, SpiritualRoot};
    use crate::script::{InitialState, Location, ScriptType, WorldSetting};

    fn create_valid_script() -> Script {
        let mut world_setting = WorldSetting::new();
        world_setting.cultivation_realms = vec![
            CultivationRealm::new("������".to_string(), 1, 0, 1.0),
        ];
        world_setting.locations = vec![Location {
            id: "sect".to_string(),
            name: "������".to_string(),
            description: "һ����ƽ����������".to_string(),
            spiritual_energy: 1.0,
        }];

        let initial_state = InitialState {
            player_name: "�������?".to_string(),
            player_spiritual_root: SpiritualRoot {
                element: Element::Fire,
                grade: Grade::Heavenly,
                affinity: 0.8,
            },
            starting_location: "sect".to_string(),
            starting_age: 16,
        };

        Script::new(
            "test".to_string(),
            "���Խű�".to_string(),
            ScriptType::Custom,
            world_setting,
            initial_state,
        )
    }

    #[test]
    fn test_validate_script_success() {
        let manager = ScriptManager::new();
        let script = create_valid_script();

        assert!(manager.validate_script(&script).is_ok());
    }

    #[test]
    fn test_validate_script_no_realms() {
        let manager = ScriptManager::new();
        let mut script = create_valid_script();
        script.world_setting.cultivation_realms.clear();

        let result = manager.validate_script(&script);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("δ������������"));
    }

    #[test]
    fn test_validate_script_no_locations() {
        let manager = ScriptManager::new();
        let mut script = create_valid_script();
        script.world_setting.locations.clear();

        let result = manager.validate_script(&script);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("δ����ص�?"));
    }

    #[test]
    fn test_validate_script_invalid_starting_location() {
        let manager = ScriptManager::new();
        let mut script = create_valid_script();
        script.initial_state.starting_location = "nonexistent".to_string();

        let result = manager.validate_script(&script);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("��ʼ�ص�"));
    }

    #[test]
    fn test_validate_script_invalid_age() {
        let manager = ScriptManager::new();
        let mut script = create_valid_script();
        script.initial_state.starting_age = 5;

        let result = manager.validate_script(&script);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("��ʼ����"));
    }

    #[test]
    fn test_load_custom_script_file_not_found() {
        let manager = ScriptManager::new();
        let result = manager.load_custom_script("nonexistent.json");

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("δ�ҵ�"));
    }

    #[test]
    fn test_load_custom_script_valid_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let manager = ScriptManager::new();
        let script = create_valid_script();

        // ����һ��������Ч�ű�JSON����ʱ�ļ�
        let mut temp_file = NamedTempFile::new().unwrap();
        let json = serde_json::to_string_pretty(&script).unwrap();
        temp_file.write_all(json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        // ���ļ����ؽű�
        let result = manager.load_custom_script(temp_file.path().to_str().unwrap());

        assert!(result.is_ok());
        let loaded_script = result.unwrap();
        assert_eq!(loaded_script.id, script.id);
        assert_eq!(loaded_script.name, script.name);
        assert_eq!(loaded_script.script_type, script.script_type);
        assert_eq!(loaded_script.world_setting.cultivation_realms.len(), 1);
        assert_eq!(loaded_script.world_setting.locations.len(), 1);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::models::{CultivationRealm, Element, Grade, SpiritualRoot};
    use crate::script::{Faction, InitialState, Location, ScriptType, Technique, WorldSetting};
    use proptest::prelude::*;

    // �ű����������ֵ������?
    fn arb_element() -> impl Strategy<Value = Element> {
        prop_oneof![
            Just(Element::Fire),
            Just(Element::Water),
            Just(Element::Wood),
            Just(Element::Metal),
            Just(Element::Earth),
        ]
    }

    fn arb_grade() -> impl Strategy<Value = Grade> {
        prop_oneof![
            Just(Grade::Heavenly),
            Just(Grade::Earthly),
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
        ("[a-zA-Z ]{3,20}", 1u32..=10, 0u32..=4, 0.1f32..=10.0f32).prop_map(
            |(name, level, sub_level, power_multiplier)| {
                CultivationRealm::new(name, level, sub_level, power_multiplier)
            },
        )
    }

    fn arb_location() -> impl Strategy<Value = Location> {
        (
            "[a-z]{3,10}",
            "[a-zA-Z ]{3,20}",
            "[a-zA-Z ]{5,50}",
            0.0f32..=10.0f32,
        )
            .prop_map(|(id, name, description, spiritual_energy)| Location {
                id,
                name,
                description,
                spiritual_energy,
            })
    }

    fn arb_faction() -> impl Strategy<Value = Faction> {
        (
            "[a-z]{3,10}",
            "[a-zA-Z ]{3,20}",
            "[a-zA-Z ]{5,50}",
            1u32..=1000,
        )
            .prop_map(|(id, name, description, power_level)| Faction {
                id,
                name,
                description,
                power_level,
            })
    }

    fn arb_technique() -> impl Strategy<Value = Technique> {
        (
            "[a-z]{3,10}",
            "[a-zA-Z ]{3,20}",
            "[a-zA-Z ]{5,50}",
            1u32..=10,
            prop::option::of(arb_element()),
        )
            .prop_map(|(id, name, description, required_realm_level, element)| Technique {
                id,
                name,
                description,
                required_realm_level,
                element,
            })
    }

    fn arb_script_type() -> impl Strategy<Value = ScriptType> {
        prop_oneof![
            Just(ScriptType::ExistingNovel),
            Just(ScriptType::RandomGenerated),
            Just(ScriptType::Custom),
        ]
    }

    // ����ȱ�ٱ����ֶεĽű��Ĳ���
    fn arb_invalid_script() -> impl Strategy<Value = Script> {
        prop_oneof![
            // û����������Ľű�?
            arb_valid_script_base().prop_map(|mut script| {
                script.world_setting.cultivation_realms.clear();
                script
            }),
            // û�еص�Ľű�?
            arb_valid_script_base().prop_map(|mut script| {
                script.world_setting.locations.clear();
                script
            }),
            // ��Ч��ʼ�ص�Ľű�?
            arb_valid_script_base().prop_map(|mut script| {
                script.initial_state.starting_location = "nonexistent_location".to_string();
                script
            }),
            // �����С�Ľű�?
            arb_valid_script_base().prop_map(|mut script| {
                script.initial_state.starting_age = 5;
                script
            }),
            // �������Ľű�
            arb_valid_script_base().prop_map(|mut script| {
                script.initial_state.starting_age = 150;
                script
            }),
        ]
    }

    fn arb_valid_script_base() -> impl Strategy<Value = Script> {
        (
            "[a-z]{3,10}",
            "[a-zA-Z ]{3,20}",
            arb_script_type(),
            prop::collection::vec(arb_cultivation_realm(), 1..5),
            prop::collection::vec(arb_location(), 1..5),
            prop::collection::vec(arb_faction(), 0..5),
            prop::collection::vec(arb_technique(), 0..10),
            "[a-zA-Z ]{3,20}",
            arb_spiritual_root(),
            10u32..=100,
        )
            .prop_map(
                |(
                    id,
                    name,
                    script_type,
                    cultivation_realms,
                    locations,
                    factions,
                    techniques,
                    player_name,
                    player_spiritual_root,
                    starting_age,
                )| {
                    let starting_location = if locations.is_empty() {
                        "sect".to_string() // ����ֵ
                    } else {
                        locations[0].id.clone()
                    };

                    let world_setting = WorldSetting {
                        cultivation_realms,
                        spiritual_roots: vec![player_spiritual_root.clone()],
                        locations,
                        factions,
                        techniques,
                    };

                    let initial_state = InitialState {
                        player_name,
                        player_spiritual_root,
                        starting_location,
                        starting_age,
                    };

                    Script::new(id, name, script_type, world_setting, initial_state)
                },
            )
    }

    // ����: ����֮��, ����3: �ű���֤һ����
    // �����κνű������ȱ�ٱ�Ҫ���������û���ֵϵͳ������?
    // ϵͳӦ�þܾ����ز����������Դ�����Ϣ
    // ��֤����: 1.6, 1.7
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_property_script_validation_rejects_invalid_scripts(
            invalid_script in arb_invalid_script()
        ) {
            let manager = ScriptManager::new();
            let result = manager.validate_script(&invalid_script);

            // ����: ������Ч�ű���Ӧ�ñ��ܾ�
            prop_assert!(
                result.is_err(),
                "ȱ�ٱ����ֶεĽű�Ӧ�ñ��ܾ�"
            );

            // ��֤������Ϣ�������Ե�
            let error_msg = result.unwrap_err().to_string();
            prop_assert!(
                !error_msg.is_empty(),
                "������Ϣ��ӦΪ��"
            );
            prop_assert!(
                error_msg.contains("�ű���֤ʧ��"),
                "������ϢӦָʾ��֤ʧ��: {}",
                error_msg
            );
        }
    }
}