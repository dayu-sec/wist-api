# 更新日志

本文件记录 `wist-api` 的所有重要变更。格式遵循 [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)，
版本号遵循[语义化版本](https://semver.org/lang/zh-CN/)。

## [0.5.1] - 2026-10-06

审查后的收口与测试补齐。**非破坏**（公共路径不变）。

### 修复

- **去掉 `AgentStatusAck` 的第二份定义**：它原先在 `agent_status` 与 `gateway` **各定义一份**——
  同 crate 里两份同名报文正是本 crate 要消灭的漂移（wire 相同，任一侧改字段就变成「同名不同形」）。
  现在 `agent_status` 是唯一定义，`wist_api::gateway::AgentStatusAck` 改为 **re-export**（旧路径仍能用）。

### 新增

- **seam 契约集成测试**（`tests/wire_contract.rs`）：钉住 11 个 envelope `kind`、5 个模块的 `CURRENT`/`API_VERSION`、
  各报文的**线上字段集**与 **`deny_unknown_fields` 兼容策略**，并为每条 seam 备了**黄金样例 JSON**
  （发送端/接收端 parse 同一份，即 `api-seam-inventory.md` §6.5 的落点）；另含枚举线上名/未知变体拒收。
- 4 个模块（`agent_status` / `agent_uplink` / `gateway` / `work`）补齐与 `enrollment` 同款的
  **模块级守护测试**：`CURRENT == "v1"` + re-export 领域类型的**编译期身份断言**。

### 文档

- README 模块表补全（原先只列了 `enrollment`）；修正 `agent_uplink` 里指向已迁出报文的陈旧引用。

## [0.5.0] - 2026-10-05

### 变更

- **`gateway` / `work` / `agent_uplink` 规范成 `v1` 子模块结构**（与 `enrollment` / `agent_status`
  一致）：新增 `<mod>::v1::API_VERSION` 与 `<mod>::CURRENT`；报文路径 `<mod>::<Item>` 经
  `pub use v1::*` **保持不变**（无破坏）。

## [0.4.0] - 2026-10-05

### 新增

- **`work` seam 模块**：`PollWork` / `WorkGrant` / `AckWork` / `WorkAccepted` / `ReportWorkResult` /
  `WorkResultAccepted` + kind 常量（由 `wist-contracts::work` 迁来，**线上 JSON 不变**）；
  re-export `WorkGrant` 内嵌的 `StandingWork` / `OneShotWork`。
- **`agent_uplink` seam 模块**：`PollAgentUplink` / `AgentUplinkGrant`（由 `wist-contracts::agent_uplink`
  迁来，**线上 JSON 不变**）。「状态」类 `AgentUplinkState` 仍留 `wist-contracts`，这里 re-export。

### 说明

- 这是「报文 vs 领域」的切分：**报文**进本 crate，**领域 / 状态**（`WorkSpec*` / `WorkKind` /
  `StandingWork` / `OneShotWork` / 状态常量 / 可执行性判定 …）留在 `wist-contracts`。

## [0.3.0] - 2026-10-05

### 新增

- **`gateway` seam 模块**：agent 面其余 seam 报文整体收进本 crate —— action-plan
  （`DispatchActionPlan` / `ActionPlanAck` / `AckStatus`）、action-results（`ReportActionResult` /
  `ResultAttestation` / `ActionResultAck`）、facts（`ReportAgentFactSummary` / `FactSummaryAccepted` /
  `FactSummaryAckStatus`）、discovery-policies（`PollDiscoveryPolicies` / `DiscoveryPoliciesReturned`）
  及 kind 常量。由 `wist-contracts::gateway` 整体迁来（**线上 JSON 不变**）；re-export 引用到的
  `ActionPlan` / `ActionResult` / `FinalStatus` / `DiscoveryAspectPolicy(Set)`。

## [0.2.0] - 2026-10-05

### 新增

- **`agent_status` seam 模块**：`agent/status` 的报文收进本 crate ——
  `AgentStatusReport`（agentd 周期上报）/ `AgentStatusAck`（回执），以及
  `AgentWorkState` / `AgentWorkStateChange` / `AgentCertificateStatus` / `AgentCredentialRenewal`，
  由 `wist-contracts::gateway` 迁来（**线上 JSON 不变**）。re-export 引用到的 `HostProfile` /
  `AgentLocalWork` / `AgentUplinkState`；结构沿用版本子模块（`agent_status::v1` + `CURRENT`）。

## [0.1.2] - 2026-10-05

### 变更

- **`enrollment` 按版本分模块，为 API v1/v2 并存做结构准备**：报文移入 `enrollment::v1`，
  `enrollment::EnrollmentRequest` 等经 `pub use v1::*` **路径不变**（无破坏）。新增
  `enrollment::CURRENT`（= v1 的线上版本号）与 `enrollment::v1::API_VERSION`。
  约定见 `wist-design/.../api-seam-inventory.md` §7。

### 内部

- 补守卫测试：钉住 `EnrollmentRequest` / `EnrollmentOutcome` 的线上字段集、`CURRENT` 与报文
  `api_version` 一致、re-export 的领域类型确为 `wist-contracts` 的那几个（编译期身份断言）。

## [0.1.1] - 2026-10-05

### 变更

- 依赖 `wist-contracts` 由 `0.2` 升到 **`0.3`**（后者移出了本 crate 承接的 enrollment 报文；
  领域类型不变）。

## [0.1.0] - 2026-10-05

首个版本：把跨进程 **API seam** 的报文收进一个独立 crate，让「同一条 seam 两侧用同一份定义」。

### 新增

- crate `wist-api` 与模块 `enrollment`：`agent/enroll` 与 `agent/credentials:renew` 两条 edge seam 的报文
  - `EnrollmentRequest`（含 `api_version` / `kind` / `certificate_signing_request`）
  - `EnrollmentEnvelope` / `EnrollmentOutcome` / `EnrollmentStatus`
  - `CredentialRenewal` / `CredentialRenewed`
  - 稳定 wire kind 常量：`SUBMIT_ENROLLMENT_REQUEST_KIND` / `RENEW_AGENT_CREDENTIAL_KIND`
- re-export 报文引用的**领域类型**（`HostProfile` / `AgentIdentity` / `AgentIdentityStatus` /
  `CredentialBundle` / `InitialConfig` / `PolicyBinding`），调用方从 `wist_api::enrollment` 一处取齐。

### 说明

- 报文原定义在 `wist-contracts::enrollment`；此类**领域类型仍留在 `wist-contracts`**（两侧共同底座），
  seam 报文（请求/响应体）迁到本 crate。`wist-contracts` 下一次发版将删除这些报文副本。
- 本 crate 只承载**跨进程 wire 报文**，不承载数据面/内部对象。
