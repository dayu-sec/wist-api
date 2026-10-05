//! `agent/work:*` seam 报文 —— **v1** 基线。
//!
//! 冻结基线：只做**加性**兼容不动它；非加性变更就新开 `v2`。

use serde::{Deserialize, Serialize};

use super::{OneShotWork, StandingWork};

/// 本版本的线上版本号（与路由 `/api/v1/…` 一致）。
pub const API_VERSION: &str = wist_contracts::API_VERSION_V1;

/// agentd → 网关：拉取工作授权快照的 envelope kind。
pub const POLL_WORK_KIND: &str = "poll_work";
/// agentd → 网关：确认收到工作的 envelope kind。
pub const ACK_WORK_KIND: &str = "ack_work";
/// agentd → 网关：上报一次性工作执行结果的 envelope kind。
pub const REPORT_WORK_RESULT_KIND: &str = "report_work_result";

/// 工作授权快照：常驻工作的当前生效版本 + 未了结的一次性工作。
///
/// 幂等、可重复拉取；`sequence` 只用来让 agentd 判断「这份跟我手上的有没有变」，
/// **不承担「指令重放」的语义**（那是控制指令流的事）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkGrant {
    pub agent_id: String,
    /// 每个面一条。
    #[serde(default)]
    pub standing: Vec<StandingWork>,
    #[serde(default)]
    pub one_shot: Vec<OneShotWork>,
    /// 授权序号（单调递增，每次授权/撤回/暂停/继续都 +1）。
    pub sequence: i64,
    pub granted_at: String,
}

/// agentd → 网关：拉取工作授权快照。
///
/// 带 `last_seen_sequence`（与本机手上那份的序号），网关可以据此在没变化时短路；
/// 带 `wait_ms` 是为了允许将来的长轮询（现在是立即返回，字段先留着，免得改协议）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PollWork {
    pub api_version: String,
    pub kind: String,
    pub agent_id: String,
    pub instance_id: String,
    pub last_seen_sequence: i64,
    pub wait_ms: i64,
    pub requested_at: String,
}

/// agentd → 网关：确认收到某份工作。
///
/// 常驻工作在 `plan_version` 变化后**也必须**确认：网关据此判断「期望的版本真到了吗」，
/// 一直没确认的就是漂移。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AckWork {
    pub api_version: String,
    pub kind: String,
    pub agent_id: String,
    pub instance_id: String,
    pub work_id: String,
    pub plan_version: i64,
    pub acknowledged_at: String,
}

/// 网关对 [`AckWork`] 的回应。
///
/// 是**工作域的结构**而不是协议消息（与 [`WorkGrant`] 同类）：它描述的是
/// 「工作已被确认」这个领域事实，也要能被用例当成 outcome 引用。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkAccepted {
    pub work_id: String,
    /// accepted | stale | unknown。
    pub status: String,
    pub accepted_at: String,
}

/// agentd → 网关：上报一次性工作的**执行结果**（进度与终态）。
///
/// 与 [`AckWork`] 的分工：确认回答「我收到了」，本消息回答「我做得怎么样了」。
/// 两者分开是因为它们的**失败代价不同**：确认丢了只是页面晚一拍，结果丢了则意味着
/// 「一件改变机器状态的活做完了，而控制面永远不知道它成没成」。
///
/// `status` 取值见 `wist_contracts::work::AGENT_REPORTABLE_WORK_STATUSES`。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportWorkResult {
    pub api_version: String,
    pub kind: String,
    pub agent_id: String,
    pub instance_id: String,
    pub work_id: String,
    pub status: String,
    /// 人看的说明：失败原因**原样带上**。
    #[serde(default)]
    pub detail: String,
    pub reported_at: String,
}

/// 网关对 [`ReportWorkResult`] 的回应。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkResultAccepted {
    pub work_id: String,
    /// accepted | stale | unknown。
    ///
    /// `stale` = 这件活已经到终态了（被撤回、超期，或已经报过终态），后到的结果**不覆盖**它。
    pub status: String,
    pub accepted_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn standing() -> StandingWork {
        StandingWork {
            work_id: "work-a".to_string(),
            agent_id: "agent-1".to_string(),
            family: "LoginSession".to_string(),
            spec: "unit-a,unit-b".to_string(),
            catalog_version: 1,
            proposal_id: None,
            plan_version: 2,
            effective_from: "2026-09-23T00:00:00Z".to_string(),
            status: "active".to_string(),
            updated_by: "admin".to_string(),
            updated_at: "2026-09-23T00:00:00Z".to_string(),
        }
    }

    fn one_shot() -> OneShotWork {
        OneShotWork {
            work_id: "work-1".to_string(),
            agent_id: "agent-1".to_string(),
            action: "upgrade".to_string(),
            spec: "0.1.4".to_string(),
            scheduled_at: "2026-09-23T00:00:00Z".to_string(),
            deadline_at: "2026-09-24T00:00:00Z".to_string(),
            timeout_seconds: 600,
            interruptible: true,
            status: "running".to_string(),
            paused_at: None,
            paused_total_seconds: 0,
            current_step: None,
            completed_steps: vec![],
            attempt: 0,
            issued_by: "admin".to_string(),
            issued_at: "2026-09-23T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn grant_round_trips_with_serde() {
        let grant = WorkGrant {
            agent_id: "agent-1".to_string(),
            standing: vec![standing()],
            one_shot: vec![one_shot()],
            sequence: 7,
            granted_at: "2026-09-23T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&grant).expect("serialize");
        let decoded: WorkGrant = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, grant);
    }

    #[test]
    fn grant_omits_absent_optional_fields_and_still_decodes() {
        let json = r#"{"agent_id":"a","standing":[],"one_shot":[],"sequence":0,
                      "granted_at":"t"}"#;
        let grant: WorkGrant = serde_json::from_str(json).expect("deserialize");
        assert!(grant.standing.is_empty());
        assert!(grant.one_shot.is_empty());
    }

    #[test]
    fn grant_rejects_unknown_fields() {
        // 两侧各自演进时，多出来的字段必须是响亮的错误。
        let json = r#"{"agent_id":"a","standing":[],"one_shot":[],"sequence":0,
                      "granted_at":"t","extra":1}"#;
        assert!(serde_json::from_str::<WorkGrant>(json).is_err());
    }
}
