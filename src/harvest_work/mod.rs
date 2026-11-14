//! `harvest/work:*` seam 报文（wist-gateway ↔ wist-harvest）。
//!
//! 与 agentd 的 [`crate::work`] 同形，但**不复用**那条 seam：worker 不是被纳管的机器，
//! 身份、存储与页面都不同（见 `wist-gateway/docs/design/harvest-work-grant.md`）。
//!
//! 版本并存约定同 [`crate::enrollment`]：这里只增不删，非加性变更新开 v2。

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
