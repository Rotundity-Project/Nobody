use crate::game_state::GameState;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 游戏持久化的存档/加载系统
pub struct SaveLoadSystem {
    save_directory: PathBuf,
}

/// 包含版本和游戏状态的存档数据结构
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SaveData {
    pub version: String,
    pub timestamp: u64,
    pub game_state: GameState,
}

/// 存档文件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveInfo {
    pub slot_id: u32,
    pub version: String,
    pub timestamp: u64,
    pub player_name: String,
    pub player_age: u32,
    pub game_time: String,
}

impl SaveLoadSystem {
    /// 使用默认存档目录创建新的SaveLoadSystem
    pub fn new() -> Self {
        let save_directory = Self::get_default_save_directory();
        Self { save_directory }
    }

    /// 使用自定义目录创建SaveLoadSystem
    pub fn with_directory(directory: PathBuf) -> Self {
        Self {
            save_directory: directory,
        }
    }

    /// 获取默认存档目录（用户的文档文件夹）
    fn get_default_save_directory() -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            let mut path = PathBuf::from(std::env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string()));
            path.push("Documents");
            path.push("Nobody");
            path.push("saves");
            path
        }

        #[cfg(not(target_os = "windows"))]
        {
            let mut path = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".to_string()));
            path.push(".nobody");
            path.push("saves");
            path
        }
    }

    /// 确保存档目录存在
    fn ensure_save_directory(&self) -> Result<()> {
        if !self.save_directory.exists() {
            fs::create_dir_all(&self.save_directory)?;
        }
        Ok(())
    }

    /// 获取存档槽的存档文件路径
    fn get_save_path(&self, slot_id: u32) -> PathBuf {
        let mut path = self.save_directory.clone();
        path.push(format!("save_{}.json", slot_id));
        path
    }

    /// 保存游戏到存档槽
    pub fn save_game(&self, slot_id: u32, save_data: &SaveData) -> Result<()> {
        self.ensure_save_directory()?;

        // 验证存档数据
        self.validate_save_data(save_data)?;

        let save_path = self.get_save_path(slot_id);
        let json = serde_json::to_string_pretty(save_data)?;
        fs::write(save_path, json)?;

        Ok(())
    }

    /// 从存档槽加载游戏
    pub fn load_game(&self, slot_id: u32) -> Result<SaveData> {
        let save_path = self.get_save_path(slot_id);

        if !save_path.exists() {
            return Err(anyhow!("未找到存档槽 {} 的存档文件", slot_id));
        }

        let json = fs::read_to_string(save_path)?;
        let save_data: SaveData = serde_json::from_str(&json)?;

        // 验证加载的数据
        self.validate_save_data(&save_data)?;

        Ok(save_data)
    }

    /// 列出所有可用的存档
    pub fn list_saves(&self) -> Result<Vec<SaveInfo>> {
        if !self.save_directory.exists() {
            return Ok(Vec::new());
        }

        let mut saves = Vec::new();

        for entry in fs::read_dir(&self.save_directory)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
                    if file_name.starts_with("save_") {
                        if let Ok(slot_id) = file_name[5..].parse::<u32>() {
                            if let Ok(save_data) = self.load_game(slot_id) {
                                let save_info = SaveInfo {
                                    slot_id,
                                    version: save_data.version,
                                    timestamp: save_data.timestamp,
                                    player_name: save_data.game_state.player.name.clone(),
                                    player_age: save_data.game_state.player.stats.lifespan.current_age,
                                    game_time: format!(
                                        "第 {} 年，第 {} 月，第 {} 日",
                                        save_data.game_state.game_time.year,
                                        save_data.game_state.game_time.month,
                                        save_data.game_state.game_time.day
                                    ),
                                };
                                saves.push(save_info);
                            }
                        }
                    }
                }
            }
        }

        saves.sort_by_key(|s| s.slot_id);
        Ok(saves)
    }

    /// 删除存档文件
    pub fn delete_save(&self, slot_id: u32) -> Result<()> {
        let save_path = self.get_save_path(slot_id);

        if !save_path.exists() {
            return Err(anyhow!("未找到存档槽 {} 的存档文件", slot_id));
        }

        fs::remove_file(save_path)?;
        Ok(())
    }

    /// 验证存档数据
    pub fn validate_save_data(&self, save_data: &SaveData) -> Result<()> {
        // 检查版本格式
        if save_data.version.is_empty() {
            return Err(anyhow!("存档数据版本为空"));
        }

        // 检查版本兼容性（当前仅支持 1.0.0）
        if !save_data.version.starts_with("1.") {
            return Err(anyhow!(
                "不兼容的存档版本: {}。期望版本 1.x",
                save_data.version
            ));
        }

        // 检查时间戳是否合理
        if save_data.timestamp == 0 {
            return Err(anyhow!("存档数据中的时间戳无效"));
        }

        // 检查游戏状态是否有必需的数据
        if save_data.game_state.player.name.is_empty() {
            return Err(anyhow!("存档数据中的玩家名称为空"));
        }

        Ok(())
    }
}

impl Default for SaveLoadSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl SaveData {
    /// 从游戏状态创建新的存档数据
    pub fn from_game_state(game_state: GameState) -> Self {
        Self {
            version: "1.0.0".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            game_state,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_state::{Character, GameTime, WorldState};
    use crate::models::{CharacterStats, CultivationRealm, Element, Grade, Lifespan, SpiritualRoot};
    use crate::script::{InitialState, Location, Script, ScriptType, WorldSetting};
    use tempfile::TempDir;

    fn create_test_game_state() -> GameState {
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
            player_name: "Test Player".to_string(),
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

        let player = Character::new(
            "player".to_string(),
            "Test Player".to_string(),
            stats,
            "sect".to_string(),
        );

        let world_state = WorldState::from_script(&script);
        let game_time = GameTime::new(1, 1, 1);

        GameState {
            script,
            player,
            world_state,
            game_time,
        }
    }

    #[test]
    fn test_save_load_system_creation() {
        let system = SaveLoadSystem::new();
        assert!(system.save_directory.to_str().unwrap().contains("saves"));
    }

    #[test]
    fn test_save_and_load_game() {
        let temp_dir = TempDir::new().unwrap();
        let system = SaveLoadSystem::with_directory(temp_dir.path().to_path_buf());

        let game_state = create_test_game_state();
        let save_data = SaveData::from_game_state(game_state.clone());

        // 保存游戏
        let result = system.save_game(1, &save_data);
        assert!(result.is_ok());

        // 加载游戏
        let loaded = system.load_game(1).unwrap();
        assert_eq!(loaded.version, save_data.version);
        assert_eq!(loaded.game_state.player.name, game_state.player.name);
    }

    #[test]
    fn test_load_nonexistent_save() {
        let temp_dir = TempDir::new().unwrap();
        let system = SaveLoadSystem::with_directory(temp_dir.path().to_path_buf());

        let result = system.load_game(999);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("未找到"));
    }

    #[test]
    fn test_list_saves() {
        let temp_dir = TempDir::new().unwrap();
        let system = SaveLoadSystem::with_directory(temp_dir.path().to_path_buf());

        let game_state = create_test_game_state();
        let save_data = SaveData::from_game_state(game_state);

        // 保存到多个槽位
        system.save_game(1, &save_data).unwrap();
        system.save_game(2, &save_data).unwrap();
        system.save_game(3, &save_data).unwrap();

        // 列出存档
        let saves = system.list_saves().unwrap();
        assert_eq!(saves.len(), 3);
        assert_eq!(saves[0].slot_id, 1);
        assert_eq!(saves[1].slot_id, 2);
        assert_eq!(saves[2].slot_id, 3);
    }

    #[test]
    fn test_delete_save() {
        let temp_dir = TempDir::new().unwrap();
        let system = SaveLoadSystem::with_directory(temp_dir.path().to_path_buf());

        let game_state = create_test_game_state();
        let save_data = SaveData::from_game_state(game_state);

        // 保存并删除
        system.save_game(1, &save_data).unwrap();
        assert!(system.load_game(1).is_ok());

        system.delete_save(1).unwrap();
        assert!(system.load_game(1).is_err());
    }

    #[test]
    fn test_validate_save_data() {
        let system = SaveLoadSystem::new();
        let game_state = create_test_game_state();
        let save_data = SaveData::from_game_state(game_state);

        // 有效的存档数据
        assert!(system.validate_save_data(&save_data).is_ok());

        // 无效版本
        let mut invalid_save = save_data.clone();
        invalid_save.version = "".to_string();
        assert!(system.validate_save_data(&invalid_save).is_err());

        // 不兼容的版本
        let mut incompatible_save = save_data.clone();
        incompatible_save.version = "2.0.0".to_string();
        assert!(system.validate_save_data(&incompatible_save).is_err());

        // 无效时间戳
        let mut invalid_timestamp = save_data.clone();
        invalid_timestamp.timestamp = 0;
        assert!(system.validate_save_data(&invalid_timestamp).is_err());

        // 空玩家名称
        let mut empty_name = save_data;
        empty_name.game_state.player.name = "".to_string();
        assert!(system.validate_save_data(&empty_name).is_err());
    }

    #[test]
    fn test_multiple_slots_isolation() {
        let temp_dir = TempDir::new().unwrap();
        let system = SaveLoadSystem::with_directory(temp_dir.path().to_path_buf());

        let mut game_state1 = create_test_game_state();
        game_state1.player.name = "Player 1".to_string();
        let save_data1 = SaveData::from_game_state(game_state1);

        let mut game_state2 = create_test_game_state();
        game_state2.player.name = "Player 2".to_string();
        let save_data2 = SaveData::from_game_state(game_state2);

        // 保存到不同槽位
        system.save_game(1, &save_data1).unwrap();
        system.save_game(2, &save_data2).unwrap();

        // 加载并验证隔离
        let loaded1 = system.load_game(1).unwrap();
        let loaded2 = system.load_game(2).unwrap();

        assert_eq!(loaded1.game_state.player.name, "Player 1");
        assert_eq!(loaded2.game_state.player.name, "Player 2");
    }
}
