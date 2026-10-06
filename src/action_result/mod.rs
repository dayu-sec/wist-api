//! `agent/action-results` seam（agentd → gateway，动作结果上报 + 回执）的报文。
//!
//! 报文引用的**领域类型**（`ActionResult` / `FinalStatus`）仍留在 `wist-contracts`；这里
//! re-export，调用方从 `wist_api::action_result` 一处取齐。版本并存约定同
//! [`crate::enrollment`]（见 `wist-design/doc/design/foundation/api-seam-inventory.md` §7）。

pub use wist_contracts::action_result::{ActionResult, FinalStatus};

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

    /// 编译期身份断言：本模块 re-export 的领域类型**就是** `wist-contracts` 的那几个
    /// （若有人另抄一份，下面的函数体将类型不匹配而编译失败）。
    #[test]
    fn domain_types_are_the_contracts_ones() {
        #[allow(dead_code)]
        fn assert_identity() {
            fn result(v: wist_contracts::action_result::ActionResult) -> ActionResult {
                v
            }
            fn final_status(v: wist_contracts::action_result::FinalStatus) -> FinalStatus {
                v
            }
            let _ = (result, final_status);
        }
        assert_identity();
    }
}
