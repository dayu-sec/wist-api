//! `agent/metrics-labels` seam —— **v1** 基线。
//!
//! 冻结基线：只做**加性**兼容不动它；非加性变更就新开 `v2`。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use wist_contracts::API_VERSION_V1;

/// 本版本的线上版本号（与路由 `/api/v1/…` 一致）。
pub const API_VERSION: &str = API_VERSION_V1;

/// agent 拉取选择类 label 的请求类型。
pub const POLL_AGENT_METRICS_LABELS_KIND: &str = "poll_agent_metrics_labels";

/// agent 拉取选择类 label 的响应类型。
pub const AGENT_METRICS_LABELS_KIND: &str = "agent_metrics_labels";

/// 管理面查看生效值的响应类型。
pub const ADMIN_METRICS_LABELS_KIND: &str = "admin_agent_metrics_labels";

/// 管理面设置手工覆盖的请求类型。
pub const SET_AGENT_METRICS_LABELS_KIND: &str = "set_agent_metrics_labels";

/// 每个键的来源词表（`AgentMetricsLabels.sources` 的取值）。
pub mod label_source {
    /// 网关部署配置（`env` / `cluster` 等）。
    pub const DEFAULT: &str = "default";
    /// 网关按事实推断（`app`）。
    pub const INFERRED: &str = "inferred";
    /// 管理面手工覆盖。
    pub const MANUAL: &str = "manual";
}

/// agentd → 网关：`POST /api/v1/agent/metrics-labels:poll` 的请求。
///
/// 为何是 `POST` + `:poll` 而不是 `GET`：与 agent 面其余拉取端点（`work:poll` /
/// `uplink:poll` / `discovery-policies:poll`）保持同一形状 —— 身份（`agent_id`）在**请求体**里，
/// 不进出 URL（不落访问日志、不落代理日志），且 `:poll` 明确它是幂等拉取而不是一次汇报。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PollAgentMetricsLabels {
    pub api_version: String,
    pub kind: String,
    pub agent_id: String,
    pub instance_id: String,
    /// 只是回声：与其它 `:poll` 端点一致（client 时间在这条路径上没有语义）。
    #[serde(default)]
    pub requested_at: String,
}

impl PollAgentMetricsLabels {
    pub fn new(agent_id: &str, instance_id: &str, requested_at: &str) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            kind: POLL_AGENT_METRICS_LABELS_KIND.to_string(),
            agent_id: agent_id.to_string(),
            instance_id: instance_id.to_string(),
            requested_at: requested_at.to_string(),
        }
    }
}

/// agentd → 网关：`POST /api/v1/agent/metrics-labels:poll` 的响应。
///
/// `revision` 是网关侧该 agent 这份值的版本（手工覆盖每次写入 +1）。agent 在
/// **版本变化或内容变化**时替换本地缓存：只比版本会让「改了部署默认、各 agent 的覆盖为空
/// （版本仍是 0）」这种变更永远推不下去。空表是合法状态（= 当前没有可用值），agent 不得自己猜。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentMetricsLabels {
    pub api_version: String,
    pub kind: String,
    pub agent_id: String,
    pub revision: u64,
    pub labels: BTreeMap<String, String>,
    /// 每个键的来源（见 [`label_source`]）。只用于展示与排障，不影响取值。
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub sources: BTreeMap<String, String>,
    pub updated_at: String,
}

/// 管理面：查看某台 agent 的生效值、来源与手工覆盖。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentMetricsLabelsView {
    pub api_version: String,
    pub kind: String,
    pub agent_id: String,
    pub revision: u64,
    /// 合并后的生效值。
    pub labels: BTreeMap<String, String>,
    /// 每个键的来源（见 [`label_source`]）。
    pub sources: BTreeMap<String, String>,
    /// 仅手工覆盖的部分（清除时用）。
    pub overrides: BTreeMap<String, String>,
    pub updated_at: String,
}

/// 管理面：设置手工覆盖。
///
/// 逐键生效：`Some(value)` = 设置/覆盖；`None`（JSON `null`）= 删除该键的手工覆盖，
/// 回落到推断或部署默认。未出现在表里的键保持原状。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetAgentMetricsLabelsRequest {
    pub api_version: String,
    pub kind: String,
    pub overrides: BTreeMap<String, Option<String>>,
}

impl SetAgentMetricsLabelsRequest {
    pub fn new(overrides: BTreeMap<String, Option<String>>) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            kind: SET_AGENT_METRICS_LABELS_KIND.to_string(),
            overrides,
        }
    }
}

impl AgentMetricsLabels {
    pub fn new(
        agent_id: &str,
        revision: u64,
        labels: BTreeMap<String, String>,
        sources: BTreeMap<String, String>,
        updated_at: &str,
    ) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            kind: AGENT_METRICS_LABELS_KIND.to_string(),
            agent_id: agent_id.to_string(),
            revision,
            labels,
            sources,
            updated_at: updated_at.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_labels() -> BTreeMap<String, String> {
        BTreeMap::from([
            ("app".to_string(), "kafka".to_string()),
            ("cluster".to_string(), "kafka-prod".to_string()),
            ("env".to_string(), "prod".to_string()),
        ])
    }

    #[test]
    fn agent_response_round_trips() {
        let payload = AgentMetricsLabels::new(
            "agent-1",
            7,
            sample_labels(),
            BTreeMap::from([
                ("app".to_string(), label_source::MANUAL.to_string()),
                ("cluster".to_string(), label_source::DEFAULT.to_string()),
                ("env".to_string(), label_source::DEFAULT.to_string()),
            ]),
            "2026-11-03T07:00:00Z",
        );
        let json = serde_json::to_string(&payload).expect("serialize");
        let parsed: AgentMetricsLabels = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, payload);
        assert_eq!(parsed.kind, AGENT_METRICS_LABELS_KIND);
    }

    #[test]
    fn empty_labels_are_a_valid_state() {
        let payload = AgentMetricsLabels::new("agent-1", 0, BTreeMap::new(), BTreeMap::new(), "t");
        let json = serde_json::to_string(&payload).expect("serialize");
        let parsed: AgentMetricsLabels = serde_json::from_str(&json).expect("deserialize");
        assert!(parsed.labels.is_empty());
    }

    #[test]
    fn overrides_can_remove_a_key_with_null() {
        let request = SetAgentMetricsLabelsRequest::new(BTreeMap::from([
            ("app".to_string(), Some("kafka".to_string())),
            ("role".to_string(), None),
        ]));
        let json = serde_json::to_string(&request).expect("serialize");
        assert!(json.contains("\"role\":null"));
        let parsed: SetAgentMetricsLabelsRequest =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.overrides.get("role"), Some(&None));
        assert_eq!(
            parsed
                .overrides
                .get("app")
                .and_then(|value| value.as_deref()),
            Some("kafka")
        );
    }

    #[test]
    fn the_poll_request_round_trips() {
        let payload = PollAgentMetricsLabels::new("agent-1", "inst-1", "2026-11-03T07:00:00Z");
        let json = serde_json::to_string(&payload).expect("serialize");
        let parsed: PollAgentMetricsLabels = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, payload);
        assert_eq!(parsed.kind, POLL_AGENT_METRICS_LABELS_KIND);
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let json = r#"{"api_version":"v1","kind":"agent_metrics_labels","agent_id":"a","revision":1,"labels":{},"updated_at":"t","extra":1}"#;
        assert!(serde_json::from_str::<AgentMetricsLabels>(json).is_err());
    }
}
