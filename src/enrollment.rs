//! `agent/enroll` 与 `agent/credentials:renew` 两条 edge seam 的报文定义。
//!
//! 这是**跨进程 seam 报文**的唯一一份定义：网关（接收端）与 agentd（发送端）都 `use` 这里，
//! 不再各自复制。报文引用的**领域类型**（`HostProfile` / `AgentIdentity` / `CredentialBundle`
//! / `InitialConfig` / `PolicyBinding`）是两侧共同底座，仍留在 `wist-contracts`；这里 re-export，
//! 调用方可以从 `wist_api::enrollment` 一处取齐整条 seam。

use serde::{Deserialize, Serialize};

use wist_contracts::API_VERSION_V1;

pub use wist_contracts::enrollment::{
    AgentIdentity, AgentIdentityStatus, CredentialBundle, HostProfile, InitialConfig, PolicyBinding,
};

pub const SUBMIT_ENROLLMENT_REQUEST_KIND: &str = "submit_enrollment_request";
pub const RENEW_AGENT_CREDENTIAL_KIND: &str = "renew_agent_credential";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnrollmentRequest {
    pub api_version: String,
    pub kind: String,
    pub token: String,
    pub credential_request: String,
    /// agent 本地生成的 **CSR**（PEM）。mTLS 是唯一凭据路径，注册**必须**带 CSR ——
    /// 网关据此签一张客户端证书；没有它就没有凭据可用（不再回落 bearer）。
    ///
    /// 私钥**永不上送**，只交公钥；且**主体由网关填** —— CSR 里声明的 subject/SAN 一律忽略，
    /// 网关按稳定哈希 `agent_id` 生成 URI SAN（见 `docs/design/agent-identity-mtls.md` §4.2）。
    pub certificate_signing_request: String,
    pub host_profile: HostProfile,
    pub capability_summary: String,
    pub requested_at: String,
}

impl EnrollmentRequest {
    pub fn new(
        token: String,
        credential_request: String,
        certificate_signing_request: String,
        host_profile: HostProfile,
        capability_summary: String,
        requested_at: String,
    ) -> Self {
        Self {
            api_version: API_VERSION_V1.to_string(),
            kind: SUBMIT_ENROLLMENT_REQUEST_KIND.to_string(),
            token,
            credential_request,
            certificate_signing_request,
            host_profile,
            capability_summary,
            requested_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnrollmentEnvelope {
    pub result: EnrollmentOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnrollmentOutcome {
    pub status: EnrollmentStatus,
    pub reason_code: Option<String>,
    pub agent_id: Option<String>,
    pub instance_id: Option<String>,
    pub issued_identity: Option<AgentIdentity>,
    pub credential_bundle: Option<CredentialBundle>,
    pub initial_config: Option<InitialConfig>,
    pub policy_binding: Option<PolicyBinding>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnrollmentStatus {
    #[serde(rename = "accepted")]
    Accepted,
    #[serde(rename = "rejected")]
    Rejected,
    #[serde(rename = "pending_review")]
    PendingReview,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CredentialRenewal {
    pub api_version: String,
    pub kind: String,
    pub agent_id: String,
    pub instance_id: String,
    pub credential_request: String,
    /// 续期时提交的 **CSR**（PEM）。与注册同口径：私钥不上送、主体由网关填
    /// （见 `docs/design/agent-identity-mtls.md` §4.2）。mTLS 是唯一凭据路径，续期**必须**带它。
    pub certificate_signing_request: String,
    pub requested_at: String,
}

impl CredentialRenewal {
    pub fn new(
        agent_id: String,
        instance_id: String,
        credential_request: String,
        certificate_signing_request: String,
        requested_at: String,
    ) -> Self {
        Self {
            api_version: API_VERSION_V1.to_string(),
            kind: RENEW_AGENT_CREDENTIAL_KIND.to_string(),
            agent_id,
            instance_id,
            credential_request,
            certificate_signing_request,
            requested_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CredentialRenewed {
    pub credential_bundle: CredentialBundle,
}

#[cfg(test)]
mod tests {
    use super::{
        CredentialRenewal, EnrollmentEnvelope, EnrollmentOutcome, EnrollmentRequest,
        EnrollmentStatus, HostProfile, RENEW_AGENT_CREDENTIAL_KIND,
    };

    #[test]
    fn enrollment_result_status_uses_wire_names() {
        let decoded: EnrollmentEnvelope =
            serde_json::from_str(r#"{"result":{"status":"accepted","reason_code":null,"agent_id":"agent-1","instance_id":"host-a","issued_identity":null,"credential_bundle":null,"initial_config":null,"policy_binding":null}}"#)
                .expect("decode");

        assert_eq!(decoded.result.status, EnrollmentStatus::Accepted);

        let encoded = serde_json::to_string(&EnrollmentOutcome {
            status: EnrollmentStatus::PendingReview,
            reason_code: Some("manual_review".to_string()),
            agent_id: None,
            instance_id: None,
            issued_identity: None,
            credential_bundle: None,
            initial_config: None,
            policy_binding: None,
        })
        .expect("encode");

        assert!(encoded.contains("\"pending_review\""));
    }

    #[test]
    fn renew_agent_credential_uses_stable_wire_kind() {
        let request = CredentialRenewal::new(
            "agent-a".to_string(),
            "instance-a".to_string(),
            "csr".to_string(),
            "-----BEGIN CERTIFICATE REQUEST-----\nA\n".to_string(),
            "2026-07-29T00:00:00Z".to_string(),
        );
        let encoded = serde_json::to_string(&request).expect("encode");

        assert!(encoded.contains(&format!("\"kind\":\"{RENEW_AGENT_CREDENTIAL_KIND}\"")));
        assert!(encoded.contains("certificate_signing_request"));

        let decoded: CredentialRenewal = serde_json::from_str(&encoded).expect("decode");
        assert_eq!(decoded.api_version, "v1");
        assert_eq!(decoded.kind, RENEW_AGENT_CREDENTIAL_KIND);

        // CSR 是必填：缺该字段的报文解不了（不再有「只要 bearer」的双轨报文）。
        let mut without_csr = serde_json::to_value(&request).expect("encode");
        without_csr
            .as_object_mut()
            .expect("object")
            .remove("certificate_signing_request");
        assert!(serde_json::from_value::<CredentialRenewal>(without_csr).is_err());
    }

    #[test]
    fn enrollment_status_uses_wire_names_and_rejects_unknown_variants() {
        for (status, name) in [
            (EnrollmentStatus::Accepted, "accepted"),
            (EnrollmentStatus::Rejected, "rejected"),
            (EnrollmentStatus::PendingReview, "pending_review"),
        ] {
            assert_eq!(
                serde_json::to_string(&status).unwrap(),
                format!("\"{name}\"")
            );
        }
        assert!(serde_json::from_str::<EnrollmentStatus>("\"unknown\"").is_err());
    }

    #[test]
    fn enrollment_request_carries_a_required_csr() {
        let profile = sample_host_profile();
        let request = EnrollmentRequest::new(
            "token-a".to_string(),
            "csr".to_string(),
            "-----BEGIN CERTIFICATE REQUEST-----\nA\n-----END CERTIFICATE REQUEST-----\n"
                .to_string(),
            profile,
            "wist-agentd:test".to_string(),
            "2026-09-28T00:00:00Z".to_string(),
        );
        let json = serde_json::to_string(&request).expect("encode");
        assert!(json.contains("certificate_signing_request"));
        let back: EnrollmentRequest = serde_json::from_str(&json).expect("decode");
        assert_eq!(back, request);

        // CSR 必填：缺该字段的报文解不了（注册不再有「不带证书」的退路）。
        let mut without_csr = serde_json::to_value(&request).expect("encode");
        without_csr
            .as_object_mut()
            .expect("object")
            .remove("certificate_signing_request");
        assert!(serde_json::from_value::<EnrollmentRequest>(without_csr).is_err());
    }

    fn sample_host_profile() -> HostProfile {
        HostProfile {
            node_id: "node-1".to_string(),
            hostname: "host-1".to_string(),
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            machine_id: "mid-1".to_string(),
            cloud_instance_id: None,
            k8s_node_uid: None,
            ip_addresses: vec!["10.0.0.1".to_string()],
        }
    }

    #[test]
    fn an_outcome_without_optional_payloads_decodes() {
        // 拒绝的注册只带 status/reason_code：其余 Option 字段缺省即可。
        let json = r#"{"result":{"status":"rejected","reason_code":"bad_token"}}"#;
        let envelope: EnrollmentEnvelope = serde_json::from_str(json).expect("decode");
        assert_eq!(envelope.result.status, EnrollmentStatus::Rejected);
        assert!(envelope.result.issued_identity.is_none());
        assert!(envelope.result.credential_bundle.is_none());
    }
}
