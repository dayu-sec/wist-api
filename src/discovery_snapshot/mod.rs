//! `agent/discovery-snapshot:poll` seam 报文：`PollDiscoverySnapshot`（拉取）与
//! `DiscoverySnapshotGrant`（下发）。
//!
//! ## 为什么单独开一条 seam
//!
//! 原文快照的上送周期需要**按机器**设置（资产变化快的主机要更密），而逐台改
//! `agentd.toml` 不可运维 —— 它由控制面在管理面（gateway-web）配置后下发。
//!
//! 但不能给 `AgentUplinkGrant` / `DiscoveryPoliciesReturned` 加字段：那两个契约两侧都
//! `deny_unknown_fields`，加字段会让「新网关 + 旧 agentd」解析失败。独立端点对两个方向都
//! 安全 —— 旧 agentd 从不调它；新 agentd 遇到旧网关得到 404，按「无下发」回落本机
//! `[discovery]` 配置（与 `agent/uplink:poll` 完全同构）。
//!
//! 版本并存约定同 [`crate::enrollment`]。

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
}
