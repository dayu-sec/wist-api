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
//! (`agent/discovery-policies:poll`), `discovery_snapshot`
//! (`agent/discovery-snapshot:poll`).
//!
//! Seam messages reference **domain types** that are shared by more than one seam
//! (`HostProfile`, `CredentialBundle`, `AgentIdentity`, …). Those live in `wist-contracts`
//! — the common base of every participant — and are re-exported here so callers can pull a
//! whole seam from one place. `wist-contracts` keeps data-plane / internal objects only.

pub mod action_plan;
pub mod action_result;
pub mod discovery_policies;
pub mod discovery_snapshot;
pub mod enrollment;
pub mod facts;
pub mod harvest_work;
pub mod metrics_labels;
pub mod status;
pub mod uplink;
pub mod work;

#[cfg(test)]
mod wire_kinds {
    use std::collections::BTreeSet;

    /// 每个 seam 的 envelope kind 与它的**线上字面量**。
    ///
    /// 这张表是路由与分支的键：改常量**名**不影响任何东西，但改**值**不会有编译错误 ——
    /// 网关会把帧当未知 kind 丢掉，agent 会把命令回成 `unsupported`。
    /// 与 `status::command_outcome::ACCEPTED` 那条同理，一律按字面量钉。
    const KINDS: &[(&str, &str)] = &[
        ("poll_agent_uplink", crate::uplink::POLL_AGENT_UPLINK_KIND),
        ("poll_work", crate::work::POLL_WORK_KIND),
        ("ack_work", crate::work::ACK_WORK_KIND),
        ("report_work_result", crate::work::REPORT_WORK_RESULT_KIND),
        ("poll_harvest_work", crate::harvest_work::POLL_WORK_KIND),
        ("ack_harvest_work", crate::harvest_work::ACK_WORK_KIND),
        (
            "report_harvest_work_result",
            crate::harvest_work::REPORT_WORK_RESULT_KIND,
        ),
        (
            "submit_harvest_enrollment",
            crate::harvest_work::ENROLL_KIND,
        ),
        (
            "renew_harvest_certificate",
            crate::harvest_work::RENEW_CERTIFICATE_KIND,
        ),
        (
            "dispatch_action_plan",
            crate::action_plan::DISPATCH_ACTION_PLAN_KIND,
        ),
        ("action_plan_ack", crate::action_plan::ACTION_PLAN_ACK_KIND),
        (
            "report_action_result",
            crate::action_result::REPORT_ACTION_RESULT_KIND,
        ),
        (
            "report_agent_fact_summary",
            crate::facts::REPORT_AGENT_FACT_SUMMARY_KIND,
        ),
        (
            "poll_discovery_policies",
            crate::discovery_policies::POLL_DISCOVERY_POLICIES_KIND,
        ),
        (
            "poll_discovery_snapshot",
            crate::discovery_snapshot::POLL_DISCOVERY_SNAPSHOT_KIND,
        ),
        (
            "poll_agent_metrics_labels",
            crate::metrics_labels::POLL_AGENT_METRICS_LABELS_KIND,
        ),
        (
            "agent_metrics_labels",
            crate::metrics_labels::AGENT_METRICS_LABELS_KIND,
        ),
        (
            "admin_agent_metrics_labels",
            crate::metrics_labels::ADMIN_METRICS_LABELS_KIND,
        ),
        (
            "set_agent_metrics_labels",
            crate::metrics_labels::SET_AGENT_METRICS_LABELS_KIND,
        ),
        (
            "submit_enrollment_request",
            crate::enrollment::SUBMIT_ENROLLMENT_REQUEST_KIND,
        ),
        (
            "renew_agent_credential",
            crate::enrollment::RENEW_AGENT_CREDENTIAL_KIND,
        ),
    ];

    #[test]
    fn envelope_kinds_keep_their_wire_names() {
        for (wire, actual) in KINDS {
            assert_eq!(
                wire, actual,
                "envelope kind 的线上值变了（两侧会互相听不懂）"
            );
        }
    }

    /// kind 是路由的键：两个 seam 撞名，一方的帧会被另一方吃掉。
    #[test]
    fn envelope_kinds_are_unique() {
        let unique: BTreeSet<&str> = KINDS.iter().map(|(_, actual)| *actual).collect();
        assert_eq!(unique.len(), KINDS.len(), "重复的 kind：{unique:?}");
    }

    /// 线上 kind 一律小写下划线：大写/空格/连字符在路由、日志与命令台账里都是坑。
    #[test]
    fn envelope_kinds_are_lowercase_snake_case() {
        for (_, actual) in KINDS {
            assert!(!actual.is_empty());
            assert!(
                actual
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
                "{actual}"
            );
        }
    }
}
