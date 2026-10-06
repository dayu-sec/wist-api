//! `agent/discovery-policies:poll` seam —— **v1** 基线。
//!
//! 冻结基线：只做**加性**兼容不动它；非加性变更就新开 `v2`。
//! 约定见 `wist-design/doc/design/foundation/api-seam-inventory.md` §7。

use serde::{Deserialize, Serialize};

use wist_contracts::API_VERSION_V1;

use super::{DiscoveryAspectPolicy, DiscoveryAspectPolicySet};

/// 本版本的线上版本号（与路由 `/api/v1/…` 一致）。
pub const API_VERSION: &str = API_VERSION_V1;

pub const POLL_DISCOVERY_POLICIES_KIND: &str = "poll_discovery_policies";

/// agentd → 网关：拉取**发现方向策略表**（控制面，复用 agent 凭据）。
///
/// 为什么是「拉」而不是网关推：agentd 没有入站监听（那要开端口、要证书、要处理公网可达），
/// 而策略是**幂等内容**（声明式、可重复拉取，与 PollWork 同类）—— 拉一次就够，不必重放。
///
/// 为什么不需要 `wait_ms`（PollControlCommands 有）：那是长轮询指令流；策略表按版本变化，
/// 轮询周期由 agentd 自己控（它知道自己能承受多密）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PollDiscoveryPolicies {
    pub api_version: String,
    pub kind: String,
    pub agent_id: String,
    pub instance_id: String,
    pub requested_at: String,
}

/// 网关返回的策略表（对应模型 `Discovery.Probe.DiscoveryAspectPolicySet`）。
///
/// 带 `policy_version`：agentd 用它判断「这份与我手上的是不是同一版」，
/// 从而在版本未变时跳过重算与日志（而不是每次都重新应用一遍）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscoveryPoliciesReturned {
    pub policy_version: i64,
    pub published_at: String,
    pub policies: Vec<DiscoveryAspectPolicy>,
    pub returned_at: String,
}

impl DiscoveryPoliciesReturned {
    /// 从策略表组响应。
    ///
    /// 不回带 `agent_id`/`instance_id`（其它 ack 会带）：这不是「确认某次上报」，
    /// 而是「把当前版本的内容交给你」—— 身份由凭证本身表达，重复一份只会多一个会失配的字段。
    pub fn from_set(set: &DiscoveryAspectPolicySet, returned_at: String) -> Self {
        Self {
            policy_version: set.policy_version,
            published_at: set.published_at.clone(),
            policies: set.policies.clone(),
            returned_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovery_policies_returned_from_set_copies_the_versioned_table() {
        let set = DiscoveryAspectPolicySet::new(
            4,
            "2026-09-27T00:00:00Z".to_string(),
            vec![DiscoveryAspectPolicy {
                aspect: "host".to_string(),
                default_interval_seconds: 900,
                min_interval_seconds: 60,
                max_interval_seconds: 3600,
                baseline: true,
                enabled_by_default: true,
                platforms: vec!["macos".to_string(), "linux".to_string()],
                yields: "os/arch".to_string(),
            }],
        );
        let returned =
            DiscoveryPoliciesReturned::from_set(&set, "2026-09-27T00:00:01Z".to_string());
        assert_eq!(returned.policy_version, 4);
        assert_eq!(returned.policies, set.policies);
        assert_eq!(returned.returned_at, "2026-09-27T00:00:01Z");

        let json = serde_json::to_string(&returned).expect("encode");
        let back: DiscoveryPoliciesReturned = serde_json::from_str(&json).expect("decode");
        assert_eq!(back, returned);
    }

    #[test]
    fn poll_discovery_policies_round_trips_and_rejects_unknown_fields() {
        let poll = PollDiscoveryPolicies {
            api_version: API_VERSION_V1.to_string(),
            kind: POLL_DISCOVERY_POLICIES_KIND.to_string(),
            agent_id: "agent-1".to_string(),
            instance_id: "inst-1".to_string(),
            requested_at: "2026-09-27T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&poll).expect("encode");
        let back: PollDiscoveryPolicies = serde_json::from_str(&json).expect("decode");
        assert_eq!(back, poll);

        let mutated = json.replacen('{', "{\"extra\":1,", 1);
        assert!(serde_json::from_str::<PollDiscoveryPolicies>(&mutated).is_err());
    }
}
