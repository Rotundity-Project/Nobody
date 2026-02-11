use crate::game_state::{Character, GameState, GameTime, WorldState};
use crate::models::{CharacterStats, Lifespan};
use crate::numerical_system::NumericalSystem;
use crate::plot_engine::{PlotEngine, PlotState, Scene};
use crate::save_load::{SaveData, SaveLoadSystem};
use crate::script::Script;
use crate::script_manager::ScriptManager;
use anyhow::{anyhow, Result};
use std::sync::{Arc, Mutex};

/// 管理游戏状态和逻辑的主游戏引擎
pub struct GameEngine {
    state: Arc<Mutex<Option<GameState>>>,
    plot_state: Arc<Mutex<Option<PlotState>>>,
    script_manager: ScriptManager,
    numerical_system: NumericalSystem,
    plot_engine: PlotEngine,
    save_load_system: SaveLoadSystem,
}

impl GameEngine {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(None)),
            plot_state: Arc::new(Mutex::new(None)),
            script_manager: ScriptManager::new(),
            numerical_system: NumericalSystem::new(),
            plot_engine: PlotEngine::new(),
            save_load_system: SaveLoadSystem::new(),
        }
    }

    /// 从剧本初始化游戏
    pub fn initialize_game(&mut self, script: Script) -> Result<GameState> {
        // 验证剧本
        self.script_manager.validate_script(&script)?;

        // 从初始状态创建玩家角色
        let player_stats = CharacterStats {
            spiritual_root: script.initial_state.player_spiritual_root.clone(),
            cultivation_realm: script
                .world_setting
                .cultivation_realms
                .first()
                .ok_or_else(|| anyhow!("剧本中未定义修炼境界"))?
                .clone(),
            techniques: Vec::new(),
            lifespan: Lifespan {
                current_age: script.initial_state.starting_age,
                max_age: 100,
                realm_bonus: 0,
            },
            combat_power: self.numerical_system.calculate_initial_combat_power(
                &script.initial_state.player_spiritual_root,
                script
                    .world_setting
                    .cultivation_realms
                    .first()
                    .ok_or_else(|| anyhow!("未定义修炼境界"))?,
            ),
        };

        let player = Character::new(
            "player".to_string(),
            script.initial_state.player_name.clone(),
            player_stats,
            script.initial_state.starting_location.clone(),
        );

        // 从剧本创建世界状态
        let world_state = WorldState::from_script(&script);

        // 初始化游戏时间
        let game_time = GameTime::new(1, 1, 1);

        // 创建游戏状态
        let game_state = GameState {
            script,
            player,
            world_state,
            game_time,
        };

        // 存储状态
        let mut state_lock = self.state.lock().unwrap();
        *state_lock = Some(game_state.clone());

        Ok(game_state)
    }

    /// 获取当前游戏状态
    pub fn get_current_state(&self) -> Result<GameState> {
        let state_lock = self.state.lock().unwrap();
        state_lock
            .clone()
            .ok_or_else(|| anyhow!("游戏未初始化"))
    }

    /// 检查游戏是否已初始化
    pub fn is_initialized(&self) -> bool {
        let state_lock = self.state.lock().unwrap();
        state_lock.is_some()
    }

    /// 保存游戏到存档槽
    pub fn save_game(&self, slot_id: u32) -> Result<()> {
        let state_lock = self.state.lock().unwrap();
        let game_state = state_lock
            .as_ref()
            .ok_or_else(|| anyhow!("无法保存：游戏未初始化"))?;

        let save_data = SaveData::from_game_state(game_state.clone());
        self.save_load_system.save_game(slot_id, &save_data)?;

        Ok(())
    }

    /// 从存档槽加载游戏
    pub fn load_game(&mut self, slot_id: u32) -> Result<GameState> {
        let save_data = self.save_load_system.load_game(slot_id)?;
        let game_state = save_data.game_state;

        // 存储加载的状态
        let mut state_lock = self.state.lock().unwrap();
        *state_lock = Some(game_state.clone());

        Ok(game_state)
    }

    /// 初始化剧情状态
    pub fn initialize_plot(&mut self) -> Result<PlotState> {
        let state_lock = self.state.lock().unwrap();
        let game_state = state_lock
            .as_ref()
            .ok_or_else(|| anyhow!("无法初始化剧情：游戏未初始化"))?;

        // 创建初始场景
        let mut initial_scene = Scene::new(
            "start".to_string(),
            "开始".to_string(),
            format!(
                "你是{}，刚刚踏入修仙之路。你的灵根是{:?}，当前境界是{}。",
                game_state.player.name,
                game_state.player.stats.spiritual_root.element,
                game_state.player.stats.cultivation_realm.name
            ),
            game_state.player.location.clone(),
        );

        // 生成初始玩家选项
        let options = self.plot_engine.generate_player_options(&initial_scene, &game_state.player.stats);
        for option in options {
            initial_scene.add_option(option);
        }

        let plot_state = PlotState::new(initial_scene);

        // 存储剧情状态
        let mut plot_lock = self.plot_state.lock().unwrap();
        *plot_lock = Some(plot_state.clone());

        Ok(plot_state)
    }

    /// 获取当前剧情状态
    pub fn get_plot_state(&self) -> Result<PlotState> {
        let plot_lock = self.plot_state.lock().unwrap();
        plot_lock
            .clone()
            .ok_or_else(|| anyhow!("剧情未初始化"))
    }

    /// 更新剧情状态
    pub fn update_plot_state(&self, new_plot_state: PlotState) -> Result<()> {
        let mut plot_lock = self.plot_state.lock().unwrap();
        *plot_lock = Some(new_plot_state);
        Ok(())
    }
}

impl Default for GameEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CultivationRealm, Element, Grade, SpiritualRoot};
    use crate::script::{InitialState, Location, ScriptType, WorldSetting};

    fn create_test_script() -> Script {
        let mut world_setting = WorldSetting::new();
        world_setting.cultivation_realms = vec![
            CultivationRealm::new("Qi Condensation".to_string(), 1, 0, 1.0),
            CultivationRealm::new("Foundation Establishment".to_string(), 2, 0, 2.0),
        ];
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
            player_name: "Test Player".to_string(),
            player_spiritual_root: SpiritualRoot {
                element: Element::Fire,
                grade: Grade::Heavenly,
                affinity: 0.9,
            },
            starting_location: "sect".to_string(),
            starting_age: 16,
        };

        Script::new(
            "test".to_string(),
            "Test Script".to_string(),
            ScriptType::Custom,
            world_setting,
            initial_state,
        )
    }

    #[test]
    fn test_game_engine_creation() {
        let engine = GameEngine::new();
        assert!(!engine.is_initialized());
    }

    #[test]
    fn test_initialize_game() {
        let mut engine = GameEngine::new();
        let script = create_test_script();

        let result = engine.initialize_game(script.clone());
        assert!(result.is_ok());

        let game_state = result.unwrap();
        assert_eq!(game_state.player.name, "Test Player");
        assert_eq!(game_state.player.location, "sect");
        assert_eq!(game_state.player.stats.lifespan.current_age, 16);
        assert_eq!(game_state.game_time.year, 1);
        assert_eq!(game_state.world_state.locations.len(), 2);
    }

    #[test]
    fn test_get_current_state() {
        let mut engine = GameEngine::new();
        let script = create_test_script();

        // 初始化之前
        assert!(engine.get_current_state().is_err());

        // 初始化之后
        engine.initialize_game(script).unwrap();
        assert!(engine.get_current_state().is_ok());
        assert!(engine.is_initialized());
    }

    #[test]
    fn test_initialize_game_with_invalid_script() {
        let mut engine = GameEngine::new();
        let mut script = create_test_script();

        // 移除修炼境界使剧本无效
        script.world_setting.cultivation_realms.clear();

        let result = engine.initialize_game(script);
        assert!(result.is_err());
    }

    #[test]
    fn test_player_initial_stats() {
        let mut engine = GameEngine::new();
        let script = create_test_script();

        let game_state = engine.initialize_game(script).unwrap();

        // 验证玩家具有正确的初始属性
        assert_eq!(
            game_state.player.stats.spiritual_root.element,
            Element::Fire
        );
        assert_eq!(game_state.player.stats.spiritual_root.grade, Grade::Heavenly);
        assert_eq!(game_state.player.stats.cultivation_realm.name, "Qi Condensation");
        assert!(game_state.player.stats.combat_power > 0);
        assert!(game_state.player.inventory.is_empty());
    }

    #[test]
    fn test_world_state_initialization() {
        let mut engine = GameEngine::new();
        let script = create_test_script();

        let game_state = engine.initialize_game(script).unwrap();

        // 验证世界状态正确初始化
        assert!(game_state.world_state.locations.contains_key("sect"));
        assert!(game_state.world_state.locations.contains_key("city"));
        assert!(game_state.world_state.global_events.is_empty());
    }

    #[test]
    fn test_plot_initialization() {
        let mut engine = GameEngine::new();
        let script = create_test_script();

        // 初始化游戏
        engine.initialize_game(script).unwrap();

        // 初始化剧情
        let plot_state = engine.initialize_plot().unwrap();

        // 验证剧情状态正确初始化
        assert_eq!(plot_state.current_scene.id, "start");
        assert!(plot_state.is_waiting_for_input);
        assert!(plot_state.plot_history.is_empty());
        assert!(plot_state.last_action_result.is_none());

        // 验证可以获取剧情状态
        let retrieved_plot = engine.get_plot_state().unwrap();
        assert_eq!(retrieved_plot.current_scene.id, "start");
    }

    #[test]
    fn test_save_game() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let mut engine = GameEngine::new();
        engine.save_load_system = SaveLoadSystem::with_directory(temp_dir.path().to_path_buf());

        let script = create_test_script();
        engine.initialize_game(script).unwrap();

        // 保存游戏
        let result = engine.save_game(1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_save_without_initialization() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let _engine = GameEngine::new();
        let mut engine_with_dir = GameEngine::new();
        engine_with_dir.save_load_system = SaveLoadSystem::with_directory(temp_dir.path().to_path_buf());

        // 尝试在未初始化时保存
        let result = engine_with_dir.save_game(1);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("未初始化"));
    }

    #[test]
    fn test_load_game() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let mut engine = GameEngine::new();
        engine.save_load_system = SaveLoadSystem::with_directory(temp_dir.path().to_path_buf());

        let script = create_test_script();
        let original_state = engine.initialize_game(script).unwrap();

        // 保存游戏
        engine.save_game(1).unwrap();

        // 创建新引擎并加载
        let mut new_engine = GameEngine::new();
        new_engine.save_load_system = SaveLoadSystem::with_directory(temp_dir.path().to_path_buf());

        let loaded_state = new_engine.load_game(1).unwrap();

        // 验证加载的状态与原始状态匹配
        assert_eq!(loaded_state.player.name, original_state.player.name);
        assert_eq!(loaded_state.player.location, original_state.player.location);
        assert_eq!(
            loaded_state.player.stats.lifespan.current_age,
            original_state.player.stats.lifespan.current_age
        );
    }

    #[test]
    fn test_save_load_roundtrip() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let mut engine = GameEngine::new();
        engine.save_load_system = SaveLoadSystem::with_directory(temp_dir.path().to_path_buf());

        let script = create_test_script();
        engine.initialize_game(script).unwrap();

        // 保存并加载
        engine.save_game(1).unwrap();
        let loaded_state = engine.load_game(1).unwrap();

        // 验证状态被保留
        let current_state = engine.get_current_state().unwrap();
        assert_eq!(current_state.player.name, loaded_state.player.name);
        assert_eq!(current_state.game_time.year, loaded_state.game_time.year);
    }
}

// 任务 12.2: 游戏引擎的集成测试
// 测试完整的保存-加载流程和游戏继续运行
#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::models::{CultivationRealm, Element, Grade, SpiritualRoot};
    use crate::script::{InitialState, Location, ScriptType, WorldSetting};
    use tempfile::TempDir;

    fn create_test_script() -> Script {
        let mut world_setting = WorldSetting::new();
        world_setting.cultivation_realms = vec![
            CultivationRealm::new("练气".to_string(), 1, 0, 1.0),
            CultivationRealm::new("筑基".to_string(), 2, 0, 2.0),
        ];
        world_setting.locations = vec![
            Location {
                id: "sect".to_string(),
                name: "青云宗".to_string(),
                description: "一个和平的修仙宗门".to_string(),
                spiritual_energy: 1.0,
            },
            Location {
                id: "city".to_string(),
                name: "凡人城市".to_string(),
                description: "繁华的凡人城市".to_string(),
                spiritual_energy: 0.1,
            },
        ];

        let initial_state = InitialState {
            player_name: "测试玩家".to_string(),
            player_spiritual_root: SpiritualRoot {
                element: Element::Fire,
                grade: Grade::Heavenly,
                affinity: 0.9,
            },
            starting_location: "sect".to_string(),
            starting_age: 16,
        };

        Script::new(
            "test".to_string(),
            "测试剧本".to_string(),
            ScriptType::Custom,
            world_setting,
            initial_state,
        )
    }

    #[test]
    fn test_complete_save_load_workflow() {
        // 测试完整的保存-加载流程
        // 验证需求: 9.3, 9.4

        let temp_dir = TempDir::new().unwrap();
        let mut engine = GameEngine::new();
        engine.save_load_system = SaveLoadSystem::with_directory(temp_dir.path().to_path_buf());

        // 1. 初始化游戏
        let script = create_test_script();
        let initial_state = engine.initialize_game(script).unwrap();
        
        assert_eq!(initial_state.player.name, "测试玩家");
        assert_eq!(initial_state.player.stats.lifespan.current_age, 16);
        assert_eq!(initial_state.game_time.year, 1);

        // 2. 修改游戏状态（模拟游戏进行）
        {
            let mut state_lock = engine.state.lock().unwrap();
            if let Some(ref mut state) = *state_lock {
                state.player.stats.lifespan.current_age = 20;
                state.game_time.year = 2;
            }
        }

        // 3. 保存游戏
        let save_result = engine.save_game(1);
        assert!(save_result.is_ok(), "保存游戏失败");

        // 4. 创建新引擎并加载
        let mut new_engine = GameEngine::new();
        new_engine.save_load_system = SaveLoadSystem::with_directory(temp_dir.path().to_path_buf());

        let loaded_state = new_engine.load_game(1).unwrap();

        // 5. 验证加载的状态正确
        assert_eq!(loaded_state.player.name, "测试玩家");
        assert_eq!(loaded_state.player.stats.lifespan.current_age, 20);
        assert_eq!(loaded_state.game_time.year, 2);

        // 6. 验证加载后游戏可以继续运行
        let current_state = new_engine.get_current_state().unwrap();
        assert_eq!(current_state.player.name, loaded_state.player.name);
        assert!(new_engine.is_initialized());
    }

    #[test]
    fn test_game_continues_after_load() {
        // 测试加载后游戏能继续运行
        // 验证需求: 9.3, 9.4

        let temp_dir = TempDir::new().unwrap();
        let mut engine = GameEngine::new();
        engine.save_load_system = SaveLoadSystem::with_directory(temp_dir.path().to_path_buf());

        // 初始化并保存
        let script = create_test_script();
        engine.initialize_game(script).unwrap();
        engine.save_game(1).unwrap();

        // 加载游戏
        let mut new_engine = GameEngine::new();
        new_engine.save_load_system = SaveLoadSystem::with_directory(temp_dir.path().to_path_buf());
        new_engine.load_game(1).unwrap();

        // 验证可以获取当前状态
        let state = new_engine.get_current_state();
        assert!(state.is_ok(), "加载后无法获取游戏状态");

        // 验证可以再次保存
        let save_again = new_engine.save_game(2);
        assert!(save_again.is_ok(), "加载后无法再次保存");

        // 验证两个存档独立
        let save1 = new_engine.save_load_system.load_game(1).unwrap();
        let save2 = new_engine.save_load_system.load_game(2).unwrap();
        assert_eq!(save1.game_state.player.name, save2.game_state.player.name);
    }

    #[test]
    fn test_multiple_save_load_cycles() {
        // 测试多次保存-加载循环
        // 验证需求: 9.3, 9.4

        let temp_dir = TempDir::new().unwrap();
        let mut engine = GameEngine::new();
        engine.save_load_system = SaveLoadSystem::with_directory(temp_dir.path().to_path_buf());

        // 初始化游戏
        let script = create_test_script();
        engine.initialize_game(script).unwrap();

        // 进行多次保存-加载循环
        for i in 1..=5 {
            // 修改状态
            {
                let mut state_lock = engine.state.lock().unwrap();
                if let Some(ref mut state) = *state_lock {
                    state.game_time.year = i;
                }
            }

            // 保存
            engine.save_game(i).unwrap();

            // 加载
            let loaded = engine.load_game(i).unwrap();
            assert_eq!(loaded.game_time.year, i, "第 {} 次循环状态不一致", i);
        }

        // 验证所有存档都存在且独立
        for i in 1..=5 {
            let save = engine.save_load_system.load_game(i).unwrap();
            assert_eq!(save.game_state.game_time.year, i);
        }
    }

    #[test]
    fn test_save_preserves_all_game_data() {
        // 测试保存保留所有游戏数据
        // 验证需求: 9.3

        let temp_dir = TempDir::new().unwrap();
        let mut engine = GameEngine::new();
        engine.save_load_system = SaveLoadSystem::with_directory(temp_dir.path().to_path_buf());

        // 初始化游戏
        let script = create_test_script();
        engine.initialize_game(script).unwrap();

        // 修改多个方面的状态
        {
            let mut state_lock = engine.state.lock().unwrap();
            if let Some(ref mut state) = *state_lock {
                state.player.stats.lifespan.current_age = 25;
                state.player.stats.techniques.push("火球术".to_string());
                state.player.location = "city".to_string();
                state.game_time.year = 3;
                state.game_time.month = 6;
                state.game_time.day = 15;
            }
        }

        // 保存并加载
        engine.save_game(1).unwrap();
        let loaded = engine.load_game(1).unwrap();

        // 验证所有数据都被保留
        assert_eq!(loaded.player.stats.lifespan.current_age, 25);
        assert_eq!(loaded.player.stats.techniques.len(), 1);
        assert_eq!(loaded.player.stats.techniques[0], "火球术");
        assert_eq!(loaded.player.location, "city");
        assert_eq!(loaded.game_time.year, 3);
        assert_eq!(loaded.game_time.month, 6);
        assert_eq!(loaded.game_time.day, 15);
    }

    #[test]
    fn test_load_updates_engine_state() {
        // 测试加载正确更新引擎状态
        // 验证需求: 9.2, 9.4

        let temp_dir = TempDir::new().unwrap();
        
        // 第一个引擎：初始化并保存
        let mut engine1 = GameEngine::new();
        engine1.save_load_system = SaveLoadSystem::with_directory(temp_dir.path().to_path_buf());
        let script = create_test_script();
        engine1.initialize_game(script).unwrap();
        
        {
            let mut state_lock = engine1.state.lock().unwrap();
            if let Some(ref mut state) = *state_lock {
                state.player.stats.lifespan.current_age = 30;
            }
        }
        engine1.save_game(1).unwrap();

        // 第二个引擎：加载
        let mut engine2 = GameEngine::new();
        engine2.save_load_system = SaveLoadSystem::with_directory(temp_dir.path().to_path_buf());
        
        // 加载前未初始化
        assert!(!engine2.is_initialized());
        
        // 加载
        engine2.load_game(1).unwrap();
        
        // 加载后已初始化
        assert!(engine2.is_initialized());
        
        // 状态正确
        let state = engine2.get_current_state().unwrap();
        assert_eq!(state.player.stats.lifespan.current_age, 30);
    }
}
