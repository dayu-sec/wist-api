//! 跨进程 **seam 契约**测试：钉住每条 seam 报文的**线上形状**与**兼容策略**。
//!
//! 这些断言面向**线上 JSON**：两侧进程（发送端 / 接收端）都用 `wist-api` 里**同一份**类型
//! 解析下面的**黄金样例**，所以本文件就是 `api-seam-inventory.md` §6.5
//! 「每个 seam 一条『两侧 parse 同一类型』契约测试 + 兼容策略断言」的落点。
//!
//! 改字段名 / 改 `kind` / 改 `api_version` / 放松 `deny_unknown_fields`，都会在这里失败。
//! 约定见 `wist-design/doc/design/foundation/api-seam-inventory.md` §6.5 / §7。

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use wist_api::{
    action_plan, action_result, discovery_policies, enrollment, facts, status, uplink, work,
};

/// 黄金样例 → 类型 → 再编码 → 再解码：必须得到同一个值（形状稳定，且能两侧互认）。
fn round_trip<T>(golden: &str) -> T
where
    T: DeserializeOwned + Serialize + PartialEq + std::fmt::Debug,
{
    let value: T = serde_json::from_str(golden).expect("golden 样例必须能被该 seam 类型解码");
    let re = serde_json::to_value(&value).expect("re-encode");
    let back: T = serde_json::from_value(re).expect("re-decode");
    assert_eq!(back, value, "黄金样例必须往返稳定");
    value
}

/// 该报文的**线上字段集**（排序后）——多一个 / 少一个都会让断言失败。
fn keys<T: DeserializeOwned + Serialize>(golden: &str) -> Vec<String> {
    let value: T = serde_json::from_str(golden).expect("golden 样例必须能被该 seam 类型解码");
    let re = serde_json::to_value(&value).expect("re-encode");
    let mut keys: Vec<String> = re
        .as_object()
        .expect("报文是 JSON 对象")
        .keys()
        .cloned()
        .collect();
    keys.sort_unstable();
    keys
}

/// 兼容策略：报文多出未知字段必须**响亮报错**（本 crate 各报文统一 `deny_unknown_fields`）。
fn rejects_unknown<T: DeserializeOwned>(golden: &str) {
    let mut value: Value = serde_json::from_str(golden).expect("golden 是合法 JSON");
    value
        .as_object_mut()
        .expect("报文是 JSON 对象")
        .insert("__unknown_field__".to_string(), json!(1));
    assert!(
        serde_json::from_value::<T>(value).is_err(),
        "未知字段必须被拒（兼容策略为 strict）"
    );
}

/// 断言字段集（把 `expected` 列表写全，读起来就是这份报文的线上契约）。
fn assert_keys<T: DeserializeOwned + Serialize>(golden: &str, expected: &[&str]) {
    let actual = keys::<T>(golden);
    let mut expected: Vec<String> = expected.iter().map(|k| (*k).to_string()).collect();
    expected.sort_unstable();
    assert_eq!(actual, expected);
}

// ── 黄金样例（线上真实形状；两侧共用） ─────────────────────────────────────

const AGENT_STATUS_REPORT: &str =
    r#"{"agent_id":"agent-1","instance_id":"inst-1","version":"0.1.31"}"#;
const AGENT_STATUS_ACK: &str =
    r#"{"agent_id":"agent-1","instance_id":"inst-1","acknowledged_at":"2026-09-27T00:00:00Z"}"#;
const AGENT_CERTIFICATE_STATUS: &str =
    r#"{"not_after":"2026-11-04T00:00:00Z","remaining_seconds":100,"state":"valid"}"#;
const AGENT_WORK_STATE_CHANGE: &str = r#"{"input_id":"app","state":"paused","reason":"spool_over_limit","at":"2026-09-27T00:00:00Z"}"#;

const POLL_AGENT_UPLINK: &str = r#"{"api_version":"v1","kind":"poll_agent_uplink",
    "agent_id":"agent-1","instance_id":"inst-1","requested_at":"2026-09-27T00:00:00Z"}"#;
const AGENT_UPLINK_GRANT: &str =
    r#"{"enabled":true,"host":"gw.example","port":9000,"granted_at":"2026-09-27T00:00:00Z"}"#;

const POLL_WORK: &str = r#"{"api_version":"v1","kind":"poll_work","agent_id":"agent-1",
    "instance_id":"inst-1","last_seen_sequence":0,"wait_ms":0,"requested_at":"2026-09-27T00:00:00Z"}"#;
const WORK_GRANT: &str = r#"{"agent_id":"agent-1","standing":[],"one_shot":[],"sequence":1,"granted_at":"2026-09-27T00:00:00Z"}"#;
const ACK_WORK: &str = r#"{"api_version":"v1","kind":"ack_work","agent_id":"agent-1",
    "instance_id":"inst-1","work_id":"work-1","plan_version":2,"acknowledged_at":"2026-09-27T00:00:00Z"}"#;
const WORK_ACCEPTED: &str =
    r#"{"work_id":"work-1","status":"accepted","accepted_at":"2026-09-27T00:00:00Z"}"#;
const REPORT_WORK_RESULT: &str = r#"{"api_version":"v1","kind":"report_work_result",
    "agent_id":"agent-1","instance_id":"inst-1","work_id":"work-1","status":"succeeded",
    "detail":"","reported_at":"2026-09-27T00:00:00Z"}"#;
const WORK_RESULT_ACCEPTED: &str =
    r#"{"work_id":"work-1","status":"accepted","accepted_at":"2026-09-27T00:00:00Z"}"#;

const DISPATCH_ACTION_PLAN: &str = r#"{"api_version":"v1","kind":"dispatch_action_plan",
    "dispatch_id":"disp-1","plan":{"api_version":"v1","kind":"action_plan",
    "meta":{"action_id":"act-1","request_id":"req-1","template_id":null,
            "tenant_id":"t","environment_id":"e","plan_version":1,
            "compiled_at":"2026-09-27T00:00:00Z","expires_at":"2026-09-28T00:00:00Z"},
    "target":{"agent_id":"agent-1","instance_id":null,"node_id":"n","host_name":null,
              "platform":"macos","arch":"arm64","selectors":{}},
    "constraints":{"risk_level":"R1","approval_ref":null,"approval_mode":"not_required",
                   "requested_by":"admin","reason":null,"max_total_duration_ms":1000,
                   "step_timeout_default_ms":500,"execution_profile":"default",
                   "required_capabilities":[]},
    "program":{"entry":"s1","steps":[{"id":"s1","kind":"invoke","op":"shell"}]}}}"#;

const ACTION_PLAN_ACK: &str = r#"{"api_version":"v1","kind":"action_plan_ack",
    "dispatch_id":"disp-1","action_id":"act-1","plan_digest":"sha256:p","agent_id":"agent-1",
    "instance_id":"inst-1","execution_id":null,"ack_status":"accepted","reason_code":null,
    "reason_message":null,"queue_position":null,"received_at":"2026-09-27T00:00:00Z",
    "acknowledged_at":"2026-09-27T00:00:01Z"}"#;

const RESULT_ATTESTATION: &str = r#"{"result_digest":"sha256:r","signature":"s","issued_by":"dev-placeholder:a","attested_at":"t"}"#;

const REPORT_ACTION_RESULT: &str = r#"{"api_version":"v1","report_id":"rep-1",
    "kind":"report_action_result","dispatch_id":null,"action_id":"act-1","report_attempt":1,
    "final_status":"succeeded","execution_id":"exec-1","plan_digest":"sha256:p",
    "agent_id":"agent-1","instance_id":"inst-1",
    "result_attestation":{"result_digest":"sha256:r","signature":"s","issued_by":"dev-placeholder:a",
                          "attested_at":"t"},
    "reported_at":"2026-09-27T00:00:02Z",
    "result":{"api_version":"v1","kind":"action_result","action_id":"act-1",
              "execution_id":"exec-1","final_status":"succeeded","step_records":[],
              "outputs":{}}}"#;

const ACTION_RESULT_ACK: &str =
    r#"{"report_id":"rep-1","agent_id":"agent-1","acknowledged_at":"2026-09-27T00:00:03Z"}"#;

const REPORT_AGENT_FACT_SUMMARY: &str = r#"{"api_version":"v1","kind":"report_agent_fact_summary",
    "report_id":"rep-1","agent_id":"agent-1","instance_id":"inst-1","content_digest":"sha256:c",
    "revision":7,"observed_at":"2026-09-27T00:00:00Z","os":"macos","arch":"arm64",
    "process_count":3,"process_executables":["/bin/sh"],"packages":["curl"],
    "listen_ports":["tcp/22"],"reported_at":"2026-09-27T00:00:01Z"}"#;

const FACT_SUMMARY_ACCEPTED: &str = r#"{"report_id":"rep-1","agent_id":"agent-1",
    "content_digest":"sha256:c","ack_status":"accepted","suggestion_id":null,
    "received_at":"2026-09-27T00:00:02Z"}"#;

const POLL_DISCOVERY_POLICIES: &str = r#"{"api_version":"v1","kind":"poll_discovery_policies",
    "agent_id":"agent-1","instance_id":"inst-1","requested_at":"2026-09-27T00:00:00Z"}"#;
const DISCOVERY_POLICIES_RETURNED: &str = r#"{"policy_version":1,"published_at":"2026-09-27T00:00:00Z",
    "policies":[],"returned_at":"2026-09-27T00:00:01Z"}"#;

const CREDENTIAL_RENEWAL: &str = r#"{"api_version":"v1","kind":"renew_agent_credential",
    "agent_id":"agent-1","instance_id":"inst-1","credential_request":"cr",
    "certificate_signing_request":"csr","requested_at":"2026-09-27T00:00:00Z"}"#;

// ── 1. envelope kind：线上契约，改名即断 ──────────────────────────────────

#[test]
fn envelope_kinds_are_stable() {
    let kinds = [
        (
            enrollment::SUBMIT_ENROLLMENT_REQUEST_KIND,
            "submit_enrollment_request",
        ),
        (
            enrollment::RENEW_AGENT_CREDENTIAL_KIND,
            "renew_agent_credential",
        ),
        (uplink::POLL_AGENT_UPLINK_KIND, "poll_agent_uplink"),
        (work::POLL_WORK_KIND, "poll_work"),
        (work::ACK_WORK_KIND, "ack_work"),
        (work::REPORT_WORK_RESULT_KIND, "report_work_result"),
        (
            action_plan::DISPATCH_ACTION_PLAN_KIND,
            "dispatch_action_plan",
        ),
        (action_plan::ACTION_PLAN_ACK_KIND, "action_plan_ack"),
        (
            action_result::REPORT_ACTION_RESULT_KIND,
            "report_action_result",
        ),
        (
            facts::REPORT_AGENT_FACT_SUMMARY_KIND,
            "report_agent_fact_summary",
        ),
        (
            discovery_policies::POLL_DISCOVERY_POLICIES_KIND,
            "poll_discovery_policies",
        ),
    ];
    for (actual, expected) in kinds {
        assert_eq!(actual, expected, "envelope kind 是线上契约，不能改名");
    }

    // 顺带钉住「互不相同」：两份报文共用一个 kind 会让接收端分不清。
    let mut seen = std::collections::BTreeSet::new();
    for (actual, _) in kinds {
        assert!(seen.insert(actual), "kind 重复：{actual}");
    }
}

// ── 2. 线上版本：路由版本才是真源 ────────────────────────────────────────

#[test]
fn seam_versions_are_v1() {
    for (label, current) in [
        ("enrollment", enrollment::CURRENT),
        ("status", status::CURRENT),
        ("uplink", uplink::CURRENT),
        ("work", work::CURRENT),
        ("action_plan", action_plan::CURRENT),
        ("action_result", action_result::CURRENT),
        ("facts", facts::CURRENT),
        ("discovery_policies", discovery_policies::CURRENT),
    ] {
        assert_eq!(
            current, "v1",
            "{label} 的 CURRENT 必须与路由 /api/v1/… 一致"
        );
    }

    // 各版本子模块的 API_VERSION 也必须落在 v1。
    assert_eq!(enrollment::v1::API_VERSION, "v1");
    assert_eq!(status::v1::API_VERSION, "v1");
    assert_eq!(uplink::v1::API_VERSION, "v1");
    assert_eq!(work::v1::API_VERSION, "v1");
    assert_eq!(action_plan::v1::API_VERSION, "v1");
    assert_eq!(action_result::v1::API_VERSION, "v1");
    assert_eq!(facts::v1::API_VERSION, "v1");
    assert_eq!(discovery_policies::v1::API_VERSION, "v1");
}

// ── 3. 各 seam 的字段集 + 兼容策略 ───────────────────────────────────────

#[test]
fn status_seam_wire_contract() {
    round_trip::<status::AgentStatusReport>(AGENT_STATUS_REPORT);
    assert_keys::<status::AgentStatusReport>(
        AGENT_STATUS_REPORT,
        &[
            "admin_latency_ms",
            "agent_id",
            "certificate_status",
            "cpu_cores",
            "cpu_percent",
            "discovery_policy_version",
            "instance_id",
            "local_work",
            "machine_profile",
            "memory_bytes",
            "uplink_state",
            "version",
            "work_state_changes",
        ],
    );
    rejects_unknown::<status::AgentStatusReport>(AGENT_STATUS_REPORT);

    assert_keys::<status::AgentStatusAck>(
        AGENT_STATUS_ACK,
        &["acknowledged_at", "agent_id", "instance_id"],
    );
    assert_keys::<status::AgentCertificateStatus>(
        AGENT_CERTIFICATE_STATUS,
        &["last_renewal", "not_after", "remaining_seconds", "state"],
    );
    assert_keys::<status::AgentWorkStateChange>(
        AGENT_WORK_STATE_CHANGE,
        &["at", "input_id", "reason", "state"],
    );
    rejects_unknown::<status::AgentWorkStateChange>(AGENT_WORK_STATE_CHANGE);
}

#[test]
fn uplink_seam_wire_contract() {
    assert_keys::<uplink::PollAgentUplink>(
        POLL_AGENT_UPLINK,
        &[
            "agent_id",
            "api_version",
            "instance_id",
            "kind",
            "requested_at",
        ],
    );
    rejects_unknown::<uplink::PollAgentUplink>(POLL_AGENT_UPLINK);
    rejects_unknown::<uplink::AgentUplinkGrant>(AGENT_UPLINK_GRANT);
}

#[test]
fn work_seam_wire_contract() {
    assert_keys::<work::PollWork>(
        POLL_WORK,
        &[
            "agent_id",
            "api_version",
            "instance_id",
            "kind",
            "last_seen_sequence",
            "requested_at",
            "wait_ms",
        ],
    );
    rejects_unknown::<work::PollWork>(POLL_WORK);

    round_trip::<work::WorkGrant>(WORK_GRANT);
    assert_keys::<work::WorkGrant>(
        WORK_GRANT,
        &["agent_id", "granted_at", "one_shot", "sequence", "standing"],
    );

    assert_keys::<work::AckWork>(
        ACK_WORK,
        &[
            "acknowledged_at",
            "agent_id",
            "api_version",
            "instance_id",
            "kind",
            "plan_version",
            "work_id",
        ],
    );
    rejects_unknown::<work::AckWork>(ACK_WORK);

    assert_keys::<work::WorkAccepted>(WORK_ACCEPTED, &["accepted_at", "status", "work_id"]);
    assert_keys::<work::WorkAccepted>(WORK_RESULT_ACCEPTED, &["accepted_at", "status", "work_id"]);
    rejects_unknown::<work::WorkAccepted>(WORK_ACCEPTED);

    assert_keys::<work::ReportWorkResult>(
        REPORT_WORK_RESULT,
        &[
            "agent_id",
            "api_version",
            "detail",
            "instance_id",
            "kind",
            "reported_at",
            "status",
            "work_id",
        ],
    );
    rejects_unknown::<work::ReportWorkResult>(REPORT_WORK_RESULT);
}

#[test]
fn action_plan_seam_wire_contract() {
    round_trip::<action_plan::DispatchActionPlan>(DISPATCH_ACTION_PLAN);
    assert_keys::<action_plan::DispatchActionPlan>(
        DISPATCH_ACTION_PLAN,
        &["api_version", "dispatch_id", "kind", "plan"],
    );
    rejects_unknown::<action_plan::DispatchActionPlan>(DISPATCH_ACTION_PLAN);

    assert_keys::<action_plan::ActionPlanAck>(
        ACTION_PLAN_ACK,
        &[
            "ack_status",
            "acknowledged_at",
            "action_id",
            "agent_id",
            "api_version",
            "dispatch_id",
            "execution_id",
            "instance_id",
            "kind",
            "plan_digest",
            "queue_position",
            "reason_code",
            "reason_message",
            "received_at",
        ],
    );
    rejects_unknown::<action_plan::ActionPlanAck>(ACTION_PLAN_ACK);
}

#[test]
fn action_result_seam_wire_contract() {
    round_trip::<action_result::ReportActionResult>(REPORT_ACTION_RESULT);
    assert_keys::<action_result::ReportActionResult>(
        REPORT_ACTION_RESULT,
        &[
            "action_id",
            "agent_id",
            "api_version",
            "dispatch_id",
            "execution_id",
            "final_status",
            "instance_id",
            "kind",
            "plan_digest",
            "report_attempt",
            "report_id",
            "reported_at",
            "result",
            "result_attestation",
        ],
    );
    rejects_unknown::<action_result::ReportActionResult>(REPORT_ACTION_RESULT);

    assert_keys::<action_result::ResultAttestation>(
        RESULT_ATTESTATION,
        &["attested_at", "issued_by", "result_digest", "signature"],
    );
    assert_keys::<action_result::ActionResultAck>(
        ACTION_RESULT_ACK,
        &["acknowledged_at", "agent_id", "report_id"],
    );
}

#[test]
fn facts_seam_wire_contract() {
    round_trip::<facts::ReportAgentFactSummary>(REPORT_AGENT_FACT_SUMMARY);
    assert_keys::<facts::ReportAgentFactSummary>(
        REPORT_AGENT_FACT_SUMMARY,
        &[
            "agent_id",
            "api_version",
            "arch",
            "content_digest",
            "host_id",
            "host_name",
            "instance_id",
            "kind",
            "listen_ports",
            "network_addresses",
            "observed_at",
            "os",
            "packages",
            "process_count",
            "process_executables",
            "report_id",
            "reported_at",
            "revision",
        ],
    );
    rejects_unknown::<facts::ReportAgentFactSummary>(REPORT_AGENT_FACT_SUMMARY);

    assert_keys::<facts::FactSummaryAccepted>(
        FACT_SUMMARY_ACCEPTED,
        &[
            "ack_status",
            "agent_id",
            "content_digest",
            "received_at",
            "report_id",
            "suggestion_id",
        ],
    );
}

#[test]
fn discovery_policies_seam_wire_contract() {
    assert_keys::<discovery_policies::PollDiscoveryPolicies>(
        POLL_DISCOVERY_POLICIES,
        &[
            "agent_id",
            "api_version",
            "instance_id",
            "kind",
            "requested_at",
        ],
    );
    rejects_unknown::<discovery_policies::PollDiscoveryPolicies>(POLL_DISCOVERY_POLICIES);
    assert_keys::<discovery_policies::DiscoveryPoliciesReturned>(
        DISCOVERY_POLICIES_RETURNED,
        &["policies", "policy_version", "published_at", "returned_at"],
    );
}

#[test]
fn enrollment_seam_wire_contract() {
    assert_keys::<enrollment::CredentialRenewal>(
        CREDENTIAL_RENEWAL,
        &[
            "agent_id",
            "api_version",
            "certificate_signing_request",
            "credential_request",
            "instance_id",
            "kind",
            "requested_at",
        ],
    );
    rejects_unknown::<enrollment::CredentialRenewal>(CREDENTIAL_RENEWAL);
    assert_keys::<enrollment::CredentialRenewed>(
        r#"{"credential_bundle":{"credential_id":"c","agent_id":"a","instance_id":"i",
        "certificate":"PEM","issued_at":"t"}}"#,
        &["credential_bundle"],
    );
}

// ── 4. 枚举的线上名 + 未知变体拒收 ───────────────────────────────────────

#[test]
fn enum_wire_names_are_stable_and_unknown_variants_fail() {
    use action_plan::AckStatus;
    use enrollment::EnrollmentStatus;
    use facts::FactSummaryAckStatus;
    use status::AgentWorkState;

    for (value, expected) in [
        (AgentWorkState::Paused, "paused"),
        (AgentWorkState::Resumed, "resumed"),
    ] {
        assert_eq!(
            serde_json::to_string(&value).unwrap(),
            format!("\"{expected}\"")
        );
    }
    assert!(serde_json::from_str::<AgentWorkState>("\"unknown\"").is_err());

    for (value, expected) in [
        (AckStatus::Accepted, "accepted"),
        (AckStatus::Rejected, "rejected"),
        (AckStatus::Queued, "queued"),
        (AckStatus::Duplicate, "duplicate"),
        (AckStatus::Stale, "stale"),
        (AckStatus::Busy, "busy"),
    ] {
        assert_eq!(
            serde_json::to_string(&value).unwrap(),
            format!("\"{expected}\"")
        );
    }
    assert!(serde_json::from_str::<AckStatus>("\"unknown\"").is_err());

    for (value, expected) in [
        (FactSummaryAckStatus::Accepted, "accepted"),
        (FactSummaryAckStatus::Duplicate, "duplicate"),
        (FactSummaryAckStatus::Rejected, "rejected"),
    ] {
        assert_eq!(
            serde_json::to_string(&value).unwrap(),
            format!("\"{expected}\"")
        );
    }
    assert!(serde_json::from_str::<FactSummaryAckStatus>("\"unknown\"").is_err());

    for (value, expected) in [
        (EnrollmentStatus::Accepted, "accepted"),
        (EnrollmentStatus::Rejected, "rejected"),
        (EnrollmentStatus::PendingReview, "pending_review"),
    ] {
        assert_eq!(
            serde_json::to_string(&value).unwrap(),
            format!("\"{expected}\"")
        );
    }
    assert!(serde_json::from_str::<EnrollmentStatus>("\"unknown\"").is_err());
}

// ── 5. 单一定义：`AgentStatusAck` 只有一份 ──────────────────────────────

/// `AgentStatusAck` 只有**一份定义**（`status` seam）。
/// 若哪天在别处又「复制一份」，这个函数体就会类型不匹配而编译失败。
#[test]
fn status_ack_has_a_single_definition() {
    fn is_the_status_one(v: status::AgentStatusAck) -> status::AgentStatusAck {
        v
    }
    let ack: status::AgentStatusAck = round_trip(AGENT_STATUS_ACK);
    let same = is_the_status_one(ack);
    assert_eq!(same.agent_id, "agent-1");
}
