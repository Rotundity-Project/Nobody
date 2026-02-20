# Nobody 运行指南

## 1. 环境要求

- Node.js 20+
- Rust 稳定版工具链
- 支持 Tauri 的桌面环境（Windows/macOS/Linux）

## 2. 安装依赖

```bash
npm install
```

## 3. 本地开发

```bash
npm run tauri:dev
```

## 4. 构建发布版本

```bash
npm run build
npm run tauri:build
```

产物位置（按平台不同）：

- Windows: `src-tauri/target/release/`
- macOS: `src-tauri/target/release/bundle/macos/`
- Linux: `src-tauri/target/release/bundle/`

## 5. 常见排查

### 5.1 依赖安装失败

- 检查 Node/Rust 版本
- 删除 `node_modules` 后重装

### 5.2 Tauri 启动失败

- 确认系统依赖完整
- 查看终端报错并按提示补依赖

### 5.3 前端构建失败

- 先执行 `npm run test`
- 再执行 `npm run build`

### 5.4 文档乱码

执行：

```bash
npm run specs:fix-encoding
npm run specs:check-encoding
```
