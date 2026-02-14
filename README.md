# Nobody - 文字修仙游戏

Nobody 是一个 AI 驱动的沉浸式文字修仙游戏，使用 Tauri2 + Vue3 + TailwindCSS 构建跨平台桌面应用。

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)
![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)
![Vue](https://img.shields.io/badge/Vue-3.5%2B-4FC08D.svg)

## ✨ 特性

- 🎮 **沉浸式文字冒险** - AI 生成独一无二的修仙剧情，每次体验都不同
- 🧠 **智能剧情引擎** - 基于大语言模型，理解上下文，生成连贯的故事
- 💾 **多存档系统** - 随时保存和加载游戏进度
- 🎨 **精美视觉设计** - 现代化 UI，毛玻璃效果，流畅动画
- ⚡ **高性能** - Rust 后端 + Vue3 前端，极致性能体验
- 📦 **跨平台** - 支持 Windows、macOS 和 Linux
- 🔊 **音效系统** - 沉浸式音效，增强游戏氛围
- 📝 **小说导出** - 将游戏剧情导出为小说文件

## 🛠️ 技术栈

### 前端
- **Vue3** (Composition API) - 现代化前端框架
- **TypeScript** - 类型安全
- **TailwindCSS** - 实用优先的 CSS 框架
- **Pinia** - 状态管理
- **Vite** - 极速构建工具
- **Vitest** - 单元测试框架

### 后端
- **Rust 1.70+** - 高性能系统编程语言
- **Tauri2** - 跨平台桌面应用框架
- **Tokio** - 异步运行时
- **Reqwest** - HTTP 客户端
- **Serde** - 序列化/反序列化

## 📋 系统要求

- **Node.js** 20+ (用于开发)
- **Rust** 1.70+ (用于开发和构建)
- **npm** 或 **yarn** (包管理器)
- **操作系统**: Windows 10+, macOS 10.15+, Linux (主流发行版)

## 🚀 快速开始

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri:dev
```

这将启动 Tauri 开发服务器，并自动编译 Rust 代码和热重载前端代码。

### 构建生产版本

```bash
npm run tauri:build
```

构建产物将位于 `src-tauri/target/release/bundle/` 目录下。

### 运行测试

```bash
npm run test
```

### 代码检查和格式化

```bash
npm run lint      # ESLint 检查
npm run format    # Prettier 格式化
```

## 📁 项目结构

```
Nobody/
├── src/                      # Vue3 前端源码
│   ├── components/           # Vue 组件
│   │   ├── AudioControlPanel.vue      # 音频控制面板
│   │   ├── CharacterPanel.vue         # 角色信息面板
│   │   ├── GameView.vue               # 主游戏界面
│   │   ├── LLMConfigDialog.vue        # LLM 配置对话框
│   │   ├── MainMenu.vue               # 主菜单
│   │   ├── NovelExporter.vue          # 小说导出器
│   │   ├── SaveLoadDialog.vue         # 存取档对话框
│   │   └── ScriptSelector.vue         # 剧本选择器
│   ├── stores/              # Pinia 状态管理
│   │   └── gameStore.ts              # 游戏状态管理
│   ├── router/              # Vue Router 路由
│   ├── types/               # TypeScript 类型定义
│   └── utils/               # 工具函数
├── src-tauri/              # Rust 后端源码
│   └── src/
│       ├── app_error.rs           # 错误处理
│       ├── event_log.rs            # 事件日志
│       ├── game_engine.rs          # 游戏引擎核心
│       ├── game_state.rs           # 游戏状态
│       ├── llm_service.rs           # LLM 服务
│       ├── llm_runtime_config.rs    # LLM 运行时配置
│       ├── memory_manager.rs        # 内存管理
│       ├── models.rs               # 数据模型
│       ├── novel_generator.rs       # 小说生成器
│       ├── novel_parser.rs          # 小说解析器
│       ├── npc.rs                   # NPC 定义
│       ├── npc_engine.rs            # NPC 引擎
│       ├── numerical_system.rs      # 数值系统
│       ├── plot_engine.rs           # 剧情引擎
│       ├── prompt_builder.rs        # Prompt 构建
│       ├── response_validator.rs   # 响应验证
│       ├── save_load.rs             # 存取档系统
│       ├── script.rs                # 脚本系统
│       ├── script_manager.rs        # 脚本管理器
│       ├── tauri_commands.rs        # Tauri 命令
│       └── lib.rs                   # 库入口
├── docs/                  # 项目文档
├── example_scripts/      # 示例脚本
├── package.json          # Node.js 依赖配置
├── tauri.conf.json       # Tauri 配置
└── README.md             # 项目说明
```

## 🎮 使用指南

### 1. 配置 LLM

首次使用需要配置 LLM 服务：

1. 点击游戏界面上的 "LLM 设置" 按钮
2. 填写您的 LLM API 配置：
   - **API 端点**: 例如 `https://api.openai.com/v1/chat/completions`
   - **API 密钥**: 您的 API 密钥
   - **模型**: 例如 `gpt-4` 或 `gpt-3.5-turbo`
   - **最大输出 Token**: 控制每次生成的文本长度（建议 500-2000）
   - **温度**: 控制生成的随机性（0.0-2.0，建议 0.7-1.0）

### 2. 开始游戏

1. 在主菜单选择 "开始新游戏"
2. 输入您的角色名称
3. 选择剧本（随机生成或手动选择）
4. 开始您的修仙之旅！

### 3. 游戏操作

- **选项模式**: 从预设选项中选择行动
- **自由输入**: 自由输入您想执行的动作
- **继续写**: 让 AI 自动推进剧情
- **保存/加载**: 随时保存和加载游戏进度

### 4. 常见问题

#### Q: 游戏生成超时怎么办？

A: 尝试以下解决方案：
1. 检查网络连接是否稳定
2. 在 LLM 设置中降低 "最大输出 Token" 数量
3. 检查 API 服务是否有速率限制
4. 稍等片刻后重试

#### Q: 生成的剧情质量不佳怎么办？

A: 可以尝试：
1. 调整 LLM 温度参数（0.7-1.0 之间）
2. 在提示词中给出更具体的引导
3. 使用更强大的 LLM 模型
4. 通过自由输入来纠正剧情方向

#### Q: 如何导出游戏剧情为小说？

A: 点击界面底部的 "导出小说" 按钮，选择导出格式和位置即可。

## 🤝 贡献指南

欢迎贡献代码、报告问题或提出建议！

### 开发流程

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 创建 Pull Request

### 代码规范

- 前端遵循 ESLint + Prettier 规范
- 后端遵循 Clippy + Rustfmt 规范
- 提交前请运行 `npm run lint` 确保代码质量

### 文档

- 新功能需要添加相应的文档
- API 变更需要更新 CHANGELOG.md
- 重要改动需要提交 Issue 讨论

## 📝 更新日志

查看 [CHANGELOG.md](./CHANGELOG.md) 了解版本更新历史。

## 📄 许可证

本项目采用 MIT 许可证。详见 [LICENSE](./LICENSE) 文件。

## 🙏 致谢

- [Tauri](https://tauri.app/) - 跨平台桌面应用框架
- [Vue.js](https://vuejs.org/) - 渐进式 JavaScript 框架
- [TailwindCSS](https://tailwindcss.com/) - 实用优先的 CSS 框架
- [Rust](https://www.rust-lang.org/) - 高性能系统编程语言

## 🔗 相关链接

- [GitHub 仓库](https://github.com/Rotundity-Project/Nobody)
- [问题反馈](https://github.com/Rotundity-Project/Nobody/issues)
- [原始项目](https://github.com/MoSaSaPlus/Nobody)

---

**Enjoy your cultivation journey! 🌟**
