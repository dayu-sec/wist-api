//! Cross-process **API seam** messages for the `wist` control plane.
//!
//! A *seam* is a runtime coupling point between two independently deployed processes
//! (`{route, request body, response body, owner, compat, api_version}`). This crate owns the
//! **wire messages** of each seam so both endpoints `use` a single definition instead of
//! keeping a copy each.
//!
//! Seam messages reference **domain types** that are shared by more than one seam
//! (`HostProfile`, `CredentialBundle`, `AgentIdentity`, …). Those live in `wist-contracts`
//! — the common base of every participant — and are re-exported here so callers can pull a
//! whole seam from one place. `wist-contracts` keeps data-plane / internal objects only.

pub mod agent_status;
pub mod enrollment;
