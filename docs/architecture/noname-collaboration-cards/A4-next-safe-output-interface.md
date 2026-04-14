# A4 任务卡: 更高一层受控输出接口预研

状态: 未开始
优先级: 中高
任务量: 中
任务联系度: 高

## 任务目标

为 `NoName Agent` 后续从“低风险提示层”走向“更高一层受控输出层”预先设计接口，但本次不直接接管最终剧情状态机，也不直接修改当前 `T7 apply` 主线。

这项任务的核心是“先设计隔离层”，不是“直接往剧情主链里塞新控制逻辑”。

## 建议范围

建议本次聚焦:

- 盘点现有低风险 apply 之上的候选输出层
- 明确哪些输出层允许受控增强
- 明确哪些范围绝对不能碰
- 设计最小接口草案

候选方向示例:

- recap / recap note
- scene augmentation
- narrative note
- intermediate narrative hint

## 建议新增或修改文件

优先新增文档或独立接口草案:

- `docs/architecture/noname-safe-output-interface-v1.md`
- `src-tauri/src/plot_engine.rs`

如果需要，也可新增:

- `src-tauri/src/noname_output_interface.rs`

## 不要碰的文件

默认不要直接修改:

- `src-tauri/src/tauri_commands.rs`
- `src-tauri/src/noname_runtime.rs`
- `src-tauri/src/noname_guardrails.rs`
- `src-tauri/src/noname_trace.rs`
- `src/platform/webRuntime.ts`

原因:

- 当前主线正在收口 `T7 apply`
- 本任务应停留在接口设计与安全边界定义，不应闯入主入口

## 可接受的接入方式

推荐:

- 先出设计草案文档
- 如果需要代码，只做独立接口和示例 stub
- 接入点通过 trait / facade / adapter 方式设计

不推荐:

- 直接改现有 `execute_player_action`
- 直接推动 `NoName` 改写最终剧情正文生成主结果

## 交付标准

满足以下条件即可视为完成:

- 明确列出下一层允许受控 apply 的输出类型
- 明确列出禁止越界的输出范围
- 给出至少一版接口草案
- 至少补一个最小测试草案或 stub 示例
- 输出一份简短设计说明，方便后续本机主线接入

## 验证命令

如果只交付文档和接口草案:

```powershell
cargo test noname_ -- --nocapture
```

如果新增了接口模块测试:

```powershell
cargo test plot_engine -- --nocapture
```

## 交付物建议

建议协作者提交:

- 设计文档
- 最小接口草案
- 受控范围与禁区清单
- 最小测试或 stub

## 备注

这项任务适合“思路清晰、愿意先做设计隔离”的协作者，不适合直接当作功能开发去硬接主线。
