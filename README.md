# Nobody - 修仙文字模拟器

Nobody 是一个 AI 驱动的修仙题材文字模拟器，技术栈为 `Tauri 2 + Vue 3 + TypeScript + Tailwind CSS`。

## 功能概览

- 主循环：剧情阅读 -> 选项/自由输入 -> 结果反馈
- 剧本系统：支持内置剧本与导入
- 世界信息：地图、关系、战斗复盘、导出
- 系统中心：LLM 配置、剧情策略、一致性策略
- 存档系统：多槽位保存/加载

## 快速开始

### 前置要求

- Node.js 20+
- Rust 1.70+
- npm

### 安装依赖

```bash
npm install
```

### 本地开发

```bash
npm run tauri:dev
```

### 构建

```bash
npm run build
npm run tauri:build
```

## 项目结构

```text
src/                    # 前端源码
src/components/         # 页面与领域组件
src/composables/        # 视图编排逻辑
src/shared/ui/          # 共享 UI 基础组件
src/stores/             # Pinia 状态管理
src-tauri/src/          # Rust 后端核心逻辑
docs/                   # 架构、UI、测试与发布文档
release/screenshots/    # 界面截图清单与产物目录
```

## 文档索引

- 架构：`docs/ARCHITECTURE.md`
- UI 重构指南：`UI_REDESIGN_GUIDE.md`
- 用户手册：`docs/USER_MANUAL.md`
- 测试指南：`测试指南.md`
- 运行说明：`RUNNING.md`

## 界面截图

截图规范与命名见：`release/screenshots/README.md`

## 开发规范

- 前端：`ESLint + Prettier`
- 后端：`clippy + rustfmt`
- 构建检查：

```bash
npm run test
npm run build
```

## License

MIT
