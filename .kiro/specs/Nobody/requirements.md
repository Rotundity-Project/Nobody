# 需求文档 - Nobody (修仙模拟器)

## 简介

Nobody 是一个 AI 驱动的文字修仙模拟器，使用 Tauri2 + Vue3 + TailwindCSS 技术栈构建。玩家可以在严谨的修仙世界中体验独特的修仙之旅，与智能 NPC 互动，最终将自己的经历生成为小说。

## 术语表

- **System**: Nobody 修仙模拟器应用程序
- **Player**: 使用 System 进行游戏的用户
- **Script**: 定义修仙世界背景、规则和初始状态的剧本
- **NPC**: 非玩家角色，由 LLM 驱动的智能角色
- **Numerical_System**: 包含灵根、境界、功法、寿元等的数值体系
- **LLM**: 大语言模型，用于驱动 AI 功能
- **Plot**: 游戏中的剧情事件和故事发展
- **Novel**: 根据玩家游戏经历生成的小说文本
- **Mod**: 使用 Lua 实现的玩家自定义扩展模块
- **Git_Repository**: 项目的版本控制仓库

## 需求

### 需求 1: 剧本系统

**用户故事**: 作为玩家，我希望能够选择不同类型的剧本开始游戏，以便体验不同的修仙世界和故事。

#### 验收标准

1. WHEN Player 启动新游戏 THEN THE System SHALL 显示剧本选择界面
2. THE System SHALL 支持三种剧本类型：现有小说剧本、随机生成剧本、自定义剧本
3. WHEN Player 选择现有小说剧本 THEN THE System SHALL 允许 Player 导入小说文件并选择角色
4. WHEN Player 选择随机生成剧本 THEN THE System SHALL 使用 LLM 生成一个完整的修仙世界设定
5. WHEN Player 选择自定义剧本 THEN THE System SHALL 允许 Player 导入预设定的剧本文件
6. WHEN Script 被加载 THEN THE System SHALL 验证 Script 包含必要的世界设定和 Numerical_System 参数
7. WHEN Script 验证失败 THEN THE System SHALL 显示错误信息并阻止游戏开始

### 需求 2: 数值体系

**用户故事**: 作为玩家，我希望游戏基于严谨的修仙数值体系运行，以确保世界的真实性和可信度。

#### 验收标准

1. THE System SHALL 维护包含灵根、境界、功法、寿元的 Numerical_System
2. WHEN LLM 生成游戏内容 THEN THE System SHALL 约束 LLM 输出符合 Numerical_System 规则
3. THE System SHALL 为每个角色（Player 和 NPC）维护数值属性
4. WHEN 角色执行行动 THEN THE System SHALL 根据 Numerical_System 计算行动结果
5. WHEN 角色境界提升 THEN THE System SHALL 更新相关数值属性并验证合理性
6. THE System SHALL 追踪角色寿元并在寿元耗尽时触发死亡事件
7. WHEN 数值计算产生冲突 THEN THE System SHALL 使用预定义的优先级规则解决冲突

### 需求 3: 小说生成功能

**用户故事**: 作为玩家，我希望在游戏结束时能够将我的修仙经历生成为小说，以便保存和分享我的故事。

#### 验收标准

1. THE System SHALL 在游戏过程中记录所有重要的 Plot 事件和角色互动
2. WHEN Player 通关或死亡 THEN THE System SHALL 提供生成小说的选项
3. WHEN Player 选择生成小说 THEN THE System SHALL 使用 LLM 将游戏记录转换为小说格式
4. THE System SHALL 生成包含章节结构的完整 Novel 文本
5. WHEN Novel 生成完成 THEN THE System SHALL 允许 Player 导出为文本文件
6. THE System SHALL 在 Novel 中保持事件的时间顺序和因果关系
7. WHEN 生成 Novel THEN THE System SHALL 使用文学化的语言风格而非游戏日志风格

### 需求 4: 智能 NPC 系统

**用户故事**: 作为玩家，我希望与具有独立思维和行为的 NPC 互动，以获得更真实和不可预测的游戏体验。

#### 验收标准

1. THE System SHALL 为每个 NPC 维护独立的 LLM 驱动的决策系统
2. THE System SHALL 为每个 NPC 存储性格特征、记忆、人际关系数据
3. WHEN NPC 遇到事件 THEN THE System SHALL 使用 LLM 根据 NPC 的性格和记忆生成行为决策
4. WHEN NPC 与 Player 或其他 NPC 互动 THEN THE System SHALL 更新相关的人际关系数据
5. THE System SHALL 允许 NPC 自主发起行动而不仅仅响应 Player 行为
6. WHEN NPC 做出决策 THEN THE System SHALL 验证决策符合 Numerical_System 约束
7. THE System SHALL 为 NPC 维护记忆系统，记录重要事件和互动历史
8. WHEN NPC 记忆超过存储限制 THEN THE System SHALL 使用重要性算法压缩或归档旧记忆

### 需求 5: 交互式剧情

**用户故事**: 作为玩家，我希望通过文本输入或选项选择来参与剧情，并看到我的选择对故事产生真实影响。

#### 验收标准

1. THE System SHALL 使用小说风格的文本呈现 Plot 内容
2. WHEN Plot 需要 Player 决策 THEN THE System SHALL 暂停剧情并等待 Player 输入
3. THE System SHALL 提供两种输入方式：自由文本输入和预设选项选择
4. WHEN System 提供选项 THEN THE System SHALL 生成 2-5 个合理的行动选项
5. WHEN Player 输入自由文本 THEN THE System SHALL 使用 LLM 解析 Player 意图
6. WHEN Player 输入无厘头或不合理的行为 THEN THE System SHALL 拒绝该行为并提示 Player 重新输入
7. THE System SHALL 根据 Player 选择和 Numerical_System 计算 Plot 后续发展
8. WHEN Plot 发展 THEN THE System SHALL 考虑所有相关 NPC 的反应和行为

### 需求 6: Mod 支持（未来功能）

**用户故事**: 作为玩家，我希望能够使用 Lua 开发和加载 Mod，以扩展游戏功能和内容。

#### 验收标准

1. THE System SHALL 提供 Lua 脚本执行环境
2. THE System SHALL 定义 Mod API 供 Lua 脚本调用
3. WHEN Player 加载 Mod THEN THE System SHALL 验证 Mod 文件格式和安全性
4. THE System SHALL 允许 Mod 修改 Numerical_System 参数和游戏规则
5. WHEN Mod 执行错误 THEN THE System SHALL 捕获错误并防止游戏崩溃
6. THE System SHALL 提供 Mod 管理界面供 Player 启用或禁用 Mod

**注意**: 此需求在当前版本中暂不实现，仅作为未来扩展的设计考虑。

### 需求 7: 版本控制集成

**用户故事**: 作为开发者，我希望项目使用 Git 进行版本管理，以便追踪代码变更和协作开发。

#### 验收标准

1. THE System SHALL 使用 Git 作为版本控制系统
2. THE Git_Repository SHALL 托管在 https://github.com/MoSaSaPlus/Nobody
3. WHEN 开发者完成主要功能模块 THEN THE System SHALL 创建有意义的 Git 提交
4. THE System SHALL 遵循清晰的分支策略和提交信息规范
5. WHEN 代码变更影响多个模块 THEN THE System SHALL 将变更组织为逻辑相关的提交

### 需求 8: 用户界面

**用户故事**: 作为玩家，我希望使用直观美观的界面进行游戏，以获得良好的用户体验。

#### 验收标准

1. THE System SHALL 使用 Tauri2 框架构建桌面应用程序
2. THE System SHALL 使用 Vue3 实现用户界面组件
3. THE System SHALL 使用 TailwindCSS 进行样式设计
4. WHEN Player 与界面交互 THEN THE System SHALL 在 200ms 内响应用户操作
5. THE System SHALL 提供清晰的视觉反馈表明系统状态（加载中、等待输入等）
6. THE System SHALL 支持窗口大小调整并保持界面布局合理
7. WHEN 显示长文本内容 THEN THE System SHALL 提供滚动功能和良好的可读性

### 需求 9: 数据持久化

**用户故事**: 作为玩家，我希望能够保存和加载游戏进度，以便随时继续我的修仙之旅。

#### 验收标准

1. THE System SHALL 支持保存当前游戏状态到本地文件
2. THE System SHALL 支持从本地文件加载已保存的游戏状态
3. WHEN Player 保存游戏 THEN THE System SHALL 存储所有必要的游戏数据（角色状态、NPC 状态、Plot 进度、历史记录）
4. WHEN Player 加载游戏 THEN THE System SHALL 恢复完整的游戏状态
5. THE System SHALL 验证存档文件的完整性和版本兼容性
6. WHEN 存档文件损坏或不兼容 THEN THE System SHALL 显示错误信息并阻止加载
7. THE System SHALL 支持多个存档槽位

### 需求 10: LLM 集成

**用户故事**: 作为系统，我需要与 LLM 服务集成，以提供 AI 驱动的游戏内容生成和 NPC 行为。

#### 验收标准

1. THE System SHALL 支持配置 LLM API 端点和认证信息
2. WHEN System 调用 LLM THEN THE System SHALL 构造包含上下文和约束的提示词
3. THE System SHALL 处理 LLM API 调用失败和超时情况
4. WHEN LLM 响应无效或不符合格式 THEN THE System SHALL 重试或使用后备方案
5. THE System SHALL 实现 LLM 响应缓存以提高性能和降低成本
6. THE System SHALL 限制单次 LLM 调用的 token 数量以控制成本
7. WHEN LLM 生成内容 THEN THE System SHALL 验证内容符合 Numerical_System 约束
