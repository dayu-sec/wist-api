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
