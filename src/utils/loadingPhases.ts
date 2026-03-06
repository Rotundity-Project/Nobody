import { getLlmProviderLabel, resolveLlmProviderKey } from './llmProvider';

export type LoadingPhase = {
  stage: string;
  progress: number;
  text: string;
};

export const buildStoryLoadingPhases = (model: string | null | undefined): LoadingPhase[] => {
  const provider = getLlmProviderLabel(resolveLlmProviderKey(model));
  return [
    { stage: '上下文构建', progress: 14, text: '正在准备剧情上下文...' },
    { stage: '模型请求', progress: 34, text: `正在连接 ${provider}...` },
    { stage: '模型生成', progress: 58, text: `${provider} 正在生成剧情，请稍候...` },
    { stage: '一致性校验', progress: 78, text: '正在执行一致性校验...' },
    { stage: '状态落盘', progress: 92, text: '正在同步世界状态...' },
  ];
};
