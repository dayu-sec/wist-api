//! `agent/status` seam —— **v1** 基线。
//!
//! agentd 周期性上报自身状态（CPU/内存/工作状态/发现策略版本/证书状态/机器画像…），网关收下并落库。
//! 冻结基线：只做**加性**兼容不动它；需要非加性变更就新开 `v2`。
//! 约定见 `wist-design/doc/design/foundation/api-seam-inventory.md` §7。

use serde::{Deserialize, Serialize};

use super::{AgentLocalWork, AgentUplinkState, HostProfile};

/// 本版本的线上版本号（与路由 `/api/v1/…` 一致）。
pub const API_VERSION: &str = wist_contracts::API_VERSION_V1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentWorkState {
    Paused,
    Resumed,
}

/// 工作状态变化（非告警、非失败）：暂停/恢复各上报一次。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentWorkStateChange {
    pub input_id: String,
    pub state: AgentWorkState,
    pub reason: String,
    pub at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentStatusReport {
    pub agent_id: String,
    pub instance_id: String,
    pub version: String,
    /// Own resident-set size in bytes reported by the agent.
    #[serde(default)]
    pub memory_bytes: Option<u64>,
    /// Agent 进程自身 CPU 占用，**单核口径**（100% = 占满一个核；多线程进程可 >100）。
    ///
    /// 它只统计 agent 进程**自己**的 CPU 时间（`getrusage(RUSAGE_SELF)`），不含它拉起的子进程。
    #[serde(default)]
    pub cpu_percent: Option<f64>,
    /// Agent 所在机器的**逻辑核数**（`available_parallelism`）。
    ///
    /// 为什么必须和 `cpu_percent` 同一份上报带上来：`cpu_percent` 是单核口径，
    /// 而运维看图时真正常问的是「这台机器被它占了百分之几」——那是 `cpu_percent / 核数`。
    /// 少了核数，右侧那个数既算不出来、也无法复核（4 核上的 13% 和 64 核上的 13% 完全不是一回事）。
    ///
    /// 为什么让 agent 报原始值、而不是它自己算好整机占比：沿用本仓已有的取舍
    /// （与 `discovery_policy_version`、事实摘要 digest 同理）——agent 只交**原始事实**
    /// （自己的 CPU 时间、自己的核数），换算只留一处，在网关。agent 自算的派生值
    /// 一旦算法退化，下游没有任何一层能发现。
    #[serde(default)]
    pub cpu_cores: Option<u32>,
    /// Measured round-trip latency to the admin control plane in milliseconds.
    #[serde(default)]
    pub admin_latency_ms: Option<u64>,
    /// 自上次上报以来的工作状态变化（paused/resumed），非告警、非失败。
    #[serde(default)]
    pub work_state_changes: Option<Vec<AgentWorkStateChange>>,
    /// 本机**实际生效**的发现方向策略版本；`None` = 还没拿到策略表（在用内建默认周期）。
    ///
    /// 为什么必须由 agent 上报、而不是网关自己记账：网关知道自己**发布**了哪一版，
    /// 但不知道某台机器**拉到并应用**了哪一版 —— 拉取可能失败、可能还没到轮询节拍、
    /// 也可能拿到后被夹取。而运维要回答的正是那句「我改了策略，哪些机器还没生效」。
    #[serde(default)]
    pub discovery_policy_version: Option<i64>,
    /// 本机**工作内容视图**（`state/work.json` 的子集）：我手里有哪些工作、各自在采哪些文件、
    /// 一次性工作做到哪一步。
    ///
    /// 为什么必须由 agent 上报：网关知道自己**授权**了什么，但「真的在采哪些文件」只有 agent
    /// 知道（本机手工加的输入、暂停、某条来源今天接不接得了）。网关侧只存最近一份。
    /// `None` = 这台 agent 还没报过（旧版本 agent 不发这个字段）。
    #[serde(default)]
    pub local_work: Option<AgentLocalWork>,
    /// 本机**实际生效**的采集输出状态（见 [`AgentUplinkState`]）。
    ///
    /// 为什么必须由 agent 上报：网关知道自己**下发**了「启用 + 目标」，但不知道 agent
    /// **生效**成了什么 —— grant 可能还没拉到、可能被本机总闸拦住、可能目标连不上。
    /// 运维要回答的正是那句「这台为什么不上送」。
    ///
    /// `None` = 这台 agent 还没报过（旧版本 agent 不发这个字段）—— 落库后保持上一次的值
    /// （与 `local_work` 同口径）。
    #[serde(default)]
    pub uplink_state: Option<AgentUplinkState>,
    /// 本机**客户端证书**状态（mTLS）；还没有证书时 `None`。
    ///
    /// 为什么要 agent 上报：证书与到期时间只有本机知道（服务端在握手期就验完了，
    /// 而**过期证书根本进不来**）；而「哪些机器快到期 / 已过期需重装」正是运维要提前看到的
    /// （见 `docs/design/agent-identity-mtls.md` §5.5）。
    ///
    /// `None` = 这台 agent 还没报过 / 没证书 —— 落库后保持上一次的值（与其他可选字段同口径）。
    #[serde(default)]
    pub certificate_status: Option<AgentCertificateStatus>,
    /// 机器画像（机器名 / `node_id` / `machine_id` / 网卡地址）—— 注册表里「这是哪台机器」的展示来源。
    ///
    /// 为什么由状态上报带（而不是只靠注册）：**凭证书首触重建**登记时，机器画像全是空的
    /// （证书只承载稳定身份），原设计指望「后续状态上报补齐」，但那条通道以前没有这些字段 ——
    /// 于是经证书注册的机器在管理面永远只剩一个 ID。这里把注册时的 `HostProfile` 原样带上。
    ///
    /// `None` = 老版本 agentd 没带 —— 落库后保持上一次的值（与 `local_work` / `uplink_state` 同口径）。
    #[serde(default)]
    pub machine_profile: Option<HostProfile>,
}

/// agent 本地客户端证书状态（上报给网关，供页面/告警展示）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentCertificateStatus {
    /// 证书到期时刻（RFC3339）。
    pub not_after: String,
    /// 距到期的剩余秒数（已过期为负）。
    pub remaining_seconds: i64,
    /// `valid` / `renew_due` / `expired`。
    pub state: String,
    /// agent 本机**最近一次续签判定**的结果（§5.5）。
    ///
    /// `None` = 老版本 agentd 不发（或本机台账一时读不到）—— 网关落库时**保留上一次的值**
    /// （与本报告里 `local_work` / `uplink_state` 同一口径）。
    #[serde(default)]
    pub last_renewal: Option<AgentCredentialRenewal>,
}

/// agent 本机**最近一次续签判定**的结果（§5.5）。
///
/// 续签是后台动作，**不记录就等于静默**：agentd 把本地台账（`identity/renewal.json`）原样带上来，
/// 网关只存 / 展示，不重算。与 [`AgentCertificateStatus`] 同口径：`None` = 还没报过。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentCredentialRenewal {
    /// `not_due` / `renewed` / `failed` / `needs_reinstall` / `revoked`（与 agentd 本地台账同口径）。
    pub outcome: String,
    /// 本次判定时刻（RFC3339）。
    pub checked_at: String,
    /// 人读细节（失败原因 / 续到了什么时候…）；无内容时为空串。
    pub detail: String,
    /// 续签后证书的到期时刻（RFC3339）；无证书时为空串。
    pub not_after: String,
}

/// 状态上报的**回执**。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentStatusAck {
    pub agent_id: String,
    pub instance_id: String,
    pub acknowledged_at: String,
}

#[cfg(test)]
mod tests {
    use super::{AgentCertificateStatus, AgentStatusReport, AgentWorkState, AgentWorkStateChange};

    #[test]
    fn agent_work_state_uses_snake_case_on_the_wire() {
        assert_eq!(
            serde_json::to_string(&AgentWorkState::Paused).unwrap(),
            "\"paused\""
        );
        assert_eq!(
            serde_json::to_string(&AgentWorkState::Resumed).unwrap(),
            "\"resumed\""
        );
        let change = AgentWorkStateChange {
            input_id: "app".to_string(),
            state: AgentWorkState::Paused,
            reason: "spool_over_limit".to_string(),
            at: "2026-09-27T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&change).expect("encode");
        let back: AgentWorkStateChange = serde_json::from_str(&json).expect("decode");
        assert_eq!(back, change);

        // 与同族的 AgentStatusReport 一致：拒绝未知字段，避免字段漂移静默通过。
        let mutated = json.replacen('{', "{\"extra\":1,", 1);
        assert!(serde_json::from_str::<AgentWorkStateChange>(&mutated).is_err());
    }

    #[test]
    fn a_minimal_agent_status_report_decodes_and_extra_keys_fail() {
        // 旧版 agent 只发三个必填字段；其余全是 #[serde(default)]。
        let json = r#"{"agent_id":"a","instance_id":"i","version":"0.1.5"}"#;
        let report: AgentStatusReport = serde_json::from_str(json).expect("decode");
        assert_eq!(report.memory_bytes, None);
        assert_eq!(report.cpu_percent, None);
        assert_eq!(report.work_state_changes, None);
        assert_eq!(report.discovery_policy_version, None);
        assert!(report.local_work.is_none());
        assert!(report.uplink_state.is_none());
        assert!(report.certificate_status.is_none());
        // 机器画像同样是可选的：旧版 agent 不带 -> None（网关落库时保持上一次的值）。
        assert!(report.machine_profile.is_none());

        assert!(
            serde_json::from_str::<AgentStatusReport>(
                r#"{"agent_id":"a","instance_id":"i","version":"v","nope":1}"#
            )
            .is_err()
        );
    }

    /// 机器画像随状态上报带上时能如实解码（机器名 / node_id / 网卡地址）。
    #[test]
    fn an_agent_status_report_carries_the_machine_profile() {
        let json = r#"{
            "agent_id":"a","instance_id":"i","version":"0.1.24",
            "machine_profile":{
                "node_id":"node-1","hostname":"host-1","os":"linux","arch":"x86_64",
                "machine_id":"mid-1","cloud_instance_id":null,"k8s_node_uid":null,
                "ip_addresses":["en0 192.168.1.5/24","10.8.0.2"]
            }
        }"#;
        let report: AgentStatusReport = serde_json::from_str(json).expect("decode");
        let profile = report.machine_profile.expect("machine_profile present");
        assert_eq!(profile.hostname, "host-1");
        assert_eq!(profile.node_id, "node-1");
        assert_eq!(profile.machine_id, "mid-1");
        assert_eq!(
            profile.ip_addresses,
            vec!["en0 192.168.1.5/24".to_string(), "10.8.0.2".to_string()]
        );
    }

    /// 证书状态里的「最近一次续签」是可选的，且拒绝未知字段：
    /// 旧 agentd 少发它必须仍能解码（上线顺序不一，网关不能因此 400）。
    #[test]
    fn certificate_status_last_renewal_is_optional_and_round_trips() {
        let legacy =
            r#"{"not_after":"2026-11-04T00:00:00Z","remaining_seconds":100,"state":"valid"}"#;
        let status: AgentCertificateStatus = serde_json::from_str(legacy).expect("decode");
        assert!(status.last_renewal.is_none());

        let json = r#"{"not_after":"2026-11-04T00:00:00Z","remaining_seconds":100,"state":"valid",
            "last_renewal":{"outcome":"renewed","checked_at":"2026-10-08T00:00:00Z",
            "detail":"credential renewed","not_after":"2026-11-04T00:00:00Z"}}"#;
        let status: AgentCertificateStatus = serde_json::from_str(json).expect("decode");
        assert_eq!(
            status.last_renewal.as_ref().expect("renewal").outcome,
            "renewed"
        );

        let encoded = serde_json::to_string(&status).expect("encode");
        let back: AgentCertificateStatus = serde_json::from_str(&encoded).expect("decode");
        assert_eq!(back, status);

        // 字段漂移（`last_renewal` 里多出未知键）必须显形。
        let drifted = r#"{"not_after":"x","remaining_seconds":0,"state":"valid",
            "last_renewal":{"outcome":"renewed","checked_at":"t","detail":"","not_after":"",
            "extra":1}}"#;
        assert!(serde_json::from_str::<AgentCertificateStatus>(drifted).is_err());
    }
}
