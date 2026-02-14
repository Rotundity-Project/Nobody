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
  lastErrorTime: number | null;
}

export const useGameStore = defineStore('game', {
  state: (): GameStoreState => ({
    currentScript: null,
    gameState: null,
    plotState: null,
    isLoading: false,
    error: null,
    lastErrorTime: null,
  }),

  getters: {
    isGameInitialized: (state) => state.gameState !== null,
    isPlotInitialized: (state) => state.plotState !== null,
    playerCharacter: (state) => state.gameState?.player || null,
    currentScene: (state) => state.plotState?.current_scene || null,
    availableOptions: (state) => state.plotState?.current_scene.available_options || [],
    isWaitingForInput: (state) => state.plotState?.is_waiting_for_input || false,

    // 改进的错误提示getter
    errorMessage: (state) => {
      if (!state.error) return null;

      const errorLower = state.error.toLowerCase();
      if (errorLower.includes('timeout') || errorLower.includes('超时')) {
        return {
          title: '故事生成超时',
          message: 'AI正在构思精彩剧情，但响应时间较长。您可以：',
          suggestions: [
            '1. 稍等片刻后重试',
            '2. 检查网络连接是否稳定',
            '3. 在LLM设置中降低"最大输出token"数量',
          ],
        };
      }

      if (errorLower.includes('rate limit') || errorLower.includes('429')) {
        return {
          title: 'API请求过于频繁',
          message: '您的AI服务调用次数已达到限制。建议：',
          suggestions: [
            '1. 等待片刻后重试',
            '2. 检查API密钥的配额限制',
            '3. 考虑升级API服务计划',
          ],
        };
      }

      if (errorLower.includes('invalid') || errorLower.includes('invalid config')) {
        return {
          title: '配置错误',
          message: 'LLM配置有误，请检查：',
          suggestions: [
            '1. API端点地址是否正确',
            '2. API密钥是否有效',
            '3. 模型名称是否正确',
          ],
        };
      }

      return {
        title: '出错了',
        message: state.error,
        suggestions: [
          '1. 稍后重试',
          '2. 检查网络连接',
          '3. 如问题持续，请查看调试信息',
        ],
      };
    },
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
        this.handleError(error);
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

        // 检查是否有生成诊断信息
        if (plotState.last_generation_diagnostics) {
          const diagnostics = plotState.last_generation_diagnostics;
          // 不直接设置错误，而是作为提示信息
          console.log('Generation diagnostics:', diagnostics);
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

            // 改进超时错误处理
            this.handleError(
              '剧情生成超时，但系统可能已保存部分进度。建议您检查当前剧情状态，或尝试"继续写"功能。'
            );
          } catch {
            this.handleError(
              '剧情生成超时。建议您：稍后重试、检查网络连接、或降低LLM设置中的最大token数量。'
            );
          }
        } else {
          this.handleError(error);
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
        this.handleError(error);
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
        this.handleError(error);
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
        this.handleError(error);
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
      } catch (error) {
        this.handleError(error);
        throw error;
      } finally {
        this.isLoading = false;
      }
    },

    async listSaveSlots() {
      try {
        return await invoke<SaveInfo[]>('list_save_slots');
      } catch (error) {
        this.handleError(error);
        throw error;
      }
    },

    clearError() {
      this.error = null;
      this.lastErrorTime = null;
    },

    resetGame() {
      this.currentScript = null;
      this.gameState = null;
      this.plotState = null;
      this.error = null;
      this.lastErrorTime = null;
    },

    // 新增：改进的错误处理方法
    private handleError(error: unknown) {
      const message = error instanceof Error ? error.message : String(error);
      this.error = message;
      this.lastErrorTime = Date.now();

      // 也可以选择在这里添加错误日志上报
      console.error('[GameStore] Error:', error);
    },
  },
});
