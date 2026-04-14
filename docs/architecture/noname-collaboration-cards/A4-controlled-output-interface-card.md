# 任务卡片 A4

任务名: 受控高一层输出接口预研
状态: 未开始
建议优先级: 中高

## 目标

为 `NoName Agent` 后续进入“比当前低风险输出层更高一层”的受控应用做接口预研，但不直接改写主剧情状态机。

## 建议范围

- 定义允许进入下一层受控 apply 的输出边界
- 定义明确禁止触碰的输出边界
- 给出最小接口草案或模块草案

## 建议涉及文件

- `src-tauri/src/plot_engine.rs`
- `docs/architecture/` 下补充一份接口草案文档

## 不要碰的文件

- `src-tauri/src/tauri_commands.rs`
- `src-tauri/src/noname_runtime.rs`
- `src-tauri/src/noname_guardrails.rs`

## 交付标准

- 列出下一层允许受控 apply 的输出
- 明确不可碰范围
- 给出最小接口草案
- 给出至少 2 条测试草案或验证场景

## 验证命令

```powershell
cargo test plot_engine -- --nocapture
```

## 备注

- 这是接口预研任务，不应直接接手当前 `T7` apply 主线
