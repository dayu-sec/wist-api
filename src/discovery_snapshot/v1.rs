//! `agent/discovery-snapshot:poll` seam —— **v1** 基线。
//!
//! 冻结基线：只做**加性**兼容不动它；非加性变更就新开 `v2`。
//! 约定见 `wist-design/doc/design/foundation/api-seam-inventory.md` §7。

use serde::{Deserialize, Serialize};

use wist_contracts::API_VERSION_V1;

/// 本版本的线上版本号（与路由 `/api/v1/…` 一致）。
pub const API_VERSION: &str = API_VERSION_V1;

pub const POLL_DISCOVERY_SNAPSHOT_KIND: &str = "poll_discovery_snapshot";

/// Agent 侧实际生效的「原文快照上送」配置。
///
/// 搭车在拉取请求里回传，**不是**一次独立的汇报动作：网关据此回答
/// 「配的是 1 小时，为什么这台实际还是 5 分钟」这类问题。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppliedDiscoverySnapshot {
    pub enabled: bool,
    pub interval_seconds: u64,
    /// `control_plane`（按网关下发）或 `local_config`（本机 `[discovery]`）。
    pub source: String,
}

impl AppliedDiscoverySnapshot {
    pub fn control_plane(&self) -> bool {
        self.source == APPLIED_SOURCE_CONTROL_PLANE
    }
}

/// `source` 的两个取值：控制面下发 / 本机配置。
pub const APPLIED_SOURCE_CONTROL_PLANE: &str = "control_plane";
pub const APPLIED_SOURCE_LOCAL_CONFIG: &str = "local_config";

/// agentd → 网关：拉取「原文发现快照上送」的控制面设置。
///
/// 与 [`crate::uplink::PollAgentUplink`] 同形：幂等、可重复拉取、没有 `wait_ms`。
/// 控制面改了设置，Agent 下一个轮询（≤30s）就换上新周期 —— 不需要重装，也不需要重启。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PollDiscoverySnapshot {
    pub api_version: String,
    pub kind: String,
    pub agent_id: String,
    pub instance_id: String,
    pub requested_at: String,
    /// 本机**当前实际生效**的值（旧 agentd 不发这个字段）。
    ///
    /// 为什么搭在这条请求上，而不是塞进 `agent/status`：status 报文也是
    /// `deny_unknown_fields`，加字段会让「新 agentd + 旧网关」解析失败 —— 那是
    /// 舰队级停摆。新端点两侧都是新的，顺带回传没有兼容风险。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied: Option<AppliedDiscoverySnapshot>,
}

/// 网关下发：**解析后**的快照上送设置（单机覆盖 > 全局设置）。
///
/// 网关**没有**任何设置时不下发这个对象（HTTP `204 No Content`）——
/// 「没有意见」与「明确要求用某个值」是两件事：后者会把 Agent 本机配置永久压掉，
/// 而本机配置是离线 / 旧网关 / 未入网时唯一的来源。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscoverySnapshotGrant {
    pub enabled: bool,
    pub interval_seconds: u64,
    /// 这份值来自哪一层：`agent`（单机覆盖）或 `global`（本网关全局设置）。
    pub source: String,
    pub granted_at: String,
}

/// `grant.source` 的两个取值。
pub const GRANT_SOURCE_AGENT: &str = "agent";
pub const GRANT_SOURCE_GLOBAL: &str = "global";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poll_round_trips_and_rejects_unknown_fields() {
        let poll = PollDiscoverySnapshot {
            api_version: API_VERSION_V1.to_string(),
            kind: POLL_DISCOVERY_SNAPSHOT_KIND.to_string(),
            agent_id: "agent-1".to_string(),
            instance_id: "inst-1".to_string(),
            requested_at: "2026-10-26T00:00:00Z".to_string(),
            applied: Some(AppliedDiscoverySnapshot {
                enabled: true,
                interval_seconds: 300,
                source: APPLIED_SOURCE_LOCAL_CONFIG.to_string(),
            }),
        };
        let json = serde_json::to_string(&poll).expect("encode");
        let back: PollDiscoverySnapshot = serde_json::from_str(&json).expect("decode");
        assert_eq!(back, poll);
        assert!(!back.applied.as_ref().expect("applied").control_plane());

        let mutated = json.replacen('{', "{\"extra\":1,", 1);
        assert!(serde_json::from_str::<PollDiscoverySnapshot>(&mutated).is_err());
    }

    /// `applied` 必须可省：旧 agentd 不带它，解析不能失败。
    #[test]
    fn applied_is_optional_on_the_wire() {
        let json = r#"{"api_version":"v1","kind":"poll_discovery_snapshot","agent_id":"a","instance_id":"i","requested_at":"t"}"#;
        let poll: PollDiscoverySnapshot = serde_json::from_str(json).expect("decode");
        assert_eq!(poll.applied, None);
    }

    #[test]
    fn grant_round_trips() {
        let grant = DiscoverySnapshotGrant {
            enabled: true,
            interval_seconds: 3600,
            source: GRANT_SOURCE_AGENT.to_string(),
            granted_at: "2026-10-26T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&grant).expect("encode");
        let back: DiscoverySnapshotGrant = serde_json::from_str(&json).expect("decode");
        assert_eq!(back, grant);
    }
}
