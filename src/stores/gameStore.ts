import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { invokeWithTimeout } from '../utils/tauriInvoke';
import type {
  Script,
  GameState,
  PlotState,
  PlayerAction,
  PlayerOption,
  MapLocationOverview,
  SaveInfo,
  WorldRegistry,
} from '../types/game';

interface GameStoreState {
  currentScript: Script | null;
  gameState: GameState | null;
  plotState: PlotState | null;
  reachableLocationIds: string[];
  mapOverview: MapLocationOverview[];
  worldRegistry: WorldRegistry | null;
  isLoading: boolean;
  error: string | null;
}

export const useGameStore = defineStore('game', {
  state: (): GameStoreState => ({
    currentScript: null,
    gameState: null,
    plotState: null,
    reachableLocationIds: [],
    mapOverview: [],
    worldRegistry: null,
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
    async refreshReachableLocations() {
      try {
        const ids = await invoke<unknown>('get_reachable_locations');
        this.reachableLocationIds = Array.isArray(ids)
          ? ids.filter((id): id is string => typeof id === 'string')
          : [];
      } catch {
        this.reachableLocationIds = [];
      }
    },

    async refreshMapOverview() {
      try {
        const nodes = await invoke<MapLocationOverview[]>('get_map_overview');
        this.mapOverview = Array.isArray(nodes) ? nodes : [];
      } catch {
        this.mapOverview = [];
      }
    },

    async refreshWorldRegistry() {
      try {
        this.worldRegistry = await invoke<WorldRegistry>('get_world_registry');
      } catch {
        this.worldRegistry = null;
      }
    },

    async applyWorldRegistryPatch(patch: unknown) {
      this.isLoading = true;
      this.error = null;
      try {
        this.worldRegistry = await invoke<WorldRegistry>('apply_world_registry_patch', { patch });
        const gameState = await invoke<GameState>('get_game_state');
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

      try {
        const trimmedName = playerName?.trim();
        script.initial_state.player_name = trimmedName || '无名弟子';
        const gameState = await invoke<GameState>('initialize_game', { script });
        this.currentScript = script;
        this.gameState = gameState;

        const plotState = await invoke<PlotState>('initialize_plot');
        this.plotState = plotState;
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
          140000,
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
        await this.refreshWorldRegistry();
        await this.refreshReachableLocations();
        await this.refreshMapOverview();

        // 显示 LLM 诊断信息（如果有）
        if (plotState.last_generation_diagnostics) {
          console.warn('LLM 诊断信息:', plotState.last_generation_diagnostics);
          this.error = plotState.last_generation_diagnostics;
        }
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);

        if (message.includes('剧情推进超时')) {
          try {
            const latestPlotState = await invokeWithTimeout<PlotState>(
              'get_plot_state',
              undefined,
              10000,
              '获取剧情状态超时，请重试',
            );
            this.plotState = latestPlotState;
            this.error = latestPlotState.last_generation_diagnostics ?? '剧情推进超时，请稍后重试';
          } catch {
            this.error = '剧情推进超时，请稍后重试。你可以尝试重连或调整 LLM 设置。';
          }
        } else {
          this.error = `操作失败: ${message}`;
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
        await invoke('save_game', { slotId });
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
        const gameState = await invoke<GameState>('load_game', { slotId });
        this.gameState = gameState;

        const plotState = await invoke<PlotState>('get_plot_state');
        this.plotState = plotState;
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
        await invoke<string>('travel_to_location', { locationId });
        const gameState = await invoke<GameState>('get_game_state');
        const plotState = await invoke<PlotState>('get_plot_state');
        this.gameState = gameState;
        this.plotState = plotState;
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
        const options = await invoke<PlayerOption[]>('get_player_options');
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

      try {
        const script = await invokeWithTimeout<Script>(
          'generate_random_script',
          undefined,
          120000,
          '随机剧本生成超时，请稍后重试',
        );
        const trimmedName = playerName?.trim();
        script.initial_state.player_name = trimmedName || '无名弟子';
        const gameState = await invoke<GameState>('initialize_game', { script });
        this.currentScript = script;
        this.gameState = gameState;

        const plotState = await invoke<PlotState>('initialize_plot');
        this.plotState = plotState;
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
        return await invoke<SaveInfo[]>('list_save_slots');
      } catch (error) {
        this.error = error instanceof Error ? error.message : String(error);
        throw error;
      }
    },

    clearError() {
      this.error = null;
    },

    resetGame() {
      this.currentScript = null;
      this.gameState = null;
      this.plotState = null;
      this.reachableLocationIds = [];
      this.mapOverview = [];
      this.worldRegistry = null;
      this.error = null;
    },
  },
});


