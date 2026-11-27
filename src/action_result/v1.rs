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

    fn report() -> ReportActionResult {
        let result = ActionResult::new(
            "act-1".to_string(),
            "exec-1".to_string(),
            FinalStatus::Succeeded,
        );
        ReportActionResult::new(
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
        )
    }

    #[test]
    fn report_action_result_new_sets_kind_and_leaves_dispatch_absent() {
        let report = report();
        assert_eq!(report.api_version, API_VERSION_V1);
        assert_eq!(report.kind, REPORT_ACTION_RESULT_KIND);
        assert_eq!(report.dispatch_id, None);

        let json = serde_json::to_string(&report).expect("encode");
        let back: ReportActionResult = serde_json::from_str(&json).expect("decode");
        assert_eq!(back, report);
    }

    /// 旧的发送方可能整份没有 `dispatch_id`（它只在随计划派发时才有）：
    /// 缺席 = `None`，而不是解析失败。这条与 `deny_unknown_fields` 是**互补**的两面 ——
    /// “少一个可选键”要能吃，“多一个未知键”要炸。
    #[test]
    fn a_report_without_a_dispatch_id_still_decodes() {
        let mut value = serde_json::to_value(report()).expect("encode");
        value.as_object_mut().expect("object").remove("dispatch_id");
        let back: ReportActionResult = serde_json::from_value(value).expect("decode");
        assert_eq!(back.dispatch_id, None);

        let mut with_dispatch = serde_json::to_value(report()).expect("encode");
        with_dispatch
            .as_object_mut()
            .expect("object")
            .insert("dispatch_id".to_string(), "disp-1".into());
        let back: ReportActionResult = serde_json::from_value(with_dispatch).expect("decode");
        assert_eq!(back.dispatch_id.as_deref(), Some("disp-1"));

        // 未知键仍然要炸（两侧各自演进时不允许静默吃下）。
        let mut drifted = serde_json::to_value(report()).expect("encode");
        drifted
            .as_object_mut()
            .expect("object")
            .insert("extra".to_string(), 1.into());
        assert!(serde_json::from_value::<ReportActionResult>(drifted).is_err());
    }

    #[test]
    fn the_ack_round_trips_and_rejects_drift() {
        let ack = ActionResultAck {
            report_id: "rep-1".to_string(),
            agent_id: "agent-1".to_string(),
            acknowledged_at: "2026-09-27T00:00:03Z".to_string(),
        };
        let json = serde_json::to_string(&ack).expect("encode");
        assert_eq!(
            serde_json::from_str::<ActionResultAck>(&json).expect("decode"),
            ack
        );

        let drifted = json.replacen('{', "{\"extra\":1,", 1);
        assert!(serde_json::from_str::<ActionResultAck>(&drifted).is_err());
    }
}
