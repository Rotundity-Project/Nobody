# Nobody 用户手册（V3）

## 1. 简介
Nobody 是一个 AI 驱动的修仙题材文字模拟器。核心体验是：

1. 阅读剧情
2. 做出选择（或自由输入）
3. 观察结果与角色成长

## 2. 环境要求

- Node.js 20+
- Rust 稳定版工具链
- 支持 Tauri 的桌面环境（Windows/macOS/Linux）

## 3. 安装与运行

```bash
npm install
npm run tauri:dev
```

生产构建：

```bash
npm run build
npm run tauri:build
```

## 4. 开始游戏

1. 打开应用后进入主菜单。
2. 点击“新游戏”，进入剧本选择。
3. 选择剧本来源：
   - 自定义 JSON
   - AI 随机生成
   - 现有小说导入
4. 开始后进入游戏主界面，推进剧情。

## 5. 主界面说明（V3）

- 主舞台层：剧情阅读 + 交互输入
- 世界层抽屉：地图行程 / 战斗复盘 / 导出
- 系统层入口：LLM 设置、剧情策略、一致性策略

### 5.1 交互方式

- 选项模式：点击选项推进剧情
- 自由输入模式：输入你的行动意图
- 自动推进：在无需输入时自动继续，可中断

### 5.2 常用操作

- `Esc`：关闭当前弹层
- `Ctrl/Cmd + S`：打开保存对话框
- `Tab`：键盘焦点切换

## 6. 存档与读档

- 支持多槽位存档。
- 可在主菜单“最近存档”继续，或在游戏内打开保存/读取对话框。

## 7. LLM 功能

配置有效的 LLM 后，可用于：

- 剧情文本生成
- 随机剧本生成
- 选项生成增强
- 自由输入意图解析

若 LLM 不可用，系统会回退到规则生成路径，保证可玩。

## 8. 常见问题

### 8.1 剧本加载失败

- 检查 JSON 结构是否合法。
- 确认 `initial_state.starting_location` 存在于 `world_setting.locations`。

### 8.2 小说导入失败

- 使用 `.txt` 或 `.md` 文件。
- 确认文本中可提取角色名。

### 8.3 请求超时

- 检查 LLM endpoint / model / api key。
- 稍后重试，或降低任务复杂度。

## 9. 相关文档

- `docs/ARCHITECTURE.md`
- `docs/API.md`
- `UI_REDESIGN_GUIDE.md`
- `release/screenshots/README.md`
