//! `agent/status` seam（edge seam B：gateway ↔ agentd）的报文。
//!
//! agentd 周期性上报自身状态，网关收下并落库。报文引用的**领域 / 数据面类型**
//! （`HostProfile` / `AgentLocalWork` / `AgentUplinkState`）仍留在 `wist-contracts`；这里 re-export，
//! 调用方从 `wist_api::agent_status` 一处取齐。版本并存约定同 [`crate::enrollment`]
//! （见 `wist-design/doc/design/foundation/api-seam-inventory.md` §7）。

pub use wist_contracts::agent_uplink::AgentUplinkState;
pub use wist_contracts::enrollment::HostProfile;
pub use wist_contracts::local_work::AgentLocalWork;

pub mod v1;
pub use v1::*;

/// 当前线上版本。新代码从这里取版本口径；并存期由路由/协商决定。
pub const CURRENT: &str = v1::API_VERSION;
