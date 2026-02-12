# Nobody 用户手册

## 1. 项目简介

Nobody 是基于 Tauri + Vue 的修仙文字模拟器。你可以通过以下入口开始游戏：

- 加载自定义 JSON 剧本
- AI 随机生成剧本
- 从现有小说导入（先选文本，再选角色）

## 2. 环境要求

- Node.js 20+
- Rust 稳定版工具链
- 可运行 Tauri 的桌面系统（Windows/macOS/Linux）

## 3. 安装与运行

```bash
npm install
npm run tauri:dev
```

生产构建：

```bash
npm run tauri:build
```

## 4. 基本流程

1. 打开应用，点击 `New Game`。
2. 选择剧本来源：`Custom` / `Random Generated` / `Existing Novel`。
3. 进入游戏界面后，通过选项或自由输入推进剧情。
4. 可使用存档槽保存和读取进度。
5. 达到生成条件后可导出小说文本。

## 5. 主要界面

- Main Menu：新游戏、加载、设置
- Script Selector：剧本来源选择与导入
- Game View：剧情文本、选项与输入
- Character Panel：姓名、境界、灵根、寿元、战力
- Save/Load Dialog：存档槽操作
- Novel Exporter：小说生成与导出

## 6. 存档说明

- 存档使用 `slot_id` 编号管理
- 推荐范围：`1..99`
- 读档后会恢复游戏状态与剧情状态

## 7. LLM 功能说明

配置有效的 LLM 后，可用于：

- 随机剧本生成
- 剧情文本生成
- NPC 决策
- 自由文本意图解析
- 小说生成

若 LLM 不可用，部分功能会回退到本地兜底逻辑。

## 8. 常见问题

- 剧本加载失败：
  - 检查 JSON 结构是否合法
  - 确认 `initial_state.starting_location` 存在于 `world_setting.locations`
- 小说解析失败：
  - 使用 `.txt` 或 `.md` 文件
  - 确认文本中可提取角色名
- 请求超时：
  - 检查 LLM endpoint、model、api key
  - 稍后重试，或降低任务复杂度

## 9. 相关文档

- `docs/SCRIPT_GUIDE.md`
- `docs/API.md`
- `CONTRIBUTING.md`
