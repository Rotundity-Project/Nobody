# Nobody 修仙模拟器 - 运行指南

## 项目简介

Nobody 是一个基于 Tauri2 + Vue3 + Rust 的修仙模拟器游戏。目前已完成 MVP 版本（v0.1.0），包含基础游戏框架、数值系统、剧本系统、存档系统和前端界面。

## 系统要求

### 必需软件

1. **Node.js** (v20.15.0 或更高版本)
   - 用于运行前端开发服务器和构建工具
   - 下载地址: https://nodejs.org/

2. **Rust** (最新稳定版)
   - 用于编译 Tauri 后端
   - 下载地址: https://www.rust-lang.org/tools/install
   - 安装命令: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

3. **Visual Studio Build Tools** (Windows)
   - Rust 在 Windows 上编译需要
   - 下载地址: https://visualstudio.microsoft.com/downloads/
   - 选择 "Desktop development with C++" 工作负载

## 安装步骤

### 1. 克隆项目

```bash
git clone https://github.com/MoSaSaPlus/Nobody.git
cd Nobody
```

### 2. 安装依赖

```bash
# 安装前端依赖
npm install
```

Rust 依赖会在首次运行时自动下载和编译。

## 运行项目

### 开发模式

在项目根目录运行以下命令启动开发服务器：

```bash
npm run tauri dev
```

这个命令会：
1. 启动 Vite 开发服务器（前端）
2. 编译 Rust 代码（后端）
3. 启动 Tauri 应用窗口

**首次运行**会比较慢（5-10分钟），因为需要下载和编译所有 Rust 依赖。后续运行会快很多。

### 生产构建

构建生产版本的可执行文件：

```bash
npm run tauri build
```

构建完成后，可执行文件位于：
- Windows: `src-tauri/target/release/nobody.exe`
- macOS: `src-tauri/target/release/bundle/macos/`
- Linux: `src-tauri/target/release/bundle/`

## 项目结构

```
Nobody/
├── src/                    # 前端源代码 (Vue3 + TypeScript)
│   ├── components/         # Vue 组件
│   │   ├── MainMenu.vue           # 主菜单
│   │   ├── ScriptSelector.vue     # 剧本选择器
│   │   ├── GameView.vue           # 游戏主界面
│   │   ├── CharacterPanel.vue     # 角色信息面板
│   │   └── SaveLoadDialog.vue     # 存档管理对话框
│   ├── stores/             # Pinia 状态管理
│   │   └── gameStore.ts           # 游戏状态 Store
│   ├── types/              # TypeScript 类型定义
│   │   └── game.ts                # 游戏类型
│   ├── router/             # Vue Router 路由
│   │   └── index.ts
│   ├── App.vue             # 根组件
│   └── main.ts             # 入口文件
│
├── src-tauri/              # 后端源代码 (Rust)
│   ├── src/
│   │   ├── models.rs              # 数据模型
│   │   ├── numerical_system.rs    # 数值系统
│   │   ├── script.rs              # 剧本定义
│   │   ├── script_manager.rs      # 剧本管理器
│   │   ├── game_state.rs          # 游戏状态
│   │   ├── game_engine.rs         # 游戏引擎
│   │   ├── plot_engine.rs         # 剧情引擎
│   │   ├── save_load.rs           # 存档系统
│   │   ├── tauri_commands.rs      # Tauri 命令接口
│   │   ├── lib.rs                 # 库入口
│   │   └── main.rs                # 应用入口
│   ├── Cargo.toml          # Rust 依赖配置
│   └── tauri.conf.json     # Tauri 配置
│
├── package.json            # Node.js 依赖配置
├── vite.config.ts          # Vite 配置
├── tailwind.config.js      # TailwindCSS 配置
└── tsconfig.json           # TypeScript 配置
```

## 当前功能

### 已实现 (MVP v0.1.0)

? **基础框架**
- Tauri2 + Vue3 + TypeScript 项目结构
- TailwindCSS 样式系统
- Vue Router 路由管理
- Pinia 状态管理

? **数值系统**
- 角色属性系统（灵根、境界、寿元等）
- 修仙境界系统
- 战斗力计算
- 属性测试覆盖

? **剧本系统**
- 自定义剧本加载（JSON 格式）
- 剧本验证
- 世界设定管理

? **游戏引擎**
- 游戏状态管理
- 存档系统（多槽位支持）
- 基础剧情引擎
- 选项式交互

? **前端界面**
- 主菜单
- 剧本选择器
- 游戏主界面
- 角色信息面板
- 存档管理对话框

### 待实现

? **第二阶段** (计划中)
- LLM 集成
- 智能 NPC 系统
- 随机剧本生成
- 自由文本输入

? **第三阶段** (计划中)
- 小说生成功能
- 现有小说导入
- UI 优化和美化
- 性能优化

## 开发命令

```bash
# 安装依赖
npm install

# 启动开发服务器
npm run tauri dev

# 构建前端
npm run build

# 构建生产版本
npm run tauri build

# 运行前端 linter
npm run lint

# 格式化代码
npm run format

# 运行 Rust 测试
cd src-tauri
cargo test --lib

# 检查 Rust 代码
cd src-tauri
cargo check --lib
```

## 常见问题

### Q: 首次运行很慢怎么办？

A: 首次运行需要下载和编译所有 Rust 依赖，这是正常的。请耐心等待 5-10 分钟。后续运行会快很多。

### Q: 编译失败怎么办？

A: 确保已安装所有必需软件：
1. Node.js (v20.15.0+)
2. Rust (最新稳定版)
3. Visual Studio Build Tools (Windows)

### Q: 如何创建自定义剧本？

A: 目前需要手动创建 JSON 格式的剧本文件。剧本格式示例将在后续版本中提供。

### Q: 游戏数据保存在哪里？

A: 存档文件保存在用户数据目录：
- Windows: `%APPDATA%/nobody/saves/`
- macOS: `~/Library/Application Support/nobody/saves/`
- Linux: `~/.local/share/nobody/saves/`

## 技术栈

- **前端**: Vue 3, TypeScript, TailwindCSS, Pinia, Vue Router
- **后端**: Rust, Tauri 2
- **构建工具**: Vite, Cargo
- **测试**: Vitest (前端), proptest (后端)

## 贡献

欢迎提交 Issue 和 Pull Request！

## 许可证

[待定]

## 联系方式

- GitHub: https://github.com/MoSaSaPlus/Nobody
- Issues: https://github.com/MoSaSaPlus/Nobody/issues

---

**注意**: 本项目目前处于 MVP 阶段，许多功能仍在开发中。
