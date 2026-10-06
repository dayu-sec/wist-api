//! `agent/action-results` seam —— **v1** 基线。
//!
//! 冻结基线：只做**加性**兼容不动它；非加性变更就新开 `v2`。
//! 约定见 `wist-design/doc/design/foundation/api-seam-inventory.md` §7。

use serde::{Deserialize, Serialize};

use wist_contracts::API_VERSION_V1;

use super::{ActionResult, FinalStatus};

/// 本版本的线上版本号（与路由 `/api/v1/…` 一致）。
pub const API_VERSION: &str = API_VERSION_V1;

pub const REPORT_ACTION_RESULT_KIND: &str = "report_action_result";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportActionResult {
    pub api_version: String,
    pub report_id: String,
    pub kind: String,
    pub dispatch_id: Option<String>,
    pub action_id: String,
    pub report_attempt: u32,
    pub final_status: FinalStatus,
    pub execution_id: String,
    pub plan_digest: String,
    pub agent_id: String,
    pub instance_id: String,
    pub result_attestation: ResultAttestation,
    pub reported_at: String,
    pub result: ActionResult,
}

impl ReportActionResult {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        report_id: String,
        action_id: String,
        report_attempt: u32,
        final_status: FinalStatus,
        execution_id: String,
        plan_digest: String,
        agent_id: String,
        instance_id: String,
        result_attestation: ResultAttestation,
        reported_at: String,
        result: ActionResult,
    ) -> Self {
        Self {
            api_version: API_VERSION_V1.to_string(),
            report_id,
            kind: REPORT_ACTION_RESULT_KIND.to_string(),
            dispatch_id: None,
            action_id,
            report_attempt,
            final_status,
            execution_id,
            plan_digest,
            agent_id,
            instance_id,
            result_attestation,
            reported_at,
            result,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultAttestation {
    /// Development placeholder until real signing and verifier plumbing is implemented.
    pub result_digest: String,
    /// Development placeholder signature. Consumers must not treat this as production attestation.
    pub signature: String,
    /// Development placeholder issuer identity, prefixed as `dev-placeholder:...`.
    pub issued_by: String,
    pub attested_at: String,
}

/// Gateway 对动作结果上报的确认响应。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionResultAck {
    pub report_id: String,
    pub agent_id: String,
    pub acknowledged_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attestation() -> ResultAttestation {
        ResultAttestation {
            result_digest: "sha256:abc".to_string(),
            signature: "dev-placeholder:sig".to_string(),
            issued_by: "dev-placeholder:agent-1".to_string(),
            attested_at: "2026-09-27T00:00:02Z".to_string(),
        }
    }

    #[test]
    fn report_action_result_new_sets_kind_and_leaves_dispatch_absent() {
        let result = ActionResult::new(
            "act-1".to_string(),
            "exec-1".to_string(),
            FinalStatus::Succeeded,
        );
        let report = ReportActionResult::new(
            "rep-1".to_string(),
            "act-1".to_string(),
            1,
            FinalStatus::Succeeded,
            "exec-1".to_string(),
            "sha256:plan".to_string(),
            "agent-1".to_string(),
            "inst-1".to_string(),
            attestation(),
            "2026-09-27T00:00:02Z".to_string(),
            result,
        );
        assert_eq!(report.api_version, API_VERSION_V1);
        assert_eq!(report.kind, REPORT_ACTION_RESULT_KIND);
        assert_eq!(report.dispatch_id, None);

        let json = serde_json::to_string(&report).expect("encode");
        let back: ReportActionResult = serde_json::from_str(&json).expect("decode");
        assert_eq!(back, report);
    }
}
