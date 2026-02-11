# 示例剧本

这个目录包含用于测试的示例剧本文件。

## test_script.json

这是一个完整的测试剧本，包含：

### 修仙境界
- 练气期（Qi Condensation）- 等级 1
- 筑基期（Foundation Establishment）- 等级 2
- 金丹期（Golden Core）- 等级 3
- 元婴期（Nascent Soul）- 等级 4
- 化神期（Spirit Transformation）- 等级 5

### 灵根类型
- 天灵根（Heavenly）- 火、水属性
- 双灵根（Double）- 木属性
- 三灵根（Triple）- 金属性
- 伪灵根（Pseudo）- 土属性

### 功法
- 基础聚气诀 - 练气期基础功法
- 烈焰掌 - 火系攻击功法
- 水盾术 - 水系防御功法
- 金丹凝练法 - 金丹期突破功法

### 地点
- 青云宗 - 主角起始地点，中型修仙宗门
- 迷雾森林 - 灵兽出没的危险区域
- 凡人镇 - 普通城镇
- 灵石矿 - 资源丰富的矿脉

### 势力
- 青云宗 - 正道宗门（实力 75）
- 暗影阁 - 中立情报组织（实力 60）
- 血魔宗 - 邪道宗门（实力 80）

### 初始状态
- 角色名：林凡
- 灵根：天灵根（火属性，亲和度 90%）
- 起始地点：青云宗
- 起始年龄：16 岁

## 如何使用

1. 启动游戏：`npm run tauri dev`
2. 点击"新游戏"
3. 选择"自定义剧本"
4. 在文件选择对话框中选择 `example_scripts/test_script.json`
5. 游戏将使用这个剧本初始化

## 剧本格式说明

剧本文件是 JSON 格式，必须包含以下字段：

- `id`: 剧本唯一标识符
- `name`: 剧本名称
- `script_type`: 剧本类型（Custom/RandomGenerated/ExistingNovel）
- `world_setting`: 世界设定
  - `cultivation_realms`: 修仙境界列表（至少1个）
  - `spiritual_roots`: 灵根类型列表
  - `techniques`: 功法列表
  - `locations`: 地点列表（至少1个）
  - `factions`: 势力列表
- `initial_state`: 初始状态
  - `player_name`: 玩家角色名
  - `player_spiritual_root`: 玩家灵根
  - `starting_location`: 起始地点（必须在 locations 中存在）
  - `starting_age`: 起始年龄（10-100 之间）

## 验证规则

剧本加载时会进行以下验证：

1. 必须至少有一个修仙境界
2. 必须至少有一个地点
3. 起始地点必须在地点列表中存在
4. 起始年龄必须在 10-100 之间

如果验证失败，会显示具体的错误信息。
