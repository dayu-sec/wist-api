//! `agent/uplink:poll` seam 报文：`PollAgentUplink`（拉取）与 `AgentUplinkGrant`（下发）。
//!
//! 「实际生效状态」`AgentUplinkState` **不是报文**，仍留在 `wist-contracts`；这里 re-export。
//! 版本并存约定同 [`crate::enrollment`]。

pub use wist_contracts::agent_uplink::AgentUplinkState;

pub mod v1;
pub use v1::*;

/// 当前线上版本。新代码从这里取版本口径；并存期由路由/协商决定。
pub const CURRENT: &str = v1::API_VERSION;

#[cfg(test)]
mod tests {
    use super::*;

    /// `CURRENT` 必须钉在 `v1`（与路由 `/api/v1/…` 一致），并与版本子模块同源。
    #[test]
    fn current_matches_the_live_wire_version() {
        assert_eq!(CURRENT, "v1");
        assert_eq!(CURRENT, v1::API_VERSION);
    }

    /// 编译期身份断言：`AgentUplinkState` **就是** `wist-contracts` 的那一个（
    /// `agent_status` 与 `agent_uplink` 两处 re-export 必须指向同一类型）。
    #[test]
    fn domain_types_are_the_contracts_ones() {
        #[allow(dead_code)]
        fn assert_identity() {
            fn uplink_state(v: wist_contracts::agent_uplink::AgentUplinkState) -> AgentUplinkState {
                v
            }
            fn same_as_agent_status(v: AgentUplinkState) -> crate::agent_status::AgentUplinkState {
                v
            }
            let _ = (uplink_state, same_as_agent_status);
        }
        assert_identity();
    }
}
