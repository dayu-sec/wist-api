//! `agent/work:*` seam 报文（拉取授权快照 `PollWork`/`WorkGrant`、确认 `AckWork`/`WorkAccepted`、
//! 上报执行结果 `ReportWorkResult`/`WorkResultAccepted`）。
//!
//! 「工作参数与状态」类**领域**类型（`WorkSpec*` / `WorkKind` / `StandingWork` / `OneShotWork` 等）
//! 仍留在 `wist-contracts`；这里 re-export `WorkGrant` 内嵌到的那两个。版本并存约定同
//! [`crate::enrollment`]。

pub use wist_contracts::work::{OneShotWork, StandingWork};

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

    /// 编译期身份断言：`WorkGrant` 内嵌的两个工作类型**就是** `wist-contracts` 的那两个。
    #[test]
    fn domain_types_are_the_contracts_ones() {
        #[allow(dead_code)]
        fn assert_identity() {
            fn standing(v: wist_contracts::work::StandingWork) -> StandingWork {
                v
            }
            fn one_shot(v: wist_contracts::work::OneShotWork) -> OneShotWork {
                v
            }
            let _ = (standing, one_shot);
        }
        assert_identity();
    }
}
