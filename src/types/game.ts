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
  elements?: Element[];
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
  event_history: GameEvent[];
}

export interface GameEvent {
  id: number;
  timestamp: number;
  event_type: string;
  description: string;
  importance: EventImportance;
}

export enum EventImportance {
  Normal = "Normal",
  Important = "Important",
}

export interface Character {
  id: string;
  name: string;
  stats: CharacterStats;
  inventory: string[];
  location: string;
  personality_tags?: string[];
  combat_status?: CombatAftermathStatus;
  growth_log?: string[];
  social_profile?: SocialProfile;
}

export interface CombatAftermathStatus {
  injury_level: number;
  reputation: number;
  enmity: number;
  qi_deviation?: number;
}

export interface SocialProfile {
  sect_affinity: number;
  mentor_bond: number;
  vendetta: number;
  favor: number;
  camp_stance: string;
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

export interface MapLocationOverview {
  location_id: string;
  name: string;
  spiritual_energy: number;
  energy_gap: number;
  reachable: boolean;
  risk_tier: 'low' | 'medium' | 'high' | string;
  estimated_steps?: number;
  suggested_path?: string[];
}

export interface PlotState {
  current_scene: Scene;
  plot_history: string[];
  is_waiting_for_input: boolean;
  interaction_state?: PlotInteractionState;
  last_action_result: ActionResult | null;
  last_generation_diagnostics?: string | null;
  last_option_generation_source?: string | null;
  last_consistency_risk_score?: number | null;
  settings: PlotSettings;
  current_chapter: ChapterState;
  chapters: ChapterState[];
  segment_count: number;
}

export type PlotInteractionState =
  | 'auto_advance'
  | 'waiting_for_choice'
  | 'waiting_for_free_text'
  | 'resolving'
  | 'cooldown';

export interface PlotSettings {
  recap_enabled: boolean;
  novel_style: string;
  llm_priority_mode?: boolean;
  llm_strict_mode?: boolean;
  min_interactions_per_chapter: number;
  max_interactions_per_chapter: number;
  target_chapter_words_min: number;
  target_chapter_words_max: number;
}

export interface ConsistencyPolicy {
  recent_window: number;
  cross_chapter_window: number;
  duplicate_recent_threshold: number;
  duplicate_cross_chapter_threshold: number;
  weight_warning: number;
  weight_critical: number;
  code_weights: Record<string, number>;
}

export interface ChapterState {
  index: number;
  title: string;
  content: string[];
  summary: string;
  interaction_count: number;
  status?: 'in_progress' | 'closed' | 'exported';
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
  meta?: ActionMeta | null;
}

export interface ActionMeta {
  action_kind?: string | null;
}

export interface GenerationTimingSummary {
  sampleCount: number;
  totalP50Ms: number;
  totalP95Ms: number;
  totalP99Ms: number;
  plotGenP95Ms: number;
  optionGenP95Ms: number;
}

export interface GenerationFailureReason {
  stage: string;
  reason: string;
  count: number;
}

export interface GenerationFailureSummary {
  sampleCount: number;
  structuredOkCount: number;
  plainOkCount: number;
  skeletonOkCount: number;
  microOkCount: number;
  presetFallbackCount: number;
  turnUpdateFallbackCount: number;
  optionLlmBlockedCount: number;
  topReasons: GenerationFailureReason[];
}

export type NoNameMode = 'disabled' | 'observeOnly' | 'assisted';

export interface NoNameCapabilityCallRecord {
  capabilityId: string;
  callKind: string;
  status: string;
}

export interface NoNameGuardrailTraceResult {
  outcome: string;
  reason?: string | null;
}

export interface NoNameApplyTraceResult {
  attempted: boolean;
  outcome: string;
  reason?: string | null;
}

export interface NoNameApplyExecutionRecord {
  target: string;
  outcome: string;
  note?: string | null;
}

export interface NoNameApplyPlanRecord {
  order: number;
  target: string;
  decision: string;
  priority: number;
  note?: string | null;
}

export type NoNameControlledOutputDecision = 'allow' | 'reject' | 'needsReview';
export type NoNameControlledOutputKind =
  | 'recapNote'
  | 'sceneAugmentation'
  | 'narrativeNote'
  | 'intermediateNarrativeHint';
export type NoNameForbiddenOutputScope =
  | 'finalPlotState'
  | 'canonWorldFact'
  | 'characterStats'
  | 'inventoryOrResource'
  | 'mapTopology'
  | 'chapterLifecycle'
  | 'playerChoice'
  | 'combatOutcome';

export interface NoNameControlledOutputReviewRecord {
  requestId: string;
  requestedKind: NoNameControlledOutputKind;
  decision: NoNameControlledOutputDecision;
  reason: string;
  normalizedKind?: NoNameControlledOutputKind | null;
  safeApplyScope?: NoNameApplyScope | null;
  policyForbiddenScopes?: NoNameForbiddenOutputScope[];
  requiresHumanReview: boolean;
}

export type NoNameProposalStatus = 'observed' | 'ready' | 'blocked' | 'applied' | 'fallback';
export type NoNameApplyScope =
  | 'diagnostics'
  | 'chapterSummaryHint'
  | 'optionBiasHint'
  | 'plotTextHint';
export type NoNameTargetSegment =
  | 'current_turn_head'
  | 'current_turn_tail'
  | 'chapter_summary_head'
  | 'chapter_summary_tail';

export interface NoNameProposal {
  proposalId: string;
  kind: string;
  producerRole: string;
  title: string;
  summary: string;
  focus: string;
  targetSegment: NoNameTargetSegment;
  intendedEffect: string;
  rationale: string;
  suggestedAction?: string | null;
  labels: string[];
  applyScopes?: NoNameApplyScope[];
  status?: NoNameProposalStatus | null;
  applyable: boolean;
}

export interface NoNameRelatedObservation {
  role: string;
  actionSummary: string;
  focus: string;
  rationale: string;
  proposal: NoNameProposal;
}

export interface NoNameProtocolEvent {
  channel: string;
  from?: string | null;
  to?: string | null;
  kind: string;
  taskId: string;
  status: string;
  detail?: string | null;
}

export interface NoNameTrace {
  traceId: string;
  sessionId: string;
  turnId: string;
  mode: NoNameMode;
  graphPath: string[];
  capabilityCalls: NoNameCapabilityCallRecord[];
  proposals: NoNameProposal[];
  proposalTransitionLog?: string[];
  applyPlanLog?: NoNameApplyPlanRecord[];
  applyExecutionLog?: NoNameApplyExecutionRecord[];
  controlledOutputReviews?: NoNameControlledOutputReviewRecord[];
  relatedObservations?: NoNameRelatedObservation[];
  protocolEvents?: NoNameProtocolEvent[];
  guardrailResult?: NoNameGuardrailTraceResult | null;
  applyResult?: NoNameApplyTraceResult | null;
  fallbackUsed: boolean;
  elapsedMs: number;
}

export interface WorldRegistry {
  session_id: string;
  seed: number;
  created_at: number;
  llm_model?: string | null;
  source: string;
  tables: {
    characters: Record<string, unknown>[];
    map_nodes: Record<string, unknown>[];
    map_edges: Record<string, unknown>[];
    techniques: Record<string, unknown>[];
    inventory_items: Record<string, unknown>[];
    factions: Record<string, unknown>[];
    story_state: Record<string, unknown>[];
    world_facts: Record<string, unknown>[];
  };
}

export interface SaveInfo {
  slot_id: number;
  version: string;
  timestamp: number;
  player_name: string;
  player_age: number;
  realm: string;
  location: string;
  game_time: string;
  noname_mode?: NoNameMode | null;
}

export enum ActionType {
  FreeText = "FreeText",
  SelectedOption = "SelectedOption"
}
