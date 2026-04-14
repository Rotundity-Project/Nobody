import { computed, type Ref } from 'vue';
import { type useGameStore } from '../stores/gameStore';

export const useRuntimeInteractionState = ({
  gameStore,
  isDevMode,
  showSaveDialog,
  showLoadDialog,
  showLLMDialog,
  showStorySettings,
  showConsistencySettings,
  showInfoTabs,
  showCharacterInfo,
  showShortcutsDialog,
  showQuickPanel,
}: {
  gameStore: ReturnType<typeof useGameStore>;
  isDevMode: boolean;
  showSaveDialog: Ref<boolean>;
  showLoadDialog: Ref<boolean>;
  showLLMDialog: Ref<boolean>;
  showStorySettings: Ref<boolean>;
  showConsistencySettings: Ref<boolean>;
  showInfoTabs: Ref<boolean>;
  showCharacterInfo: Ref<boolean>;
  showShortcutsDialog: Ref<boolean>;
  showQuickPanel: Ref<boolean>;
}) => {
  const plotInteractionState = computed(() => {
    if (gameStore.plotState?.interaction_state) {
      return gameStore.plotState.interaction_state;
    }
    if (!gameStore.isGameInitialized) {
      return 'auto_advance';
    }
    if (!gameStore.isWaitingForInput) {
      return 'auto_advance';
    }
    return gameStore.availableOptions.length > 0 ? 'waiting_for_choice' : 'waiting_for_free_text';
  });

  const isNoInputAdvanceState = computed(
    () =>
      gameStore.isGameInitialized &&
      (
        plotInteractionState.value === 'cooldown' ||
        gameStore.plotState?.last_option_generation_source === 'not_waiting_for_input' ||
        gameStore.plotState?.last_option_generation_source === 'consistency_non_waiting_fallback'
      ),
  );

  const shouldShowInputPanel = computed(
    () =>
      gameStore.isGameInitialized &&
      !isNoInputAdvanceState.value &&
      (plotInteractionState.value === 'waiting_for_choice'
        || plotInteractionState.value === 'waiting_for_free_text'),
  );

  const shouldAutoAdvance = computed(
    () =>
      gameStore.isGameInitialized &&
      (!gameStore.isWaitingForInput
        || plotInteractionState.value === 'cooldown'
        || isNoInputAdvanceState.value),
  );

  const hasBlockingOverlay = computed(
    () =>
      showSaveDialog.value
      || showLoadDialog.value
      || showLLMDialog.value
      || showStorySettings.value
      || showConsistencySettings.value
      || showInfoTabs.value
      || showCharacterInfo.value
      || showShortcutsDialog.value
      || showQuickPanel.value,
  );

  const shouldAutoFollowNewParagraph = computed(
    () => shouldAutoAdvance.value && !hasBlockingOverlay.value,
  );

  const optionSourceLabel = computed(() => {
    const source = gameStore.plotState?.last_option_generation_source;
    if (!source) {
      return '';
    }
    const labels: Record<string, string> = {
      llm_structured: '模型结构化',
      llm_regenerated: '模型再生成',
      rule_fallback: '规则回退',
      rule_fallback_latency_budget: '规则回退（时延预算）',
      previous_reused: '复用上一组选项',
      not_waiting_for_input: '当前无需输入',
      consistency_non_waiting_fallback: '一致性兜底自动推进',
    };
    return labels[source] ?? '未知来源';
  });

  const optionSourceHint = computed(() => {
    const source = gameStore.plotState?.last_option_generation_source ?? '';
    const diag = gameStore.plotState?.last_generation_diagnostics ?? '';
    const noNameBiasMatch = diag.match(/NoName选项偏置：([^；\n]+)/);
    if (noNameBiasMatch) {
      return noNameBiasMatch[0];
    }
    if (source === 'rule_fallback_latency_budget') {
      return '受时延预算影响，已跳过模型选项再生成';
    }
    if (diag.includes('skipped(latency_budget)')) {
      return '部分增强步骤因时延预算被跳过';
    }
    return '';
  });

  const userFacingError = computed(() => {
    const raw = gameStore.error ?? '';
    if (!raw) {
      return null;
    }
    if (isDevMode) {
      return raw;
    }
    const isVerboseDiagnostics = raw.includes('；选项来源：')
      || raw.includes('；耗时(ms)：')
      || raw.includes('回退：')
      || raw.includes('已降级为骨架生成')
      || raw.includes('双通道生成：');
    if (isVerboseDiagnostics) {
      return '本轮剧情生成质量不稳定，建议检查 LLM 设置后重试。';
    }
    return raw;
  });

  const shouldShowLlmSetupShortcut = computed(() => {
    const err = userFacingError.value ?? '';
    return err.includes('选项续写未获取到 LLM 剧情文本');
  });

  const interactionStateLabel = computed(() => {
    const mapping: Record<string, string> = {
      auto_advance: '自动推进',
      waiting_for_choice: '等待选项',
      waiting_for_free_text: '等待自由输入',
      resolving: '处理中',
      cooldown: '冷却阶段',
    };
    return mapping[plotInteractionState.value] ?? plotInteractionState.value;
  });

  const interactionStateToneClass = computed(() => {
    if (plotInteractionState.value === 'resolving') return 'runtime-state-gold';
    if (plotInteractionState.value === 'waiting_for_free_text') return 'runtime-state-ink';
    if (plotInteractionState.value === 'cooldown') return 'runtime-state-ember';
    return 'runtime-state-cool';
  });

  const consistencyRiskScore = computed(() => {
    const structured = gameStore.plotState?.last_consistency_risk_score;
    if (typeof structured === 'number') {
      return structured;
    }
    const diag = gameStore.plotState?.last_generation_diagnostics ?? '';
    const matched = diag.match(/风险分[=:：]\s*(\d+)/);
    if (!matched) {
      return null;
    }
    const value = Number(matched[1]);
    return Number.isFinite(value) ? value : null;
  });

  return {
    plotInteractionState,
    isNoInputAdvanceState,
    shouldShowInputPanel,
    shouldAutoAdvance,
    shouldAutoFollowNewParagraph,
    hasBlockingOverlay,
    optionSourceLabel,
    optionSourceHint,
    userFacingError,
    shouldShowLlmSetupShortcut,
    interactionStateLabel,
    interactionStateToneClass,
    consistencyRiskScore,
  };
};
