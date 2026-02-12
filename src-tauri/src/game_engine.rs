use crate::event_log::{EventImportance, EventLog};
use crate::game_state::{Character, GameState, GameTime, WorldState};
use crate::models::{CharacterStats, Element, Grade, Lifespan, SpiritualRoot};
use crate::npc::{CoreValue, Goal, NPC, NPCMemory, Personality, PersonalityTrait};
use crate::npc_engine::{NPCDecision, NPCEngine, NPCEvent};
use crate::numerical_system::NumericalSystem;
use crate::plot_engine::{PlotEngine, PlotState, Scene};
use crate::save_load::{SaveData, SaveInfo, SaveLoadSystem};
use crate::script::{Script, ScriptType};
use crate::script_manager::ScriptManager;
use anyhow::{anyhow, Result};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// 管理游戏状态和逻辑的主游戏引擎
pub struct GameEngine {
    state: Arc<Mutex<Option<GameState>>>,
    plot_state: Arc<Mutex<Option<PlotState>>>,
    script_manager: ScriptManager,
    numerical_system: NumericalSystem,
    plot_engine: PlotEngine,
    npc_engine: NPCEngine,
    event_log: Arc<Mutex<EventLog>>,
    save_load_system: SaveLoadSystem,
}

const EVENT_LOG_MAX_EVENTS: usize = 600;
const EVENT_LOG_MAX_IMPORTANT: usize = 200;
const EVENT_LOG_MAX_ARCHIVES: usize = 50;

#[derive(Debug, Clone)]
struct RandomStartProfile {
    starting_realm: crate::models::CultivationRealm,
    spiritual_root: SpiritualRoot,
    starting_location: String,
    starting_age: u32,
    max_age: u32,
}

impl GameEngine {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(None)),
            plot_state: Arc::new(Mutex::new(None)),
            script_manager: ScriptManager::new(),
            numerical_system: NumericalSystem::new(),
            plot_engine: PlotEngine::new(),
            npc_engine: NPCEngine::new(),
            event_log: Arc::new(Mutex::new(EventLog::new())),
            save_load_system: SaveLoadSystem::new(),
        }
    }

    fn random_seed() -> u64 {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        // Mix with stack address to reduce repeat rate in rapid consecutive calls.
        let addr_mix = (&nanos as *const u64 as usize) as u64;
        nanos ^ addr_mix.rotate_left(17)
    }

    fn rng_next(seed: &mut u64) -> u64 {
        let mut x = *seed;
        if x == 0 {
            x = 0x9E37_79B9_7F4A_7C15;
        }
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        *seed = x;
        x
    }

    fn rand_u32(seed: &mut u64, min: u32, max: u32) -> u32 {
        if min >= max {
            return min;
        }
        let span = (max - min + 1) as u64;
        min + (Self::rng_next(seed) % span) as u32
    }

    fn rand_f32(seed: &mut u64, min: f32, max: f32) -> f32 {
        if min >= max {
            return min;
        }
        let val = (Self::rng_next(seed) as f64 / u64::MAX as f64) as f32;
        min + (max - min) * val
    }

    fn choose_weighted_index(seed: &mut u64, weights: &[u32]) -> usize {
        let total: u32 = weights.iter().sum();
        if total == 0 {
            return 0;
        }
        let mut roll = Self::rand_u32(seed, 1, total);
        for (idx, weight) in weights.iter().enumerate() {
            if *weight == 0 {
                continue;
            }
            if roll <= *weight {
                return idx;
            }
            roll -= *weight;
        }
        weights.len().saturating_sub(1)
    }

    fn random_root_grade(seed: &mut u64) -> Grade {
        // 单灵根10%，双灵根30%，三灵根40%，杂灵根20%
        match Self::choose_weighted_index(seed, &[10, 30, 40, 20]) {
            0 => Grade::Heavenly,
            1 => Grade::Double,
            2 => Grade::Triple,
            _ => Grade::Pseudo,
        }
    }

    fn random_element(seed: &mut u64) -> Element {
        match Self::rand_u32(seed, 0, 4) {
            0 => Element::Metal,
            1 => Element::Wood,
            2 => Element::Water,
            3 => Element::Fire,
            _ => Element::Earth,
        }
    }

    fn build_random_start_profile(&self, script: &Script) -> Option<RandomStartProfile> {
        let mut realms = script.world_setting.cultivation_realms.clone();
        if realms.is_empty() {
            return None;
        }
        realms.sort_by_key(|r| r.level);

        let mut seed = Self::random_seed();
        let realm_weights = match realms.len() {
            0 => vec![],
            1 => vec![100],
            2 => vec![75, 25],
            3 => vec![70, 22, 8],
            _ => {
                let mut w = vec![68, 20, 8];
                let remain_slots = realms.len() - 3;
                let mut remain = 4u32;
                for _ in 0..remain_slots {
                    let portion = (remain / remain_slots as u32).max(1);
                    w.push(portion);
                    remain = remain.saturating_sub(portion);
                }
                w
            }
        };
        let realm_idx = Self::choose_weighted_index(&mut seed, &realm_weights)
            .min(realms.len().saturating_sub(1));
        let mut starting_realm = realms[realm_idx].clone();
        starting_realm.sub_level = Self::rand_u32(&mut seed, 0, 2);

        let grade = Self::random_root_grade(&mut seed);
        let affinity = match grade {
            Grade::Heavenly => Self::rand_f32(&mut seed, 0.90, 1.00),
            Grade::Double => Self::rand_f32(&mut seed, 0.78, 0.92),
            Grade::Triple => Self::rand_f32(&mut seed, 0.62, 0.82),
            Grade::Pseudo => Self::rand_f32(&mut seed, 0.40, 0.70),
        };
        let spiritual_root = SpiritualRoot {
            element: Self::random_element(&mut seed),
            grade: grade.clone(),
            affinity,
        };

        let starting_location = if script.world_setting.locations.is_empty() {
            script.initial_state.starting_location.clone()
        } else {
            let idx = Self::rand_u32(&mut seed, 0, script.world_setting.locations.len() as u32 - 1)
                as usize;
            script.world_setting.locations[idx].id.clone()
        };

        let base_age_min = 15 + (realm_idx as u32 * 2);
        let base_age_max = base_age_min + 8;
        let starting_age = Self::rand_u32(&mut seed, base_age_min, base_age_max);
        let grade_bonus = match grade {
            Grade::Heavenly => 35,
            Grade::Double => 22,
            Grade::Triple => 12,
            Grade::Pseudo => 0,
        };
        let realm_bonus = (starting_realm.level.saturating_sub(1) * 8) as i32;
        let jitter = Self::rand_u32(&mut seed, 0, 10) as i32;
        let mut max_age = 95 + grade_bonus + realm_bonus + jitter;
        let min_required = starting_age as i32 + 40;
        if max_age < min_required {
            max_age = min_required;
        }

        Some(RandomStartProfile {
            starting_realm,
            spiritual_root,
            starting_location,
            starting_age,
            max_age: max_age as u32,
        })
    }

    /// 从剧本初始化游戏
    pub fn initialize_game(&mut self, script: Script) -> Result<GameState> {
        // 验证剧本
        self.script_manager.validate_script(&script)?;

        // 从初始状态创建玩家角色
        let mut starting_realm = script
            .world_setting
            .cultivation_realms
            .iter()
            .find(|realm| realm.name == "练气" || realm.name == "炼气")
            .or_else(|| script.world_setting.cultivation_realms.first())
            .ok_or_else(|| anyhow!("剧本中未定义修炼境界"))?
            .clone();
        let mut player_spiritual_root = script.initial_state.player_spiritual_root.clone();
        let mut starting_location = script.initial_state.starting_location.clone();
        let mut starting_age = script.initial_state.starting_age;
        let mut max_age = 100u32;

        // 随机剧本每次开局都重新随机角色信息，避免固定模板体验。
        if script.script_type == ScriptType::RandomGenerated {
            if let Some(profile) = self.build_random_start_profile(&script) {
                starting_realm = profile.starting_realm;
                player_spiritual_root = profile.spiritual_root;
                starting_location = profile.starting_location;
                starting_age = profile.starting_age;
                max_age = profile.max_age;
            }
        }

        let player_stats = CharacterStats {
            spiritual_root: player_spiritual_root.clone(),
            cultivation_realm: starting_realm.clone(),
            techniques: Vec::new(),
            lifespan: Lifespan {
                current_age: starting_age,
                max_age,
                realm_bonus: 0,
            },
            combat_power: self.numerical_system.calculate_initial_combat_power(
                &player_spiritual_root,
                &starting_realm,
            ),
        };

        let player = Character::new(
            "player".to_string(),
            script.initial_state.player_name.clone(),
            player_stats,
            starting_location,
        );

        // 从剧本创建世界状态
        let world_state = WorldState::from_script(&script);

        // 初始化游戏时间
        let game_time = GameTime::new(1, 1, 1);

        // 创建游戏状态
        let mut game_state = GameState {
            script,
            player,
            world_state,
            game_time,
            event_history: Vec::new(),
        };

        {
            let mut log = self.event_log.lock().unwrap();
            *log = EventLog::new();
            log.log_event(
                u64::from(game_state.game_time.total_days),
                "game_start",
                format!("已为 {} 初始化游戏", game_state.player.name),
                EventImportance::Important,
            );
            game_state.event_history = log.all_events().to_vec();
        }

        // 初始化新局 NPC，避免沿用旧局状态。
        self.initialize_npcs_for_new_game(&game_state);

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

    /// 更新当前游戏状态
    pub fn update_current_state(&self, new_state: GameState) -> Result<()> {
        let mut state_lock = self.state.lock().unwrap();
        *state_lock = Some(new_state);
        Ok(())
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

        self.log_event(
            u64::from(game_state.game_time.total_days),
            "save",
            format!("已保存到槽位 {}", slot_id),
            EventImportance::Normal,
        );
        let mut save_state = game_state.clone();
        save_state.event_history = self.snapshot_event_history();
        let plot_snapshot = {
            let plot_lock = self.plot_state.lock().unwrap();
            plot_lock.clone()
        };
        let save_data = SaveData::from_game_state_with_plot(save_state, plot_snapshot);
        self.save_load_system.save_game(slot_id, &save_data)?;

        Ok(())
    }

    /// 从存档槽加载游戏
    pub fn load_game(&mut self, slot_id: u32) -> Result<GameState> {
        let save_data = self.save_load_system.load_game(slot_id)?;
        let mut game_state = save_data.game_state;
        {
            let mut log = self.event_log.lock().unwrap();
            *log = EventLog::from_events(game_state.event_history.clone());
            log.log_event(
                u64::from(game_state.game_time.total_days),
                "load",
                format!("已从槽位 {} 读取存档", slot_id),
                EventImportance::Important,
            );
            game_state.event_history = log.all_events().to_vec();
        }

        // 存储加载的状态
        let mut state_lock = self.state.lock().unwrap();
        *state_lock = Some(game_state.clone());
        drop(state_lock);

        // 优先恢复存档中的剧情状态，避免读档后剧情丢失。
        if let Some(saved_plot_state) = save_data.plot_state {
            let mut plot_lock = self.plot_state.lock().unwrap();
            *plot_lock = Some(saved_plot_state);
        } else {
            // 兼容旧存档：若无剧情状态，则重建默认开篇。
            self.initialize_plot()?;
        }

        Ok(game_state)
    }

    /// 初始化剧情状态
    pub fn initialize_plot(&mut self) -> Result<PlotState> {
        let game_state = self
            .state
            .lock()
            .unwrap()
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow!("无法初始化剧情：游戏未初始化"))?;

        let opening_text = self.plot_engine.generate_opening_plot(
            &game_state.player.name,
            &game_state.player.stats.cultivation_realm.name,
            &format!("{:?}", game_state.player.stats.spiritual_root.element),
            &game_state.player.location,
        );

        self.initialize_plot_with_opening(opening_text, None)
    }

    pub fn initialize_plot_with_opening(
        &mut self,
        opening_text: String,
        opening_options: Option<Vec<crate::plot_engine::PlayerOption>>,
    ) -> Result<PlotState> {
        let game_state = self
            .state
            .lock()
            .unwrap()
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow!("无法初始化剧情：游戏未初始化"))?;

        let mut initial_scene = Scene::new(
            "start".to_string(),
            "第一章".to_string(),
            opening_text.clone(),
            game_state.player.location,
        );

        if let Some(options) = opening_options {
            for option in options {
                initial_scene.add_option(option);
            }
        } else {
            // 生成初始玩家选项
            let options = self
                .plot_engine
                .generate_player_options(&initial_scene, &game_state.player.stats);
            for option in options {
                initial_scene.add_option(option);
            }
        }

        let mut plot_state = PlotState::new(initial_scene);
        plot_state.append_segment(opening_text);

        // 存储剧情状态
        let mut plot_lock = self.plot_state.lock().unwrap();
        *plot_lock = Some(plot_state.clone());

        self.log_event(
            self.current_timestamp(),
            "plot_init",
            "剧情已初始化到当前场景".to_string(),
            EventImportance::Normal,
        );
        self.sync_event_history_to_state();

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

    pub fn update_plot_settings(&self, settings: crate::plot_engine::PlotSettings) -> Result<PlotState> {
        let mut plot_lock = self.plot_state.lock().unwrap();
        let state = plot_lock
            .as_mut()
            .ok_or_else(|| anyhow!("剧情未初始化"))?;
        state.settings = settings;
        Ok(state.clone())
    }


    pub fn process_npc_reactions_for_events(
        &mut self,
        events: &[String],
    ) -> Result<Vec<NPCDecision>> {
        let mut all_decisions = Vec::new();
        for (idx, event_text) in events.iter().enumerate() {
            let event = NPCEvent {
                timestamp: idx as u64 + 1,
                description: event_text.clone(),
                involved_npc_ids: Vec::new(),
                importance: 0.7,
                emotional_impact: 0.2,
                affinity_impact: 1,
                trust_impact: 1,
            };
            self.log_event(
                event.timestamp,
                "story_event",
                event.description.clone(),
                EventImportance::Normal,
            );
            let decisions = self.npc_engine.process_event(&event);
            for decision in &decisions {
                self.log_event(
                    event.timestamp,
                    "npc_reaction",
                    format!("{} -> {}", decision.npc_id, decision.action),
                    EventImportance::Normal,
                );
            }
            all_decisions.extend(decisions);
        }
        self.sync_event_history_to_state();
        Ok(all_decisions)
    }

    fn initialize_npcs_for_new_game(&mut self, game_state: &GameState) {
        self.npc_engine = NPCEngine::new();

        let npc = NPC {
            id: "npc_elder_1".to_string(),
            name: "Sect Elder".to_string(),
            stats: CharacterStats {
                spiritual_root: game_state.player.stats.spiritual_root.clone(),
                cultivation_realm: game_state.player.stats.cultivation_realm.clone(),
                techniques: vec!["Guidance".to_string()],
                lifespan: Lifespan {
                    current_age: 80,
                    max_age: 180,
                    realm_bonus: 40,
                },
                combat_power: game_state.player.stats.combat_power.saturating_add(200),
            },
            personality: Personality {
                traits: vec![PersonalityTrait::Calm, PersonalityTrait::Righteous],
                goals: vec![Goal {
                    description: "Guide young disciples".to_string(),
                    priority: 8,
                }],
                values: vec![CoreValue {
                    name: "Order".to_string(),
                    weight: 0.9,
                }],
            },
            memory: NPCMemory::default(),
            relationships: std::collections::HashMap::new(),
        };

        self.npc_engine.insert_npc(npc);
    }
    /// 列出存档槽信息
    pub fn list_saves(&self) -> Result<Vec<SaveInfo>> {
        self.save_load_system.list_saves()
    }

    pub fn log_event(
        &self,
        timestamp: u64,
        event_type: impl Into<String>,
        description: impl Into<String>,
        importance: EventImportance,
    ) {
        let mut log = self.event_log.lock().unwrap();
        log.log_event(timestamp, event_type, description, importance);
        log.archive_if_needed(
            EVENT_LOG_MAX_EVENTS,
            EVENT_LOG_MAX_IMPORTANT,
            EVENT_LOG_MAX_ARCHIVES,
        );
    }

    fn snapshot_event_history(&self) -> Vec<crate::event_log::GameEvent> {
        let log = self.event_log.lock().unwrap();
        log.all_events().to_vec()
    }

    fn sync_event_history_to_state(&self) {
        let history = self.snapshot_event_history();
        let mut state_lock = self.state.lock().unwrap();
        if let Some(ref mut state) = *state_lock {
            state.event_history = history;
        }
    }

    fn current_timestamp(&self) -> u64 {
        self.get_current_state()
            .map(|s| u64::from(s.game_time.total_days))
            .unwrap_or(0)
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
    use crate::numerical_system::Action;
    use crate::plot_engine::{PlayerOption, PlotSettings};
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

    fn create_random_script() -> Script {
        let mut script = create_test_script();
        script.script_type = ScriptType::RandomGenerated;
        script
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
        assert!(!plot_state.plot_history.is_empty());
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

    #[test]
    fn test_random_generated_script_uses_randomized_profile() {
        let mut engine = GameEngine::new();
        let script = create_random_script();
        let game_state = engine.initialize_game(script.clone()).unwrap();

        let realm_names = script
            .world_setting
            .cultivation_realms
            .iter()
            .map(|r| r.name.clone())
            .collect::<Vec<String>>();
        assert!(realm_names.contains(&game_state.player.stats.cultivation_realm.name));

        let location_ids = script
            .world_setting
            .locations
            .iter()
            .map(|l| l.id.clone())
            .collect::<Vec<String>>();
        assert!(location_ids.contains(&game_state.player.location));

        assert!((15..=40).contains(&game_state.player.stats.lifespan.current_age));
        assert!(
            game_state.player.stats.lifespan.max_age
                >= game_state.player.stats.lifespan.current_age + 40
        );
        assert!(game_state.player.stats.combat_power > 0);
    }

    #[test]
    fn test_update_current_state_replaces_state() {
        let mut engine = GameEngine::new();
        let script = create_test_script();
        let mut state = engine.initialize_game(script).unwrap();
        state.player.name = "Updated Player".to_string();

        engine.update_current_state(state.clone()).unwrap();
        let current = engine.get_current_state().unwrap();
        assert_eq!(current.player.name, "Updated Player");
    }

    #[test]
    fn test_get_plot_state_before_initialization_fails() {
        let engine = GameEngine::new();
        let result = engine.get_plot_state();
        assert!(result.is_err());
    }

    #[test]
    fn test_initialize_plot_without_game_fails() {
        let mut engine = GameEngine::new();
        let result = engine.initialize_plot();
        assert!(result.is_err());
    }

    #[test]
    fn test_initialize_plot_with_custom_opening_options() {
        let mut engine = GameEngine::new();
        let script = create_test_script();
        engine.initialize_game(script).unwrap();

        let custom_options = vec![PlayerOption {
            id: 0,
            description: "自定义开局选项".to_string(),
            requirements: vec![],
            action: Action::Rest,
        }];

        let plot_state = engine
            .initialize_plot_with_opening("自定义开场".to_string(), Some(custom_options))
            .unwrap();

        assert_eq!(plot_state.current_scene.description, "自定义开场");
        assert_eq!(plot_state.current_scene.available_options.len(), 1);
        assert_eq!(plot_state.current_scene.available_options[0].description, "自定义开局选项");
    }

    #[test]
    fn test_update_plot_settings_requires_initialized_plot() {
        let engine = GameEngine::new();
        let result = engine.update_plot_settings(PlotSettings::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_update_plot_settings_success() {
        let mut engine = GameEngine::new();
        let script = create_test_script();
        engine.initialize_game(script).unwrap();
        engine.initialize_plot().unwrap();

        let settings = PlotSettings {
            recap_enabled: false,
            novel_style: "纪实风格".to_string(),
            min_interactions_per_chapter: 2,
            max_interactions_per_chapter: 4,
            target_chapter_words_min: 1500,
            target_chapter_words_max: 2500,
        };

        let updated = engine.update_plot_settings(settings.clone()).unwrap();
        assert_eq!(updated.settings, settings);
    }

    #[test]
    fn test_update_plot_state_replaces_state() {
        let mut engine = GameEngine::new();
        let script = create_test_script();
        engine.initialize_game(script).unwrap();
        let mut plot = engine.initialize_plot().unwrap();
        plot.current_scene.name = "更新后的章节".to_string();

        engine.update_plot_state(plot.clone()).unwrap();
        let current = engine.get_plot_state().unwrap();
        assert_eq!(current.current_scene.name, "更新后的章节");
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
    fn test_property_22_plot_triggers_npc_reactions() {
        let mut engine = GameEngine::new();
        let script = create_test_script();
        engine.initialize_game(script).unwrap();

        let events = vec!["player breaks through a minor bottleneck".to_string()];
        let reactions = engine.process_npc_reactions_for_events(&events).unwrap();

        assert!(!reactions.is_empty());
        assert!(reactions.iter().all(|r| !r.action.is_empty()));
    }

    #[test]
    fn test_event_log_records_story_and_npc_reaction() {
        let mut engine = GameEngine::new();
        let script = create_test_script();
        engine.initialize_game(script).unwrap();

        let events = vec!["battle erupted near the sect gate".to_string()];
        let reactions = engine.process_npc_reactions_for_events(&events).unwrap();
        assert!(!reactions.is_empty());

        let state = engine.get_current_state().unwrap();
        assert!(!state.event_history.is_empty());
        assert!(state
            .event_history
            .iter()
            .any(|e| e.event_type.as_ref() == "story_event" && !e.description.is_empty()));
        assert!(state
            .event_history
            .iter()
            .any(|e| e.event_type.as_ref() == "npc_reaction" && !e.description.is_empty()));
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






