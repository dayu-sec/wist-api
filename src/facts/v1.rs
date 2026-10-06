//! `agent/facts` seam —— **v1** 基线。
//!
//! 冻结基线：只做**加性**兼容不动它；非加性变更就新开 `v2`。
//! 约定见 `wist-design/doc/design/foundation/api-seam-inventory.md` §7。

use serde::{Deserialize, Serialize};

use wist_contracts::API_VERSION_V1;

/// 本版本的线上版本号（与路由 `/api/v1/…` 一致）。
pub const API_VERSION: &str = API_VERSION_V1;

pub const REPORT_AGENT_FACT_SUMMARY_KIND: &str = "report_agent_fact_summary";

/// 事实上报的确认状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactSummaryAckStatus {
    /// 已入库。
    Accepted,
    /// 内容未变（网关按**自算**摘要判定）：只刷留痕，未改内容、未重复计分。
    Duplicate,
    /// envelope 或身份非法。
    Rejected,
}

/// agentd → 网关的事实**摘要**上报（控制面）。
///
/// 与数据面上的原文快照（`ReportDiscoverySnapshot`）分工不同，**不是同一条路**：
/// 摘要只服务用途推断（网关侧按规则表算），去重后 10~30 KB，走已认证的控制面；
/// 原文快照一台几百 KB，走数据面给中心做资产整理。所以网关只接摘要。
///
/// 幂等键是内容摘要，不是 `revision`（后者每轮 refresh 无条件 +1）。
///
/// agentd **无条件周期全量**上报，判重归网关：网关用
/// `wist_contracts::fact_summary::FactContent::content_digest` 从收到的内容**自己算**摘要，
/// 以此判重。`content_digest` 字段因此只是 agent 的**声明**：
/// 与网关算出来的不一致时会记 `FactDigestMismatch` 告警（可能只是版本偏差，**不拒收**）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportAgentFactSummary {
    pub api_version: String,
    pub kind: String,
    pub report_id: String,
    pub agent_id: String,
    pub instance_id: String,
    /// agent 侧声明的内容摘要。**不是**判重键：网关从下列内容字段自算，此值只作版本偏差的金丝雀。
    pub content_digest: String,
    /// 仅留痕：快照 revision 每轮 refresh 无条件 +1，网关不据它判重。
    pub revision: i64,
    /// 仅留痕：观察到的事实属于哪一刻（快照生成时间）。
    pub observed_at: String,
    pub os: String,
    pub arch: String,
    /// 仅留痕：去重前的进程条数（去重会毁掉基数，留一个原始计数备查），不进摘要。
    pub process_count: i64,
    /// 去重后的进程可执行标识。注意两边不同源：
    /// macOS 是 `ps -axo comm=` 给的完整路径，Linux 是 `/proc/{pid}/comm`（只有 basename）。
    pub process_executables: Vec<String>,
    /// 已装包名（仅 linux；macOS 侧待定）。
    pub packages: Vec<String>,
    pub listen_ports: Vec<String>,
    // ── 以下三个是**留痕/展示**字段：**不进内容摘要**，也不参与判重 ──
    //
    // 为什么不进摘要：摘要回答的是「内容变了没有」（幂等键与用途判据的输入）。
    // 机器名、IP 会因 DHCP/改名而变，但它们不影响「这台机器是干什么用的」——
    // 放进摘要会让每次换网就触发一次重报与重算。所以它们只用于展示与追溯。
    // 也正因如此，`fact-v1` 的字段集**没变**，不需要 bump 版本、不需要强制重报。
    /// 主机标识（发现里 `host` 方向的 `host.id`）。
    #[serde(default)]
    pub host_id: String,
    /// 主机名（`host.name`）。
    #[serde(default)]
    pub host_name: String,
    /// 网卡地址（每块网卡一条，形如 `en0 192.168.1.5/24`）。
    #[serde(default)]
    pub network_addresses: Vec<String>,
    pub reported_at: String,
}

impl ReportAgentFactSummary {
    #[allow(clippy::too_many_arguments)]
    pub fn new_agent_facts(
        report_id: String,
        agent_id: String,
        instance_id: String,
        content_digest: String,
        revision: i64,
        observed_at: String,
        os: String,
        arch: String,
        process_count: i64,
        process_executables: Vec<String>,
        packages: Vec<String>,
        listen_ports: Vec<String>,
        reported_at: String,
    ) -> Self {
        Self {
            api_version: API_VERSION_V1.to_string(),
            kind: REPORT_AGENT_FACT_SUMMARY_KIND.to_string(),
            report_id,
            agent_id,
            instance_id,
            content_digest,
            revision,
            observed_at,
            os,
            arch,
            process_count,
            process_executables,
            packages,
            listen_ports,
            host_id: String::new(),
            host_name: String::new(),
            network_addresses: Vec::new(),
            reported_at,
        }
    }

    /// 补上**留痕/展示**字段（不参与内容摘要与判重）。
    ///
    /// 为什么另开一个方法而不是给构造函数再加三个参数：那个函数已经有 13 个位置参数，
    /// 再加就是 16 个 —— 调用方只需错一次顺序，就会把主机名传成 os、把端口传成包名，
    /// 而这类错**不会报错**（都是 String/Vec<String>），只会静默写错数据。
    pub fn with_display(
        mut self,
        host_id: String,
        host_name: String,
        network_addresses: Vec<String>,
    ) -> Self {
        self.host_id = host_id;
        self.host_name = host_name;
        self.network_addresses = network_addresses;
        self
    }
}

/// Gateway 对事实上报的确认响应（对应模型 `FactSummaryAccepted`）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FactSummaryAccepted {
    pub report_id: String,
    pub agent_id: String,
    pub content_digest: String,
    pub ack_status: FactSummaryAckStatus,
    /// 幂等命中（`duplicate`）时仍回带已存的建议，Agent 侧不必再问一次。
    pub suggestion_id: Option<String>,
    pub received_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fact_summary_ack_status_uses_snake_case_and_rejects_unknown() {
        assert_eq!(
            serde_json::to_string(&FactSummaryAckStatus::Duplicate).unwrap(),
            "\"duplicate\""
        );
        assert!(serde_json::from_str::<FactSummaryAckStatus>("\"nope\"").is_err());
    }

    #[test]
    fn new_agent_facts_defaults_display_fields_then_with_display_fills_them() {
        let summary = ReportAgentFactSummary::new_agent_facts(
            "fact_1".to_string(),
            "agent-1".to_string(),
            "inst-1".to_string(),
            "fact-v1:sha256:abc".to_string(),
            7,
            "2026-09-27T00:00:00Z".to_string(),
            "macos".to_string(),
            "arm64".to_string(),
            3,
            vec!["/usr/bin/a".to_string()],
            Vec::new(),
            vec!["443".to_string()],
            "2026-09-27T00:00:01Z".to_string(),
        );
        assert_eq!(summary.kind, REPORT_AGENT_FACT_SUMMARY_KIND);
        assert!(summary.host_id.is_empty());
        assert!(summary.network_addresses.is_empty());

        let with_display = summary.with_display(
            "host-id".to_string(),
            "host-name".to_string(),
            vec!["en0 10.0.0.1/24".to_string()],
        );
        assert_eq!(with_display.host_id, "host-id");
        assert_eq!(with_display.host_name, "host-name");

        let json = serde_json::to_string(&with_display).expect("encode");
        let back: ReportAgentFactSummary = serde_json::from_str(&json).expect("decode");
        assert_eq!(back, with_display);
    }
}
