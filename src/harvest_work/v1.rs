//! `harvest/*` seam 报文 —— **v1** 基线。
//!
//! 冻结基线：只做**加性**兼容不动它；非加性变更就新开 `v2`。
//!
//! 与 `agent/work:*` 的区别（照抄形状，不复用类型）：
//! - 身份是 **worker**（`harvest:<source_code>`），不是 agent；
//! - `standing.spec` 是**结构化对象**（不是「字符串里套 JSON」）；
//! - 接入与 agentd 同一套：本地密钥 → CSR → 网关 CA 签客户端证书（mTLS）。

use serde::{Deserialize, Serialize};

use wist_contracts::API_VERSION_V1;

/// 本版本的线上版本号（与路由 `/api/v1/…` 一致）。
pub const API_VERSION: &str = API_VERSION_V1;

/// harvest → 网关：拉取工作授权快照。
pub const POLL_WORK_KIND: &str = "poll_harvest_work";
/// harvest → 网关：确认收到工作。
pub const ACK_WORK_KIND: &str = "ack_harvest_work";
/// harvest → 网关：上报一次性工作的执行结果。
pub const REPORT_WORK_RESULT_KIND: &str = "report_harvest_work_result";
/// harvest → 网关：用一次性接入券换客户端证书。
pub const ENROLL_KIND: &str = "submit_harvest_enrollment";
/// harvest → 网关：用上一张证书续期。
pub const RENEW_CERTIFICATE_KIND: &str = "renew_harvest_certificate";

/// standing 工作唯一的 family（本期）。
pub const FAMILY_INVENTORY: &str = "inventory";

// ─────────────────────────── 工作参数（spec） ───────────────────────────

/// 常驻工作的参数：**结构化对象**，由网关物化、harvest 只应用。
///
/// 纪律与 agentd 一致：解析失败不当「没事可做」，而是明确报错（页面显示「参数坏了」）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HarvestSpec {
    /// 云账号凭据的**引用名**（如 `tcloud:prod`）。真值只在宿主 secret 文件，
    /// 永不下发（设计 §3.4）。
    pub credential_ref: String,
    /// `TENCENT_CLOUD` / `AWS`。与数据源定义不一致时拒绝应用。
    pub vendor: String,
    /// `cloud_host` / `vpc` / `subnet` / `eip`。空数组 = 什么都不采（合法，但页面告警）。
    #[serde(default)]
    pub assets: Vec<String>,
    /// 区域范围：`"auto"` 或显式白名单。
    #[serde(default)]
    pub regions: HarvestRegions,
    /// 采集节奏。`on_demand` = 不排定时，只响应一次性工作。
    pub cadence: HarvestCadence,
    /// 限速；缺省用本机默认。`null` 与缺省同义。
    #[serde(default)]
    pub rate: Option<HarvestRate>,
}

impl HarvestSpec {
    /// 是否按定时调度（`on_demand` 返回 `false`）。
    pub fn is_scheduled(&self) -> bool {
        !self.cadence.is_on_demand()
    }

    /// 采集面是否为空。
    pub fn collects_nothing(&self) -> bool {
        self.assets.is_empty()
    }
}

/// 区域范围。
///
/// 用**未标记枚举**而不是 `serde_json::Value`：`"auto"` 与数组两种形态都要能被
/// `deny_unknown_fields` 之外的校验拿住（网关落库前会拒绝空串与空数组）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HarvestRegions {
    /// 只接受字面量 `"auto"`（由网关校验，不在反序列化期做，避免把错误藏进类型）。
    Auto(String),
    /// 显式区域白名单。
    List(Vec<String>),
}

impl Default for HarvestRegions {
    fn default() -> Self {
        Self::Auto("auto".to_string())
    }
}

impl HarvestRegions {
    pub fn is_auto(&self) -> bool {
        matches!(self, Self::Auto(value) if value == "auto")
    }
}

/// 采集节奏。
///
/// **普通结构 + `mode` 判别字段**（而不是 tagged enum）：只有一个原因 ——
/// serde 的 `deny_unknown_fields` 对 tagged / untagged enum 不生效，
/// 那样 `{"mode":"on_demand","oops":1}` 会被静默接受，接口契约就不再是严格的。
/// `mode` 的取值与字段要求由网关的 `validate_spec` 校验。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HarvestCadence {
    /// `daily` | `interval` | `on_demand`
    pub mode: String,
    /// `mode=daily` 必填：`HH:MM`（本地时区）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at: Option<String>,
    /// `mode=daily` 可选：抖动上限（秒）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jitter_seconds: Option<u64>,
    /// `mode=interval` 必填。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds: Option<u64>,
}

pub const CADENCE_DAILY: &str = "daily";
pub const CADENCE_INTERVAL: &str = "interval";
pub const CADENCE_ON_DEMAND: &str = "on_demand";

impl HarvestCadence {
    pub fn daily(at: impl Into<String>, jitter_seconds: u64) -> Self {
        Self {
            mode: CADENCE_DAILY.to_string(),
            at: Some(at.into()),
            jitter_seconds: Some(jitter_seconds),
            seconds: None,
        }
    }

    pub fn interval(seconds: u64) -> Self {
        Self {
            mode: CADENCE_INTERVAL.to_string(),
            at: None,
            jitter_seconds: None,
            seconds: Some(seconds),
        }
    }

    pub fn on_demand() -> Self {
        Self {
            mode: CADENCE_ON_DEMAND.to_string(),
            at: None,
            jitter_seconds: None,
            seconds: None,
        }
    }

    pub fn is_on_demand(&self) -> bool {
        self.mode == CADENCE_ON_DEMAND
    }
}

/// 限速（云 API 有配额，多站点同日采集时靠它错开）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HarvestRate {
    pub qps: f64,
    #[serde(default)]
    pub burst: f64,
}

// ─────────────────────────── 工作授权 ───────────────────────────

/// 常驻工作：一个 worker 一条（`family = inventory`）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StandingHarvestWork {
    pub work_id: String,
    pub family: String,
    pub spec: HarvestSpec,
    /// 工作按哪一版采集目录展开。本期恒为 `0`（D3：目录化留到 P3）。
    #[serde(default)]
    pub catalog_version: i64,
    /// 生效依据：指向已批准的提案；人工直填时为 `null`（D3）。
    #[serde(default)]
    pub proposal_id: Option<String>,
    /// 期望版本：网关每次改动 +1；harvest 回报实际版本，与它比对即得漂移。
    pub plan_version: i64,
    pub effective_from: String,
    /// `active` | `paused` | `revoked`。`revoked` 是终态（不进快照，见 `build_grant`）。
    pub status: String,
    pub updated_by: String,
    pub updated_at: String,
}

impl StandingHarvestWork {
    pub fn is_active(&self) -> bool {
        self.status == STATUS_ACTIVE
    }
}

/// standing 的对外状态（`revoked` 不进快照：撤销 = 不返回 `standing`）。
pub const STATUS_ACTIVE: &str = "active";
pub const STATUS_PAUSED: &str = "paused";
pub const STATUS_REVOKED: &str = "revoked";

/// 一次性工作：有明确终态。
///
/// 本期（P1）网关不派，字段先定型，避免 P2 加字段动契约。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OneShotHarvestWork {
    pub work_id: String,
    /// `inventory.run` | `inventory.rescan`
    pub action: String,
    pub spec: OneShotHarvestSpec,
    pub scheduled_at: String,
    pub deadline_at: String,
    pub timeout_seconds: i64,
    pub interruptible: bool,
    /// 见 `ONE_SHOT_WORK_STATUSES`。
    pub status: String,
    #[serde(default)]
    pub attempt: i64,
    pub issued_by: String,
    pub issued_at: String,
}

impl OneShotHarvestWork {
    /// 是否**未了结**（快照里只带未了结的活）。
    pub fn is_outstanding(&self) -> bool {
        !ONE_SHOT_TERMINAL_STATUSES.contains(&self.status.as_str())
    }
}

/// 一次性工作的参数（三个 action 共用；未用到的字段留空）。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OneShotHarvestSpec {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assets: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub regions: Option<HarvestRegions>,
    /// `true` = 忽略可续跑 spool，强制开新 `snap_id`。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fresh: Option<bool>,
}

/// 一次性工作的状态取值。
pub const ONE_SHOT_WORK_STATUSES: &[&str] = &[
    "dispatched",
    "accepted",
    "running",
    "succeeded",
    "failed",
    "timed_out",
    "canceled",
    "expired",
];

/// 终态：到这些状态后结果不再被覆盖。
pub const ONE_SHOT_TERMINAL_STATUSES: &[&str] =
    &["succeeded", "failed", "timed_out", "canceled", "expired"];

/// 工作授权快照。
///
/// 幂等、可重复拉取；`sequence` 只用来判断「这份跟我手上的有没有变」，
/// **不承担「指令重放」的语义**。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HarvestWorkGrant {
    pub worker_id: String,
    pub source_code: String,
    /// `None` = 未授权 / 已撤销（此时 harvest 不调度）。
    ///
    /// 刻意**不** `skip_serializing_if`：未授权时字段是 `null`，而不是缺失 ——
    /// 非 Rust 客户端按 `standing: Standing | null` 读，缺字段会被当成 `undefined`。
    pub standing: Option<StandingHarvestWork>,
    /// 只带**未了结**的一次性工作。
    #[serde(default)]
    pub one_shot: Vec<OneShotHarvestWork>,
    /// 授权序号（单调递增：每次授权/改 spec/暂停/继续/撤销都 +1）。
    pub sequence: i64,
    pub granted_at: String,
}

// ─────────────────────────── 请求 / 响应 ───────────────────────────

/// harvest → 网关：拉取工作授权快照。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PollHarvestWork {
    pub api_version: String,
    pub kind: String,
    pub worker_id: String,
    pub instance_id: String,
    /// 与本机手上那份的序号；网关可据此短路（本期仍返回完整快照）。
    pub last_seen_sequence: i64,
    /// 将来的长轮询预留（现在立即返回）。
    pub wait_ms: i64,
    pub requested_at: String,
}

impl PollHarvestWork {
    pub fn new(
        worker_id: impl Into<String>,
        instance_id: impl Into<String>,
        last_seen_sequence: i64,
        requested_at: impl Into<String>,
    ) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            kind: POLL_WORK_KIND.to_string(),
            worker_id: worker_id.into(),
            instance_id: instance_id.into(),
            last_seen_sequence,
            wait_ms: 0,
            requested_at: requested_at.into(),
        }
    }
}

/// harvest → 网关：确认收到某份工作。
///
/// `plan_version` 变化后**也必须**确认：网关据此判断「期望的版本真到了吗」，
/// 一直没确认的就是漂移。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AckHarvestWork {
    pub api_version: String,
    pub kind: String,
    pub worker_id: String,
    pub instance_id: String,
    pub work_id: String,
    pub plan_version: i64,
    pub acknowledged_at: String,
}

impl AckHarvestWork {
    pub fn new(
        worker_id: impl Into<String>,
        instance_id: impl Into<String>,
        work_id: impl Into<String>,
        plan_version: i64,
        acknowledged_at: impl Into<String>,
    ) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            kind: ACK_WORK_KIND.to_string(),
            worker_id: worker_id.into(),
            instance_id: instance_id.into(),
            work_id: work_id.into(),
            plan_version,
            acknowledged_at: acknowledged_at.into(),
        }
    }
}

/// 网关对 [`AckHarvestWork`] 的回应。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HarvestWorkAccepted {
    pub work_id: String,
    /// `accepted` | `stale` | `unknown`。
    pub status: String,
    pub accepted_at: String,
}

/// harvest → 网关：上报一次性工作的执行结果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportHarvestResult {
    pub api_version: String,
    pub kind: String,
    pub worker_id: String,
    pub instance_id: String,
    pub work_id: String,
    /// `running` | `succeeded` | `failed`。
    pub status: String,
    /// 人看的说明：失败原因**原样带上**。
    #[serde(default)]
    pub detail: String,
    pub reported_at: String,
}

impl ReportHarvestResult {
    pub fn new(
        worker_id: impl Into<String>,
        instance_id: impl Into<String>,
        work_id: impl Into<String>,
        status: impl Into<String>,
        detail: impl Into<String>,
        reported_at: impl Into<String>,
    ) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            kind: REPORT_WORK_RESULT_KIND.to_string(),
            worker_id: worker_id.into(),
            instance_id: instance_id.into(),
            work_id: work_id.into(),
            status: status.into(),
            detail: detail.into(),
            reported_at: reported_at.into(),
        }
    }
}

/// 网关对 [`ReportHarvestResult`] 的回应。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HarvestResultAccepted {
    pub work_id: String,
    /// `accepted` | `stale` | `unknown`。`stale` = 工作已到终态，后到的结果不覆盖。
    pub status: String,
    pub accepted_at: String,
}

// ─────────────────────────── 接入 / 续期 ───────────────────────────

/// harvest → 网关：用一次性接入券换客户端证书。
///
/// 与 `agent/enroll` 同一套纪律：**私钥永不上送**，只交 CSR；主体（URI SAN）由网关填，
/// CSR 里声明的 subject / SAN 一律忽略。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HarvestEnrollmentRequest {
    pub api_version: String,
    pub kind: String,
    pub ticket: String,
    pub worker_id: String,
    pub source_code: String,
    pub instance_id: String,
    pub certificate_signing_request: String,
    pub requested_at: String,
}

impl HarvestEnrollmentRequest {
    pub fn new(
        ticket: impl Into<String>,
        worker_id: impl Into<String>,
        source_code: impl Into<String>,
        instance_id: impl Into<String>,
        certificate_signing_request: impl Into<String>,
        requested_at: impl Into<String>,
    ) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            kind: ENROLL_KIND.to_string(),
            ticket: ticket.into(),
            worker_id: worker_id.into(),
            source_code: source_code.into(),
            instance_id: instance_id.into(),
            certificate_signing_request: certificate_signing_request.into(),
            requested_at: requested_at.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HarvestEnrollmentEnvelope {
    pub result: HarvestEnrollmentOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HarvestEnrollmentOutcome {
    pub status: HarvestEnrollmentStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worker_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    /// 签好的客户端证书（PEM）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub certificate_pem: Option<String>,
    /// 验链用的 CA 证书（PEM）。只用于让 harvest 确信「对面是这把 CA」，
    /// 验 client 的根**只留服务端**。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ca_certificate_pem: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub not_after: Option<String>,
}

impl HarvestEnrollmentOutcome {
    pub fn rejected(reason_code: impl Into<String>) -> Self {
        Self {
            status: HarvestEnrollmentStatus::Rejected,
            reason_code: Some(reason_code.into()),
            worker_id: None,
            instance_id: None,
            certificate_pem: None,
            ca_certificate_pem: None,
            not_after: None,
        }
    }

    pub fn accepted(
        worker_id: impl Into<String>,
        instance_id: impl Into<String>,
        certificate_pem: impl Into<String>,
        ca_certificate_pem: impl Into<String>,
        not_after: impl Into<String>,
    ) -> Self {
        Self {
            status: HarvestEnrollmentStatus::Accepted,
            reason_code: None,
            worker_id: Some(worker_id.into()),
            instance_id: Some(instance_id.into()),
            certificate_pem: Some(certificate_pem.into()),
            ca_certificate_pem: Some(ca_certificate_pem.into()),
            not_after: Some(not_after.into()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HarvestEnrollmentStatus {
    #[serde(rename = "accepted")]
    Accepted,
    #[serde(rename = "rejected")]
    Rejected,
}

/// harvest → 网关：用上一张客户端证书续期。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HarvestCertificateRenewal {
    pub api_version: String,
    pub kind: String,
    pub worker_id: String,
    pub instance_id: String,
    pub certificate_signing_request: String,
    pub requested_at: String,
}

impl HarvestCertificateRenewal {
    pub fn new(
        worker_id: impl Into<String>,
        instance_id: impl Into<String>,
        certificate_signing_request: impl Into<String>,
        requested_at: impl Into<String>,
    ) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            kind: RENEW_CERTIFICATE_KIND.to_string(),
            worker_id: worker_id.into(),
            instance_id: instance_id.into(),
            certificate_signing_request: certificate_signing_request.into(),
            requested_at: requested_at.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HarvestCertificateRenewed {
    pub certificate_pem: String,
    pub ca_certificate_pem: String,
    pub not_after: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> HarvestSpec {
        HarvestSpec {
            credential_ref: "tcloud:prod".to_string(),
            vendor: "TENCENT_CLOUD".to_string(),
            assets: vec!["vpc".to_string(), "subnet".to_string()],
            regions: HarvestRegions::Auto("auto".to_string()),
            cadence: HarvestCadence::daily("00:30", 300),
            rate: Some(HarvestRate {
                qps: 5.0,
                burst: 5.0,
            }),
        }
    }

    #[test]
    fn grant_round_trips_with_serde() {
        let grant = HarvestWorkGrant {
            worker_id: "harvest:tencent-cloud-prod".to_string(),
            source_code: "tencent-cloud-prod".to_string(),
            standing: Some(StandingHarvestWork {
                work_id: "hsw:harvest:tencent-cloud-prod:inventory".to_string(),
                family: FAMILY_INVENTORY.to_string(),
                spec: spec(),
                catalog_version: 0,
                proposal_id: None,
                plan_version: 3,
                effective_from: "2026-11-14T00:00:00Z".to_string(),
                status: STATUS_ACTIVE.to_string(),
                updated_by: "admin".to_string(),
                updated_at: "2026-11-14T08:00:00Z".to_string(),
            }),
            one_shot: vec![],
            sequence: 13,
            granted_at: "2026-11-14T08:30:05Z".to_string(),
        };
        let json = serde_json::to_string(&grant).expect("serialize");
        let decoded: HarvestWorkGrant = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, grant);
    }

    #[test]
    fn on_demand_cadence_is_not_scheduled() {
        let mut value = spec();
        value.cadence = HarvestCadence::on_demand();
        assert!(!value.is_scheduled());
        assert!(spec().is_scheduled());
    }

    #[test]
    fn cadence_encodes_mode_and_rejects_unknown_fields() {
        let daily = serde_json::to_value(HarvestCadence::daily("00:30", 300)).unwrap();
        assert_eq!(daily["mode"], "daily");
        assert_eq!(daily["at"], "00:30");

        let interval = serde_json::to_value(HarvestCadence::interval(3600)).unwrap();
        assert_eq!(interval["mode"], "interval");

        let on_demand = serde_json::to_value(HarvestCadence::on_demand()).unwrap();
        assert_eq!(on_demand["mode"], "on_demand");

        // tagged enum 的 `deny_unknown_fields` 是空操作；结构体版才是真的。
        let extra = r#"{"mode":"on_demand","oops":1}"#;
        assert!(serde_json::from_str::<HarvestCadence>(extra).is_err());
    }

    #[test]
    fn regions_accept_auto_and_lists() {
        let auto: HarvestRegions = serde_json::from_str("\"auto\"").unwrap();
        assert!(auto.is_auto());
        let list: HarvestRegions = serde_json::from_str(r#"["ap-guangzhou"]"#).unwrap();
        assert_eq!(list, HarvestRegions::List(vec!["ap-guangzhou".to_string()]));
    }

    #[test]
    fn grant_rejects_unknown_fields() {
        let json = r#"{"worker_id":"w","source_code":"s","standing":null,"one_shot":[],
                       "sequence":1,"granted_at":"t","extra":1}"#;
        assert!(serde_json::from_str::<HarvestWorkGrant>(json).is_err());
    }

    #[test]
    fn spec_rejects_unknown_fields() {
        let json = r#"{"credential_ref":"c","vendor":"TENCENT_CLOUD","assets":[],
                       "cadence":{"mode":"on_demand"},"extra":1}"#;
        assert!(serde_json::from_str::<HarvestSpec>(json).is_err());
    }

    #[test]
    fn one_shot_outstanding_uses_terminal_statuses() {
        let work = OneShotHarvestWork {
            work_id: "w".to_string(),
            action: "inventory.run".to_string(),
            spec: OneShotHarvestSpec::default(),
            scheduled_at: "t".to_string(),
            deadline_at: "t".to_string(),
            timeout_seconds: 60,
            interruptible: true,
            status: "running".to_string(),
            attempt: 0,
            issued_by: "admin".to_string(),
            issued_at: "t".to_string(),
        };
        assert!(work.is_outstanding());
        let mut done = work.clone();
        done.status = "succeeded".to_string();
        assert!(!done.is_outstanding());
    }

    #[test]
    fn poll_and_ack_builders_use_v1_kinds() {
        let poll = PollHarvestWork::new("w", "i", 0, "t");
        assert_eq!(poll.api_version, "v1");
        assert_eq!(poll.kind, POLL_WORK_KIND);
        let ack = AckHarvestWork::new("w", "i", "work", 1, "t");
        assert_eq!(ack.kind, ACK_WORK_KIND);
    }

    #[test]
    fn renewal_builder_uses_the_renewal_kind() {
        let renewal = HarvestCertificateRenewal::new("w", "i", "csr", "t");
        assert_eq!(renewal.api_version, "v1");
        assert_eq!(renewal.kind, RENEW_CERTIFICATE_KIND);
        assert_eq!(renewal.certificate_signing_request, "csr");
    }
}
