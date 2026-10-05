//! `agent/enroll` 与 `agent/credentials:renew` 两条 edge seam 的报文。
//!
//! 这是**跨进程 seam 报文**的唯一一份定义：网关（接收端）与 agentd（发送端）都 `use` 这里，
//! 不再各自复制。报文引用的**领域类型**（`HostProfile` / `AgentIdentity` / `CredentialBundle`
//! / `InitialConfig` / `PolicyBinding`）是两侧共同底座，仍留在 `wist-contracts`；这里 re-export，
//! 调用方可以从 `wist_api::enrollment` 一处取齐整条 seam。
//!
//! ## 版本并存约定（v1 / v2 …）
//!
//! 一条 seam 的**每一版**是独立一组 `{route, req, resp}`：各自一个版本子模块（`v1`、`v2`…）
//! 与各自的 handler。**不**用一个结构体 + `api_version` 分支硬扛两版。
//!
//! - **非加性（breaking）变更** → 新开一版 + 新路由（`/api/v2/…`）；老版本子模块**冻结、只增不删**。
//! - **纯加字段** → 才在同版本内演进，并按 `upgrade-order.md` §4 **接收端先升**
//!   （其余情况接收端多为 `deny_unknown_fields`，非加性一律走新版本）。
//! - `CURRENT` 指出新代码该用的版本；**路由并存 / 版本协商**留待真正出现 v2 时再接。
//!
//! 约定详见 `wist-design/doc/design/foundation/api-seam-inventory.md` §7。

/// 版本无关的**领域类型**（被多条 seam 复用），来自 `wist-contracts`。
pub use wist_contracts::enrollment::{
    AgentIdentity, AgentIdentityStatus, CredentialBundle, HostProfile, InitialConfig, PolicyBinding,
};

pub mod v1;
pub use v1::*;

/// 当前线上版本。新代码从这里取版本口径；并存期由路由/协商决定。
pub const CURRENT: &str = "v1";
