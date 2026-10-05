//! `gateway` 面 agent seam 报文（action-plan / action-results / facts / discovery-policies）。
//!
//! 报文引用的**领域类型**（`ActionPlan` / `ActionResult` / `FinalStatus` /
//! `DiscoveryAspectPolicy(Set)`）仍留在 `wist-contracts`；这里 re-export，调用方从
//! `wist_api::gateway` 一处取齐。版本并存约定同 [`crate::enrollment`]
//! （见 `wist-design/doc/design/foundation/api-seam-inventory.md` §7）。

pub use wist_contracts::action_plan::ActionPlan;
pub use wist_contracts::action_result::{ActionResult, FinalStatus};
pub use wist_contracts::discovery_policy::{DiscoveryAspectPolicy, DiscoveryAspectPolicySet};

pub mod v1;
pub use v1::*;

/// 当前线上版本。新代码从这里取版本口径；并存期由路由/协商决定。
pub const CURRENT: &str = v1::API_VERSION;
