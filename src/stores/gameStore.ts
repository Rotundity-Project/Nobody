import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { invokeWithTimeout } from '../utils/tauriInvoke';
import type {
  Script,
  GameState,
  PlotState,
  PlayerAction,
  PlayerOption,
  SaveInfo,
} from '../types/game';

interface GameStoreState {
  currentScript: Script | null;
  gameState: GameState | null;
  plotState: PlotState | null;
  isLoading: boolean;
  error: string | null;
}

export const useGameStore = defineStore('game', {
  state: (): GameStoreState => ({
    currentScript: null,
    gameState: null,
    plotState: null,
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
        await invoke<string>('execute_player_action', { action });
        
        const gameState = await invoke<GameState>('get_game_state');
        this.gameState = gameState;
        
        const plotState = await invoke<PlotState>('get_plot_state');
        this.plotState = plotState;
      } catch (error) {
        this.error = error instanceof Error ? error.message : String(error);
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
          45000,
          '随机剧本生成超时，请稍后重试',
        );
        const trimmedName = playerName?.trim();
        script.initial_state.player_name = trimmedName || '无名弟子';
        const gameState = await invoke<GameState>('initialize_game', { script });
        this.currentScript = script;
        this.gameState = gameState;

        const plotState = await invoke<PlotState>('initialize_plot');
        this.plotState = plotState;
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
      this.error = null;
    },
  },
});
