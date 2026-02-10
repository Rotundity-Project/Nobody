# Nobody - 修仙模拟器

Nobody 是一个 AI 驱动的文字修仙模拟器，使用 Tauri2 + Vue3 + TailwindCSS 技术栈构建。

## 技术栈

- **前端**: Vue3 (Composition API) + TailwindCSS + TypeScript
- **后端**: Tauri2 (Rust)
- **状态管理**: Pinia
- **AI 集成**: LLM API
- **版本控制**: Git

## 开发环境设置

### 前置要求

- Node.js 20+
- Rust 1.70+
- npm 或 yarn

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri:dev
```

### 构建生产版本

```bash
npm run tauri:build
```

## 项目结构

```
Nobody/
├── src/                  # Vue3 前端代码
│   ├── components/       # Vue 组件
│   ├── stores/          # Pinia 状态管理
│   ├── router/          # Vue Router 路由
│   └── types/           # TypeScript 类型定义
├── src-tauri/           # Rust 后端代码
│   └── src/
│       ├── models/      # 数据模型
│       └── services/    # 业务逻辑
└── .kiro/specs/         # 项目规范文档
```

## 代码规范

- 前端使用 ESLint + Prettier
- 后端使用 Clippy + Rustfmt

运行代码检查：
```bash
npm run lint
npm run format
```

## License

MIT

## 仓库

https://github.com/MoSaSaPlus/Nobody
