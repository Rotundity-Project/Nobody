# Nobody 文档导航

`docs/` 用于保存仓库内正式文档，面向产品、架构、QA、发布和长期维护。

## 与 `.kiro/` 的职责边界

- `docs/`：正式文档，适合长期保留、代码评审、团队协作、发布与回归。
- `.kiro/`：本地规格、工作草稿、阶段性计划、交接记录，不作为运行时依赖。

如果你要了解项目现状，优先看 `docs/`。
如果你要追溯本地思路演进、任务拆解和历史 handoff，再看 `.kiro/`。

## 推荐阅读顺序

1. `ARCHITECTURE.md`
2. `USER_MANUAL.md`
3. `API.md`
4. `architecture/README.md`
5. `qa/` 与 `release/` 下的专题文档

## 目录说明

### `architecture/`

系统设计与领域建模文档。

重点文件：

- `architecture/domain-model-v2.md`
- `architecture/noname-agent-v1.md`
- `architecture/noname-memory-context-v1.md`
- `architecture/noname-framework-protocol-v1.md`
- `architecture/noname-v1-blueprint.md`

### `qa/`

测试报告、属性测试矩阵、性能报告、手工验证记录。

### `release/`

版本说明、上线检查、迁移说明、发布模板。

### `ui/`

界面规范、文案词汇、视觉审计、主题约束。

## NoName Agent 文档入口

如果你要了解 `Nobody` 的 Agent 设计，请按这个顺序读：

1. `architecture/noname-agent-v1.md`
2. `architecture/noname-memory-context-v1.md`
3. `architecture/noname-framework-protocol-v1.md`
4. `architecture/noname-v1-blueprint.md`

## 维护规则

- 新的正式设计文档优先写入 `docs/`。
- 草稿、手记、对话式规划优先写入 `.kiro/`。
- 当 `.kiro/` 中的内容已经稳定，可提炼后迁移为 `docs/` 中的正式文档。
