//! Cross-process **agent-seam** messages for the `wist` control plane (gateway ↔ agentd).
//!
//! A *seam* is a runtime coupling point between two independently deployed processes
//! (`{route, request body, response body, owner, compat, api_version}`). This crate owns the
//! **wire messages** of the agent-facing seams so gateway and agentd `use` a single definition
//! instead of keeping a copy each. (The center-facing seams — `gateway/register`,
//! `gateway/status`, … — are generated into `wist-control`, not owned here.)
//!
//! Each module is one **seam topic** and maps to a route: `enrollment` (`agent/enroll`,
//! `agent/credentials:renew`), `status` (`agent/status`), `uplink` (`agent/uplink:poll`),
//! `work` (`agent/work:*`), `action_plan` (`agent/action-plan`), `action_result`
//! (`agent/action-results`), `facts` (`agent/facts`), `discovery_policies`
//! (`agent/discovery-policies:poll`).
//!
//! Seam messages reference **domain types** that are shared by more than one seam
//! (`HostProfile`, `CredentialBundle`, `AgentIdentity`, …). Those live in `wist-contracts`
//! — the common base of every participant — and are re-exported here so callers can pull a
//! whole seam from one place. `wist-contracts` keeps data-plane / internal objects only.

pub mod action_plan;
pub mod action_result;
pub mod discovery_policies;
pub mod enrollment;
pub mod facts;
pub mod status;
pub mod uplink;
pub mod work;
