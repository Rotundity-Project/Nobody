// Game types matching Rust backend structures

export interface Script {
  id: string;
  name: string;
  script_type: ScriptType;
  world_setting: WorldSetting;
  initial_state: InitialState;
}

export enum ScriptType {
  ExistingNovel = "existing_novel",
  RandomGenerated = "random_generated",
  Custom = "custom"
}

export interface WorldSetting {
  cultivation_realms: CultivationRealm[];
  spiritual_roots: SpiritualRoot[];
  techniques: Technique[];
  locations: Location[];
  factions: Faction[];
}

export interface Technique {
  id: string;
  name: string;
  description: string;
  required_realm_level: number;
  element: Element | null;
}

export interface CultivationRealm {
  name: string;
  level: number;
  sub_level: number;
  power_multiplier: number;
}

export interface SpiritualRoot {
  element: Element;
  grade: Grade;
  affinity: number;
}

export enum Element {
  Fire = "Fire",
  Water = "Water",
  Wood = "Wood",
  Metal = "Metal",
  Earth = "Earth"
}

export enum Grade {
  Heavenly = "Heavenly",
  Pseudo = "Pseudo",
  Triple = "Triple",
  Double = "Double"
}

export interface Location {
  id: string;
  name: string;
  description: string;
  spiritual_energy: number;
}

export interface Faction {
  id: string;
  name: string;
  description: string;
  power_level: number;
}

export interface InitialState {
  player_name: string;
  player_spiritual_root: SpiritualRoot;
  starting_location: string;
  starting_age: number;
}

export interface GameState {
  script: Script;
  player: Character;
  world_state: WorldState;
  game_time: GameTime;
}

export interface Character {
  id: string;
  name: string;
  stats: CharacterStats;
  inventory: string[];
  location: string;
}

export interface CharacterStats {
  spiritual_root: SpiritualRoot;
  cultivation_realm: CultivationRealm;
  techniques: string[];
  lifespan: Lifespan;
  combat_power: number;
}

export interface Lifespan {
  current_age: number;
  max_age: number;
  realm_bonus: number;
}

export interface WorldState {
  locations: Record<string, Location>;
  factions: Record<string, Faction>;
  global_events: string[];
}

export interface GameTime {
  year: number;
  month: number;
  day: number;
  total_days: number;
}

export interface PlotState {
  current_scene: Scene;
  plot_history: string[];
  is_waiting_for_input: boolean;
  last_action_result: ActionResult | null;
}

export interface Scene {
  id: string;
  name: string;
  description: string;
  location: string;
  available_options: PlayerOption[];
}

export interface PlayerOption {
  id: number;
  description: string;
  requirements: string[];
  action: Action;
}

export interface Action {
  Cultivate?: null;
  Breakthrough?: null;
  Rest?: null;
  Custom?: { description: string };
}

export interface ActionResult {
  success: boolean;
  description: string;
  stat_changes: StatChange[];
  events: string[];
}

export interface StatChange {
  stat_name: string;
  old_value: number;
  new_value: number;
}

export interface PlayerAction {
  action_type: ActionType;
  content: string;
  selected_option_id: number | null;
}

export enum ActionType {
  FreeText = "FreeText",
  SelectedOption = "SelectedOption"
}
