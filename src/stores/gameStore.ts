import { defineStore } from 'pinia';
import { invokeRuntime, invokeWithTimeout } from '../utils/tauriInvoke';
import type {
  Script,
  GameState,
  PlotState,
  PlayerAction,
  PlayerOption,
  MapLocationOverview,
  SaveInfo,
  WorldRegistry,
  GenerationTimingSummary,
  GenerationFailureSummary,
  NoNameTrace,
} from '../types/game';

interface GameStoreState {
  currentScript: Script | null;
  gameState: GameState | null;
  plotState: PlotState | null;
  lastInitializationDurationMs: number | null;
  reachableLocationIds: string[];
  mapOverview: MapLocationOverview[];
  worldRegistry: WorldRegistry | null;
  generationDiagnostics: string[];
  generationTimingSummary: GenerationTimingSummary | null;
  generationFailureSummary: GenerationFailureSummary | null;
  noNameTraces: NoNameTrace[];
  backgroundNotice: { id: string; message: string } | null;
  isLoading: boolean;
  error: string | null;
}

const LLM_TIMEOUT_MS = 60 * 1000;
const OPTION_LLM_STORY_BLOCKED_MARKER = '本轮为选项续写：未获得可用 LLM 剧情文本';
const MAX_GENERATION_DIAGNOSTICS = 40;

export const useGameStore = defineStore('game', {
  state: (): GameStoreState => ({
    currentScript: null,
    gameState: null,
    plotState: null,
    lastInitializationDurationMs: null,
    reachableLocationIds: [],
    mapOverview: [],
    worldRegistry: null,
    generationDiagnostics: [],
    generationTimingSummary: null,
    generationFailureSummary: null,
    noNameTraces: [],
    backgroundNotice: null,
    isLoading: false,
    error: null,
  }),

  getters: {
    isGameInitialized: (state) => state.gameState !== null,
    isPlotInitialized: (state) => state.plotState !== null,
    playerCharacter: (state) => state.gameState?.player || null,
    currentScene: (state) => state.plotState?.current_scene || null,
    availableOptions: (state) => state.plotState?.current_scene.available_options || [],
    isWaitingForInput: (state) => state.plotState?.is_waiting_for_input || false,
  },

  actions: {
    async tryRehydrateLastQuickModeSegment() {
      try {
        const replaced = await invokeRuntime<boolean>('rehydrate_last_quick_mode_segment', undefined);
        if (!replaced) {
          return;
        }
        const plotState = await invokeWithTimeout<PlotState>(
          'get_plot_state',
          undefined,
          8000,
          '获取剧情状态超时，请重试',
        );
        this.plotState = plotState;
        this.appendGenerationDiagnostics(plotState.last_generation_diagnostics);
        await this.refreshGenerationDiagnosticsSummary();
        await this.refreshNoNameTraces();
        this.backgroundNotice = {
          id: `rehydrate-${Date.now()}`,
          message: '快速模式结果已补全为完整叙事版本',
        };
      } catch (error) {
        console.warn('快速模式后台补全失败，已忽略：', error);
      }
    },

    async refreshReachableLocations() {
      try {
        const ids = await invokeRuntime<unknown>('get_reachable_locations', undefined);
        this.reachableLocationIds = Array.isArray(ids)
          ? ids.filter((id): id is string => typeof id === 'string')
          : [];
      } catch {
        this.reachableLocationIds = [];
      }
    },

    async refreshMapOverview() {
      try {
        const nodes = await invokeRuntime<MapLocationOverview[]>('get_map_overview', undefined);
        this.mapOverview = Array.isArray(nodes) ? nodes : [];
      } catch {
        this.mapOverview = [];
      }
    },

    async refreshWorldRegistry() {
      try {
        this.worldRegistry = await invokeRuntime<WorldRegistry>('get_world_registry', undefined);
      } catch {
        this.worldRegistry = null;
      }
    },

    async refreshNoNameTraces() {
      try {
        const traces = await invokeRuntime<NoNameTrace[]>('get_noname_recent_traces', undefined);
        this.noNameTraces = Array.isArray(traces) ? traces : [];
      } catch {
        this.noNameTraces = [];
      }
    },

    appendGenerationDiagnostics(diag: string | null | undefined) {
      const text = String(diag ?? '').trim();
      if (!text) {
        return;
      }
      this.generationDiagnostics = [
        ...this.generationDiagnostics.slice(-(MAX_GENERATION_DIAGNOSTICS - 1)),
        text,
      ];
    },

    async refreshGenerationDiagnosticsSummary() {
      if (this.generationDiagnostics.length === 0) {
        this.generationTimingSummary = null;
        this.generationFailureSummary = null;
        return;
      }
      try {
        this.generationTimingSummary = await invokeRuntime<GenerationTimingSummary>(
          'summarize_generation_diagnostics',
          { diagnostics: this.generationDiagnostics },
        );
      } catch {
        this.generationTimingSummary = null;
      }
      try {
        this.generationFailureSummary = await invokeRuntime<GenerationFailureSummary>(
          'summarize_generation_failures',
          { diagnostics: this.generationDiagnostics },
        );
      } catch {
        this.generationFailureSummary = null;
      }
    },

    clearGenerationDiagnostics() {
      this.generationDiagnostics = [];
      this.generationTimingSummary = null;
      this.generationFailureSummary = null;
    },

    async clearNoNameTraces() {
      try {
        await invokeRuntime('clear_noname_recent_traces', undefined);
      } catch {
        // Older runtimes or web mocks may not implement this command.
      } finally {
        this.noNameTraces = [];
      }
    },

    getGenerationDiagnosticsText() {
      if (this.generationDiagnostics.length === 0) {
        return '暂无诊断数据。';
      }
      return this.generationDiagnostics
        .map((line, index) => `${index + 1}. ${line}`)
        .join('\n');
    },

    getNoNameTraceDebugText() {
      if (this.noNameTraces.length === 0) {
        return '暂无 NoName Agent Trace。';
      }
      const latest = this.noNameTraces[this.noNameTraces.length - 1];
      const path = latest.graphPath.join(' -> ') || '无';
      const calls = latest.capabilityCalls.length > 0
        ? latest.capabilityCalls
          .map((item) => `${item.callKind}:${item.capabilityId}[${item.status}]`)
          .join(', ')
        : '无';
      const proposal = latest.proposals.length > 0
        ? latest.proposals[latest.proposals.length - 1]
        : null;
      const proposalStatus = proposal?.status ?? (proposal?.applyable ? 'ready' : 'observed');
      const proposalScopes = proposal?.applyScopes && proposal.applyScopes.length > 0
        ? proposal.applyScopes.join(', ')
        : '无';
      const applyExecutions = latest.applyExecutionLog && latest.applyExecutionLog.length > 0
        ? latest.applyExecutionLog
          .map((item) => `${item.target}:${item.outcome}${item.note ? `(${item.note})` : ''}`)
          .join(', ')
        : '无';
      const applyPlans = latest.applyPlanLog && latest.applyPlanLog.length > 0
        ? latest.applyPlanLog
          .map((item) => `#${item.order}:${item.target}:${item.decision}@${item.priority}${item.note ? `(${item.note})` : ''}`)
          .join(', ')
        : '无';
      const relatedObservations = latest.relatedObservations && latest.relatedObservations.length > 0
        ? latest.relatedObservations
          .map((item) => `${item.role}:${item.focus}`)
          .join(', ')
        : '无';
      const protocolEvents = latest.protocolEvents && latest.protocolEvents.length > 0
        ? latest.protocolEvents
          .slice(-6)
          .map((item) => `${item.channel}:${item.kind}:${item.status}`)
          .join(', ')
        : '无';
      return [
        `最近 Trace：${latest.traceId}`,
        `模式：${latest.mode}`,
        `路径：${path}`,
        `能力调用：${calls}`,
        `提案：${proposal ? `${proposal.title} / ${proposal.focus} / ${proposalStatus}` : '无'}`,
        `协作观察：${relatedObservations}`,
        `协议事件：${protocolEvents}`,
        `目标段：${proposal?.targetSegment || '无'}`,
        `预期效果：${proposal?.intendedEffect || '无'}`,
        `作用域：${proposalScopes}`,
        `预检：${latest.applyResult ? `${latest.applyResult.outcome}${latest.applyResult.reason ? ` (${latest.applyResult.reason})` : ''}` : '无'}`,
        `应用计划：${applyPlans}`,
        `应用执行：${applyExecutions}`,
        `状态迁移：${latest.proposalTransitionLog && latest.proposalTransitionLog.length > 0 ? latest.proposalTransitionLog.join(', ') : '无'}`,
        `护栏：${latest.guardrailResult ? `${latest.guardrailResult.outcome}${latest.guardrailResult.reason ? ` (${latest.guardrailResult.reason})` : ''}` : '无'}`,
        `回退：${latest.fallbackUsed ? '是' : '否'}`,
        `耗时：${latest.elapsedMs} ms`,
        `历史条数：${this.noNameTraces.length}`,
      ].join('\n');
    },

    async applyWorldRegistryPatch(patch: unknown) {
      this.isLoading = true;
      this.error = null;
      try {
        this.worldRegistry = await invokeRuntime<WorldRegistry>('apply_world_registry_patch', { patch });
        const gameState = await invokeRuntime<GameState>('get_game_state', undefined);
        this.gameState = gameState;
      } catch (error) {
        this.error = error instanceof Error ? error.message : String(error);
        throw error;
      } finally {
        this.isLoading = false;
      }
    },

    async initializeGame(script: Script, playerName?: string) {
      this.isLoading = true;
      this.error = null;
      this.lastInitializationDurationMs = null;
      const startedAt = Date.now();

      try {
        const trimmedName = playerName?.trim();
        script.initial_state.player_name = trimmedName || '无名弟子';
        const gameState = await invokeWithTimeout<GameState>(
          'initialize_game',
          { script },
          LLM_TIMEOUT_MS,
          '初始化世界超时，请检查 LLM 配置后重试',
        );
        this.currentScript = script;
        this.gameState = gameState;

        const plotState = await invokeWithTimeout<PlotState>(
          'initialize_plot',
          undefined,
          LLM_TIMEOUT_MS,
          '初始化剧情超时，请检查 LLM 配置后重试',
        );
        this.plotState = plotState;
        this.appendGenerationDiagnostics(plotState.last_generation_diagnostics);
        await this.refreshGenerationDiagnosticsSummary();
        await this.refreshNoNameTraces();
        this.lastInitializationDurationMs = Date.now() - startedAt;
        await this.refreshWorldRegistry();
        await this.refreshReachableLocations();
        await this.refreshMapOverview();
      } catch (error) {
        this.error = error instanceof Error ? error.message : String(error);
        throw error;
      } finally {
        this.isLoading = false;
      }
    },

    async executePlayerAction(action: PlayerAction) {
      this.isLoading = true;
      this.error = null;

      try {
        await invokeWithTimeout<string>(
          'execute_player_action',
          { action },
          LLM_TIMEOUT_MS,
          '剧情推进超时，请稍后重试',
        );

        const gameState = await invokeWithTimeout<GameState>(
          'get_game_state',
          undefined,
          8000,
          '获取游戏状态超时，请重试',
        );
        this.gameState = gameState;

        const plotState = await invokeWithTimeout<PlotState>(
          'get_plot_state',
          undefined,
          8000,
          '获取剧情状态超时，请重试',
        );
        this.plotState = plotState;
        this.appendGenerationDiagnostics(plotState.last_generation_diagnostics);
        await this.refreshGenerationDiagnosticsSummary();
        await this.refreshNoNameTraces();
        await this.refreshWorldRegistry();
        await this.refreshReachableLocations();
        await this.refreshMapOverview();

        // 显示 LLM 诊断信息（如果有）
        if (plotState.last_generation_diagnostics) {
          console.warn('LLM 诊断信息:', plotState.last_generation_diagnostics);
        }
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);

        if (message.includes('剧情推进超时')) {
          try {
            await invokeWithTimeout<string>(
              'execute_player_action',
              { action, quickMode: true },
              15000,
              '快速模式执行超时，请稍后重试',
            );
            const gameState = await invokeWithTimeout<GameState>(
              'get_game_state',
              undefined,
              8000,
              '获取游戏状态超时，请重试',
            );
            this.gameState = gameState;
            const quickPlotState = await invokeWithTimeout<PlotState>(
              'get_plot_state',
              undefined,
              10000,
              '获取剧情状态超时，请重试',
            );
            this.plotState = quickPlotState;
            this.appendGenerationDiagnostics(quickPlotState.last_generation_diagnostics);
            await this.refreshGenerationDiagnosticsSummary();
            await this.refreshNoNameTraces();
            await this.refreshWorldRegistry();
            await this.refreshReachableLocations();
            await this.refreshMapOverview();
            this.error = `主链路超时，已切换快速模式续写。${quickPlotState.last_generation_diagnostics ?? ''}`.trim();
            void this.tryRehydrateLastQuickModeSegment();
          } catch {
            try {
              const latestPlotState = await invokeWithTimeout<PlotState>(
                'get_plot_state',
                undefined,
                10000,
                '获取剧情状态超时，请重试',
              );
              this.plotState = latestPlotState;
              this.appendGenerationDiagnostics(latestPlotState.last_generation_diagnostics);
              await this.refreshGenerationDiagnosticsSummary();
              await this.refreshNoNameTraces();
              this.error = latestPlotState.last_generation_diagnostics ?? '剧情推进超时，请稍后重试';
            } catch {
              this.error = '剧情推进超时，请稍后重试。你可以尝试重连或调整 LLM 设置。';
            }
          }
        } else {
          if (message.includes(OPTION_LLM_STORY_BLOCKED_MARKER)) {
            this.error = '本次选项续写未获取到 LLM 剧情文本，系统已拦截预设剧情回退。请检查并更新 LLM 配置后重试。';
          } else {
            this.error = `操作失败: ${message}`;
          }
        }
        throw error;
      } finally {
        this.isLoading = false;
      }
    },
    async saveGame(slotId: number) {
      this.isLoading = true;
      this.error = null;

      try {
        await invokeRuntime('save_game', { slotId });
      } catch (error) {
        this.error = error instanceof Error ? error.message : String(error);
        throw error;
      } finally {
        this.isLoading = false;
      }
    },

    async loadGame(slotId: number) {
      this.isLoading = true;
      this.error = null;

      try {
        const gameState = await invokeRuntime<GameState>('load_game', { slotId });
        this.gameState = gameState;

        const plotState = await invokeRuntime<PlotState>('get_plot_state', undefined);
        this.plotState = plotState;
        this.appendGenerationDiagnostics(plotState.last_generation_diagnostics);
        await this.refreshGenerationDiagnosticsSummary();
        await this.refreshNoNameTraces();
        await this.refreshWorldRegistry();
        await this.refreshReachableLocations();
        await this.refreshMapOverview();
      } catch (error) {
        this.error = error instanceof Error ? error.message : String(error);
        throw error;
      } finally {
        this.isLoading = false;
      }
    },

    async travelToLocation(locationId: string) {
      this.isLoading = true;
      this.error = null;
      try {
        await invokeRuntime<string>('travel_to_location', { locationId });
        const gameState = await invokeRuntime<GameState>('get_game_state', undefined);
        const plotState = await invokeRuntime<PlotState>('get_plot_state', undefined);
        this.gameState = gameState;
        this.plotState = plotState;
        this.appendGenerationDiagnostics(plotState.last_generation_diagnostics);
        await this.refreshGenerationDiagnosticsSummary();
        await this.refreshNoNameTraces();
        await this.refreshWorldRegistry();
        await this.refreshReachableLocations();
        await this.refreshMapOverview();
      } catch (error) {
        this.error = error instanceof Error ? error.message : String(error);
        throw error;
      } finally {
        this.isLoading = false;
      }
    },

    async getPlayerOptions() {
      try {
        const options = await invokeRuntime<PlayerOption[]>('get_player_options', undefined);
        if (this.plotState && this.plotState.current_scene) {
          this.plotState.current_scene.available_options = options;
        }
        return options;
      } catch (error) {
        this.error = error instanceof Error ? error.message : String(error);
        throw error;
      }
    },

    async initializeRandomGame(playerName?: string) {
      this.isLoading = true;
      this.error = null;
      this.lastInitializationDurationMs = null;
      const startedAt = Date.now();

      try {
        const script = await invokeWithTimeout<Script>(
          'generate_random_script',
          undefined,
          120000,
          '随机剧本生成超时，请稍后重试',
        );
        const trimmedName = playerName?.trim();
        script.initial_state.player_name = trimmedName || '无名弟子';
        const gameState = await invokeWithTimeout<GameState>(
          'initialize_game',
          { script },
          LLM_TIMEOUT_MS,
          '初始化世界超时，请检查 LLM 配置后重试',
        );
        this.currentScript = script;
        this.gameState = gameState;

        const plotState = await invokeWithTimeout<PlotState>(
          'initialize_plot',
          undefined,
          LLM_TIMEOUT_MS,
          '初始化剧情超时，请检查 LLM 配置后重试',
        );
        this.plotState = plotState;
        this.appendGenerationDiagnostics(plotState.last_generation_diagnostics);
        await this.refreshGenerationDiagnosticsSummary();
        await this.refreshNoNameTraces();
        this.lastInitializationDurationMs = Date.now() - startedAt;
        await this.refreshWorldRegistry();
        await this.refreshReachableLocations();
        await this.refreshMapOverview();
      } catch (error) {
        this.error = error instanceof Error ? error.message : String(error);
        throw error;
      } finally {
        this.isLoading = false;
      }
    },

    async listSaveSlots() {
      try {
        return await invokeRuntime<SaveInfo[]>('list_save_slots', undefined);
      } catch (error) {
        this.error = error instanceof Error ? error.message : String(error);
        throw error;
      }
    },

    clearError() {
      this.error = null;
    },

    clearBackgroundNotice() {
      this.backgroundNotice = null;
    },

    resetGame() {
      this.currentScript = null;
      this.gameState = null;
      this.plotState = null;
      this.lastInitializationDurationMs = null;
      this.reachableLocationIds = [];
      this.mapOverview = [];
      this.worldRegistry = null;
      this.generationDiagnostics = [];
      this.generationTimingSummary = null;
      this.generationFailureSummary = null;
      this.noNameTraces = [];
      this.backgroundNotice = null;
      this.error = null;
    },
  },
});
