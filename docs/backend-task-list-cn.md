# AuroraPulse 后端任务表

更新日期：2026-07-26

## 产品目标

AuroraPulse 是供授权 Agent 使用的本地个人资料与记忆层。第一阶段先验证聊天资料是否真的能帮助 Agent 理解用户，再从分析结果中沉淀长期记忆。

第一个闭环：

```text
用户导出一段微信聊天记录
    -> Aurora 导入并标准化消息
    -> 用户限定会话和时间范围
    -> Codex 通过 MCP 分页读取该范围
    -> Codex 根据用户的问题分析聊天
    -> 分析结论引用对应消息和时间
```

第二个闭环将在第一个闭环完成后开始：

```text
Codex 从分析中提出记忆候选
    -> 用户确认或修正
    -> Aurora 保存正式记忆
    -> 后续 Agent 可以检索
    -> 用户可以修改、过期或删除
```

## 状态说明

- `[x]` 已完成
- `[ ]` 尚未开始
- 每次只完成一个小任务，完成后停下来复核
- 未进入当前任务的代码不提前实现

## 当前指针

```text
当前状态：第一个闭环 / Aurora Chat JSON v1 解析与校验已完成，等待复核
当前任务：将标准 JSON 解析为强类型结构并校验全部 v1 约束（已完成）
下一步：实现 TXT 最小解析格式
现在不要做：微信解析、MCP 聊天工具、长期记忆、数字替身面板
```

## 已有基础

- [x] 后端目录使用 `backend/`
- [x] 现有 Rust MCP 项目已移动到 `backend/`
- [x] 根目录 Cargo workspace 可正常测试
- [x] 后端按 API、Application、Domain、Infrastructure 分层
- [x] MCP Adapter 只保留 MCP 协议职责
- [x] Context Gateway、Disclosure Policy、本地文件、审计和数据库分别归位
- [x] 本地身份、当前关注、偏好和隐私文件
- [x] `get_identity`
- [x] `get_current_focus`
- [x] `search_personal_context`
- [x] `DisclosurePolicy`
- [x] 任务级 `ContextPack`
- [x] 本地 MCP 访问审计
- [x] Codex 真实 MCP 调用验证

## 已确认的开发约定

- [x] 使用 Docker 运行独立的本地 PostgreSQL
- [x] 当前开发环境为 PostgreSQL 15 Alpine
- [x] 本地开发数据库名使用 `aurorapulse`
- [x] 容器只绑定 `127.0.0.1:5434`，避免与现有数据库冲突
- [x] `DATABASE_URL` 从环境变量或 `.env` 读取
- [x] 第一版先导入 Aurora 标准 JSON/TXT，不直接解决所有微信导出格式
- [x] 原始聊天是证据，不自动成为用户记忆
- [x] Codex 只能读取用户明确选择的会话和时间范围

## Milestone 1：PostgreSQL 基础

目标：Rust 后端可以可靠连接本地 PostgreSQL 并运行迁移，原有 MCP 读取能力不回归。

### 1.1 依赖和连接约定

- [x] 引入 `sqlx` PostgreSQL 依赖
- [x] 在 `.env.example` 增加 `DATABASE_URL`
- [x] 保持原有测试通过

### 1.2 连接池和迁移入口

- [x] 创建 `backend/migrations/`
- [x] 建立 `PgPool`
- [x] 启动时运行迁移
- [x] 数据库连接失败时返回明确错误

### 1.3 健康检查和测试配置

- [ ] 增加数据库健康检查
- [x] 创建本地 Docker 开发数据库
- [x] 增加 PostgreSQL 集成测试配置
- [ ] 验证迁移可以从空数据库执行

验收：

```text
后端启动
-> 连接 PostgreSQL
-> 自动运行迁移
-> health check 成功
-> 原有 get_identity 仍然正常
```

## Milestone 2：聊天来源领域模型

目标：定义导入批次、会话、消息和授权分析范围，不涉及 MCP。

- [x] 定义 `ImportBatch`
- [x] 定义 `Conversation`
- [x] 定义 `Message`
- [x] 定义 `AnalysisScope`
- [x] 明确消息发送者、时间、文本和原始顺序
- [x] 明确 Analysis Scope 的会话、开始时间和结束时间
- [ ] 为模型约束增加单元测试

约束：

- 原始消息不可被静默改写
- 每条消息必须能追溯到导入批次
- Analysis Scope 必须有明确的时间边界
- 暂时不处理图片、语音、视频和文件内容

## Milestone 3：聊天数据库结构与 Repository

目标：能够保存和读取标准化聊天记录。

- [x] 创建 `import_batches` 表
- [x] 创建 `conversations` 表
- [x] 创建 `messages` 表
- [x] 创建 `analysis_scopes` 表
- [x] 为会话、时间和消息顺序建立索引
- [x] 实现新建导入批次
- [x] 实现保存会话和消息
- [x] 实现原始文件哈希去重
- [x] 实现创建 Analysis Scope
- [x] 实现按 Scope 和游标分页读取消息
- [x] 增加真实 PostgreSQL 集成测试

## Milestone 4：标准聊天导入

目标：先通过稳定的 Aurora 标准格式跑通导入，不被微信格式差异阻塞。

- [x] 定义 Aurora 标准聊天 JSON 格式
- [x] 准备一份不含私人内容的测试样例
- [x] 实现 JSON 解析与校验
- [ ] 实现 TXT 最小解析格式
- [ ] 实现导入预览
- [ ] 实现导入命令
- [ ] 显示导入的会话数、消息数和时间范围
- [ ] 重复导入同一文件不会重复保存消息

首版命令目标：

```text
aurora chat import <file>
aurora chat conversations
```

## Milestone 5：用户限定分析范围

目标：用户先授权范围，Agent 不能自行扩大读取边界。

- [ ] 按会话创建 Analysis Scope
- [ ] 要求开始和结束时间
- [ ] Scope 保存创建时间和用途
- [ ] Scope 可以检查和撤销
- [ ] Scope 过期后不可读取

首版命令目标：

```text
aurora chat scope create <conversation> --from <time> --to <time>
aurora chat scope show <scope-id>
aurora chat scope revoke <scope-id>
```

## Milestone 6：MCP 聊天读取工具

目标：Codex 只在一个有效 Scope 内分页读取聊天。

- [ ] 新增 `get_analysis_scope`
- [ ] 新增 `read_chat_window`
- [ ] 使用稳定游标分页
- [ ] 限制每页消息数量和内容长度
- [ ] 返回发送者、时间和可引用的消息 ID
- [ ] 禁止读取 Scope 外的会话或时间
- [ ] 对每次聊天披露应用隐私策略
- [ ] 对每次读取写入本地审计
- [ ] 增加 MCP 工具测试

工具权限：

```text
get_analysis_scope    source.read.scoped
read_chat_window      source.read.scoped
```

## Milestone 7：Codex 真实分析闭环

- [ ] 导入一份真实但范围可控的微信聊天记录
- [ ] 用户创建会话和时间范围
- [ ] Codex 连接 Aurora MCP
- [ ] Codex 分页读取完整范围
- [ ] Codex 根据一个明确问题完成分析
- [ ] 主要结论引用消息 ID 和时间
- [ ] 审计日志可以说明披露了什么
- [ ] Scope 撤销后 Codex 无法继续读取

完成以上验收后，第一个闭环才算完成。

## 第二个闭环：长期个人记忆

第一个闭环完成前不实现。

- [ ] 定义 `MemoryCandidate`
- [ ] Codex 从聊天分析中提出候选
- [ ] 用户确认、修正或拒绝候选
- [ ] 定义正式 `Memory`
- [ ] 保存来源消息引用
- [ ] 保存 `MemoryRevision`
- [ ] 支持修改、过期和删除
- [ ] MCP 只检索有效且允许披露的正式记忆
- [ ] 删除或拒绝的理解不会被自动重新提议

## 后续任务：现在不要做

### 微信格式适配

- [ ] 调研实际可获得的微信导出格式
- [ ] 微信 TXT/HTML/备份 Adapter
- [ ] 联系人与群成员映射
- [ ] 图片、语音、视频和文件元数据

### 其他来源

- [ ] GPT/Gemini 对话 Adapter
- [ ] 邮件 Adapter
- [ ] 笔记和文档 Adapter

### 数字替身面板

- [ ] 本地 Dashboard API
- [ ] 聊天来源和导入记录
- [ ] Analysis Scope 管理
- [ ] 人生时间线
- [ ] 长期记忆管理
- [ ] Agent 授权与访问记录

### 完整授权系统

- [ ] 客户端身份
- [ ] `source.read.scoped`
- [ ] `memory.read`
- [ ] `memory.propose`
- [ ] 敏感资料权限
- [ ] 临时授权、永久授权和撤销

### 更强检索

- [ ] PostgreSQL 全文检索
- [ ] 时间和人物过滤
- [ ] 冲突记忆检测
- [ ] 来源质量排序
- [ ] 有真实需求后再评估语义检索和向量数据库

## 每次开发结束前检查

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo test --all-targets`
- [ ] 当前任务对应的测试已增加并通过
- [ ] 原有 MCP 读取链路没有回归
- [ ] 新增披露行为有审计记录
- [ ] 文档中的当前指针已更新
