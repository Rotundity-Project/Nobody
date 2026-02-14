# 贡献指南

感谢你对 Nobody 项目的关注！我们欢迎任何形式的贡献。

## 如何贡献

### 报告 Bug

如果你发现了 Bug，请先在 Issues 中搜索，看看是否已经有人报告过。如果没有，请创建一个新的 Issue，并按照 Bug 报告模板填写。

### 提出新功能

如果你有新功能的想法，请先在 Issues 中讨论，确保这个功能符合项目的发展方向。

### 提交代码

1. Fork 项目
2. 创建你的特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交你的更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启一个 Pull Request

## 开发环境设置

### 前端

```bash
npm install
npm run dev
```

### 后端

项目使用 Rust 开发后端。确保你安装了 Rust 1.70 或更高版本。

```bash
cd src-tauri
cargo build
```

## 代码规范

### 前端

- 使用 ESLint 和 Prettier 进行代码格式化
- 运行 `npm run lint` 检查代码质量
- 遵循 Vue 3 组合式 API 风格

### 后端

- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 编写单元测试

## 提交信息

提交信息应该清晰描述你的更改：

```
<type>(<scope>): <subject>

<body>
```

Type 可以是：
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式（不影响代码运行的变动）
- `refactor`: 重构（既不是新增功能，也不是修改 Bug 的代码变动）
- `perf`: 性能优化
- `test`: 增加测试
- `chore`: 构建过程或辅助工具的变动

示例：

```
feat(llm): add retry mechanism for failed LLM requests

- Add exponential backoff
- Increase max retries from 2 to 3
- Add timeout configuration
```

## 测试

在提交 PR 之前，请确保：

- [ ] 所有现有测试通过
- [ ] 新功能包含相应的测试
- [ ] 代码符合项目规范

## 文档

如果你更改了 API 或添加了新功能，请更新相关文档。

## 获取帮助

如果你有任何问题，请随时在 Issues 中提问或联系项目维护者。
