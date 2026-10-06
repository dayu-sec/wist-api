//! `agent/facts` seam（agentd → gateway，事实摘要上报 + 回执）的报文。
//!
//! 本 seam 的报文只用**基本类型**组装（进程/包/端口列表、摘要串…），不内嵌任何共享
//! 领域类型，故这里没有 `wist-contracts` 的 re-export。版本并存约定同 [`crate::enrollment`]
//! （见 `wist-design/doc/design/foundation/api-seam-inventory.md` §7）。

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
