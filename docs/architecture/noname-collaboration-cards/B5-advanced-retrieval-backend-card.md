# 任务卡片 B5

任务名: 外部知识与高级检索后端
状态: 未开始
建议优先级: 低到中

## 目标

为未来的 lore / wiki / 设定集 / 向量检索 / 图检索做后端预研与接口草案。

## 建议范围

- 评估向量检索后端
- 评估图检索后端
- 评估外部 lore RAG 的接入方式
- 形成接口建议，不要求立即落地生产依赖

## 建议涉及文件

- `docs/architecture/` 下新增技术评估文档
- 如需代码，可新增实验性模块，但应与主线隔离

## 不要碰的文件

- `src-tauri/src/tauri_commands.rs`
- `src-tauri/src/noname_runtime.rs`
- `src-tauri/src/noname_guardrails.rs`
- 当前所有 `T7` apply 主线文件

## 交付标准

- 至少形成一份后端选型与接口建议文档
- 明确 V1 不做什么，V2/V3 可做什么
- 如果有实验代码，应保证与主线隔离

## 验证命令

```powershell
cargo test noname_ -- --nocapture
```

## 备注

- 这项任务更偏技术预研，不适合作为当前主线优先事项
