//! `agent/uplink:poll` seam 报文：`PollAgentUplink`（拉取）与 `AgentUplinkGrant`（下发）。
//!
//! 「实际生效状态」`AgentUplinkState` **不是报文**，仍留在 `wist-contracts`；这里 re-export。

use serde::{Deserialize, Serialize};

// 报文引用的领域/状态类型仍留在 `wist-contracts`。
pub use wist_contracts::agent_uplink::AgentUplinkState;

/// agentd → 网关：拉取数据面上送启用的 envelope kind。
pub const POLL_AGENT_UPLINK_KIND: &str = "poll_agent_uplink";

/// agentd → 网关：拉取数据面上送启用。
///
/// 与 `wist_contracts::work::PollWork` 同形（同一套 agent 凭据、同一份实例标识），
/// 因为它是同一类「拉期望状态」的动作；只是期望状态的内容不同。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PollAgentUplink {
    pub api_version: String,
    pub kind: String,
    pub agent_id: String,
    pub instance_id: String,
    pub requested_at: String,
}

/// 网关 → agentd：数据面上送的当前期望状态。
///
/// 与 `wist_contracts::work::WorkGrant` 同类：描述的是「这个 Agent 的上送现在应当是什么样」这个
/// 领域事实（幂等、可重复拉取），而不是一次协议动作。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentUplinkGrant {
    /// 是否启用数据面上送的**主机内容**（日志 / 指标）。`false` = 待命。
    pub enabled: bool,
    /// 数据面地址。`enabled = true` 时给出即**覆盖**本机 `tcp.addr`；
    /// 缺省表示「沿用本机配置的目标」。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    pub granted_at: String,
}

impl AgentUplinkGrant {
    /// 待命：明确关掉。
    pub fn standby(granted_at: String) -> Self {
        Self {
            enabled: false,
            host: None,
            port: None,
            granted_at,
        }
    }

    /// 启用：带上目标。
    pub fn enabled_at(host: String, port: u16, granted_at: String) -> Self {
        Self {
            enabled: true,
            host: Some(host),
            port: Some(port),
            granted_at,
        }
    }

    /// 要覆盖的本机目标（`enabled` 且绑定齐了 `host`/`port` 才有）。
    ///
    /// `host` 会 `trim`：空白主机不是目标（把它当目标只会拼出连不上的 `" :9000"`）。
    pub fn target(&self) -> Option<(&str, u16)> {
        match (self.enabled, self.host.as_deref(), self.port) {
            (true, Some(host), Some(port)) => {
                let host = host.trim();
                (!host.is_empty()).then_some((host, port))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standby_round_trips_and_omits_absent_target() {
        // 待命帧不该把空目标序列化出去：`host`/`port` 缺失就是「沿用本机」的判据。
        let grant = AgentUplinkGrant::standby("2026-09-26T00:00:00Z".to_string());
        let json = serde_json::to_string(&grant).expect("encode");
        assert!(!json.contains("host"), "{json}");
        assert!(!json.contains("port"), "{json}");
        let back: AgentUplinkGrant = serde_json::from_str(&json).expect("decode");
        assert_eq!(back, grant);
        assert_eq!(back.target(), None);
    }

    #[test]
    fn an_enabled_grant_carries_the_target_to_override() {
        let grant = AgentUplinkGrant::enabled_at(
            "c-001.gateway.example".to_string(),
            9000,
            "2026-09-26T00:00:00Z".to_string(),
        );
        assert_eq!(grant.target(), Some(("c-001.gateway.example", 9000)));
        let json = serde_json::to_string(&grant).expect("encode");
        let back: AgentUplinkGrant = serde_json::from_str(&json).expect("decode");
        assert_eq!(back, grant);
    }

    #[test]
    fn an_enabled_grant_without_a_target_falls_back_to_the_local_kind() {
        let grant = AgentUplinkGrant {
            enabled: true,
            host: None,
            port: None,
            granted_at: "t".to_string(),
        };
        assert_eq!(grant.target(), None);

        // 空 host 也不能被当成目标：那只会拼出连不上的 ":9000"。
        let blank = AgentUplinkGrant {
            enabled: true,
            host: Some(String::new()),
            port: Some(9000),
            granted_at: "t".to_string(),
        };
        assert_eq!(blank.target(), None);

        // 只有空白的 host 同理。
        let whitespace = AgentUplinkGrant {
            enabled: true,
            host: Some("   ".to_string()),
            port: Some(9000),
            granted_at: "t".to_string(),
        };
        assert_eq!(whitespace.target(), None);

        // 前后带空白的真实主机要被裁成可用目标。
        let padded = AgentUplinkGrant {
            enabled: true,
            host: Some(" gw.example ".to_string()),
            port: Some(9000),
            granted_at: "t".to_string(),
        };
        assert_eq!(padded.target(), Some(("gw.example", 9000)));

        // 有 host 但缺 port 也不算目标。
        let host_only = AgentUplinkGrant {
            enabled: true,
            host: Some("gw.example".to_string()),
            port: None,
            granted_at: "t".to_string(),
        };
        assert_eq!(host_only.target(), None);

        // 未启用的 grant 即使目标齐全也不构成覆盖。
        let standby_with_target = AgentUplinkGrant {
            enabled: false,
            host: Some("gw.example".to_string()),
            port: Some(9000),
            granted_at: "t".to_string(),
        };
        assert_eq!(standby_with_target.target(), None);
    }

    #[test]
    fn a_newer_gateway_field_is_rejected_rather_than_ignored() {
        let json = r#"{"enabled":true,"granted_at":"t","extra":1}"#;
        assert!(serde_json::from_str::<AgentUplinkGrant>(json).is_err());
    }

    #[test]
    fn poll_round_trips_and_rejects_unknown_fields() {
        let poll = PollAgentUplink {
            api_version: wist_contracts::API_VERSION_V1.to_string(),
            kind: POLL_AGENT_UPLINK_KIND.to_string(),
            agent_id: "agent-1".to_string(),
            instance_id: "inst-1".to_string(),
            requested_at: "2026-09-26T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&poll).expect("encode");
        let back: PollAgentUplink = serde_json::from_str(&json).expect("decode");
        assert_eq!(back, poll);

        let bad = r#"{"api_version":"v1","kind":"poll_agent_uplink","agent_id":"a",
                      "instance_id":"i","requested_at":"t","extra":1}"#;
        assert!(serde_json::from_str::<PollAgentUplink>(bad).is_err());
    }
}
