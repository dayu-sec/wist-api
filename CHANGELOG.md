# 更新日志

本文件记录 `wist-api` 的所有重要变更。格式遵循 [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)，
版本号遵循[语义化版本](https://semver.org/lang/zh-CN/)。

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
