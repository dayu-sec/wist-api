//! `agent/uplink:poll` seam 报文：`PollAgentUplink`（拉取）与 `AgentUplinkGrant`（下发）。
//!
//! 「实际生效状态」`AgentUplinkState` **不是报文**，仍留在 `wist-contracts`；这里 re-export。
//! 版本并存约定同 [`crate::enrollment`]。

pub use wist_contracts::agent_uplink::AgentUplinkState;

pub mod v1;
pub use v1::*;

/// 当前线上版本。新代码从这里取版本口径；并存期由路由/协商决定。
pub const CURRENT: &str = v1::API_VERSION;
