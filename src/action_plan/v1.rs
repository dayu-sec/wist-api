//! `agent/action-plan` seam —— **v1** 基线。
//!
//! 冻结基线：只做**加性**兼容不动它；非加性变更就新开 `v2`。
//! 约定见 `wist-design/doc/design/foundation/api-seam-inventory.md` §7。

use serde::{Deserialize, Serialize};

use wist_contracts::API_VERSION_V1;

use super::ActionPlan;

/// 本版本的线上版本号（与路由 `/api/v1/…` 一致）。
pub const API_VERSION: &str = API_VERSION_V1;

pub const DISPATCH_ACTION_PLAN_KIND: &str = "dispatch_action_plan";
pub const ACTION_PLAN_ACK_KIND: &str = "action_plan_ack";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispatchActionPlan {
    pub api_version: String,
    pub kind: String,
    pub dispatch_id: String,
    pub plan: ActionPlan,
}

impl DispatchActionPlan {
    pub fn new(dispatch_id: String, plan: ActionPlan) -> Self {
        Self {
            api_version: API_VERSION_V1.to_string(),
            kind: DISPATCH_ACTION_PLAN_KIND.to_string(),
            dispatch_id,
            plan,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionPlanAck {
    pub api_version: String,
    pub kind: String,
    pub dispatch_id: String,
    pub action_id: String,
    pub plan_digest: String,
    pub agent_id: String,
    pub instance_id: String,
    pub execution_id: Option<String>,
    pub ack_status: AckStatus,
    pub reason_code: Option<String>,
    pub reason_message: Option<String>,
    pub queue_position: Option<u64>,
    pub received_at: String,
    pub acknowledged_at: String,
}

impl ActionPlanAck {
    pub fn builder(
        dispatch_id: String,
        action_id: String,
        ack_status: AckStatus,
    ) -> ActionPlanAckBuilder {
        ActionPlanAckBuilder {
            dispatch_id,
            action_id,
            plan_digest: String::new(),
            agent_id: String::new(),
            instance_id: String::new(),
            execution_id: None,
            ack_status,
            reason_code: None,
            reason_message: None,
            queue_position: None,
            received_at: String::new(),
            acknowledged_at: String::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        dispatch_id: String,
        action_id: String,
        plan_digest: String,
        agent_id: String,
        instance_id: String,
        execution_id: Option<String>,
        ack_status: AckStatus,
        received_at: String,
        acknowledged_at: String,
    ) -> Self {
        Self::builder(dispatch_id, action_id, ack_status)
            .plan_digest(plan_digest)
            .agent_id(agent_id)
            .instance_id(instance_id)
            .execution_id(execution_id)
            .received_at(received_at)
            .acknowledged_at(acknowledged_at)
            .build()
    }
}

#[derive(Debug, Clone)]
pub struct ActionPlanAckBuilder {
    dispatch_id: String,
    action_id: String,
    plan_digest: String,
    agent_id: String,
    instance_id: String,
    execution_id: Option<String>,
    ack_status: AckStatus,
    reason_code: Option<String>,
    reason_message: Option<String>,
    queue_position: Option<u64>,
    received_at: String,
    acknowledged_at: String,
}

impl ActionPlanAckBuilder {
    pub fn plan_digest(mut self, plan_digest: String) -> Self {
        self.plan_digest = plan_digest;
        self
    }

    pub fn agent_id(mut self, agent_id: String) -> Self {
        self.agent_id = agent_id;
        self
    }

    pub fn instance_id(mut self, instance_id: String) -> Self {
        self.instance_id = instance_id;
        self
    }

    pub fn execution_id(mut self, execution_id: Option<String>) -> Self {
        self.execution_id = execution_id;
        self
    }

    pub fn reason_code(mut self, reason_code: Option<String>) -> Self {
        self.reason_code = reason_code;
        self
    }

    pub fn reason_message(mut self, reason_message: Option<String>) -> Self {
        self.reason_message = reason_message;
        self
    }

    pub fn queue_position(mut self, queue_position: Option<u64>) -> Self {
        self.queue_position = queue_position;
        self
    }

    pub fn received_at(mut self, received_at: String) -> Self {
        self.received_at = received_at;
        self
    }

    pub fn acknowledged_at(mut self, acknowledged_at: String) -> Self {
        self.acknowledged_at = acknowledged_at;
        self
    }

    pub fn build(self) -> ActionPlanAck {
        ActionPlanAck {
            api_version: API_VERSION_V1.to_string(),
            kind: ACTION_PLAN_ACK_KIND.to_string(),
            dispatch_id: self.dispatch_id,
            action_id: self.action_id,
            plan_digest: self.plan_digest,
            agent_id: self.agent_id,
            instance_id: self.instance_id,
            execution_id: self.execution_id,
            ack_status: self.ack_status,
            reason_code: self.reason_code,
            reason_message: self.reason_message,
            queue_position: self.queue_position,
            received_at: self.received_at,
            acknowledged_at: self.acknowledged_at,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AckStatus {
    #[serde(rename = "accepted")]
    Accepted,
    #[serde(rename = "rejected")]
    Rejected,
    #[serde(rename = "queued")]
    Queued,
    #[serde(rename = "duplicate")]
    Duplicate,
    #[serde(rename = "stale")]
    Stale,
    #[serde(rename = "busy")]
    Busy,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan() -> ActionPlan {
        serde_json::from_str(
            r#"{"api_version":"v1","kind":"action_plan",
                "meta":{"action_id":"act-1","request_id":"req-1","template_id":null,
                        "tenant_id":"t","environment_id":"e","plan_version":1,
                        "compiled_at":"2026-09-27T00:00:00Z","expires_at":"2026-09-28T00:00:00Z"},
                "target":{"agent_id":"agent-1","instance_id":null,"node_id":"n","host_name":null,
                          "platform":"macos","arch":"arm64","selectors":{}},
                "constraints":{"risk_level":"R1","approval_ref":null,"approval_mode":"not_required",
                               "requested_by":"admin","reason":null,"max_total_duration_ms":1000,
                               "step_timeout_default_ms":500,"execution_profile":"default",
                               "required_capabilities":[]},
                "program":{"entry":"s1","steps":[{"id":"s1","kind":"invoke","op":"shell"}]}}"#,
        )
        .expect("plan")
    }

    #[test]
    fn dispatch_action_plan_new_stamps_the_envelope() {
        let dispatch = DispatchActionPlan::new("disp-1".to_string(), plan());
        assert_eq!(dispatch.api_version, API_VERSION_V1);
        assert_eq!(dispatch.kind, DISPATCH_ACTION_PLAN_KIND);

        let json = serde_json::to_string(&dispatch).expect("encode");
        let back: DispatchActionPlan = serde_json::from_str(&json).expect("decode");
        assert_eq!(back, dispatch);
    }

    #[test]
    fn action_plan_ack_builder_and_new_stamp_the_envelope() {
        let built =
            ActionPlanAck::builder("disp-1".to_string(), "act-1".to_string(), AckStatus::Queued)
                .plan_digest("sha256:plan".to_string())
                .agent_id("agent-1".to_string())
                .instance_id("inst-1".to_string())
                .queue_position(Some(3))
                .received_at("2026-09-27T00:00:00Z".to_string())
                .acknowledged_at("2026-09-27T00:00:01Z".to_string())
                .build();
        assert_eq!(built.api_version, API_VERSION_V1);
        assert_eq!(built.kind, ACTION_PLAN_ACK_KIND);
        assert_eq!(built.queue_position, Some(3));
        assert_eq!(built.reason_code, None);

        let constructed = ActionPlanAck::new(
            "disp-1".to_string(),
            "act-1".to_string(),
            "sha256:plan".to_string(),
            "agent-1".to_string(),
            "inst-1".to_string(),
            None,
            AckStatus::Accepted,
            "2026-09-27T00:00:00Z".to_string(),
            "2026-09-27T00:00:01Z".to_string(),
        );
        assert_eq!(constructed.kind, ACTION_PLAN_ACK_KIND);

        let json = serde_json::to_string(&constructed).expect("encode");
        let back: ActionPlanAck = serde_json::from_str(&json).expect("decode");
        assert_eq!(back, constructed);
    }

    #[test]
    fn ack_status_uses_the_wire_names_and_rejects_unknown_variants() {
        for (status, name) in [
            (AckStatus::Accepted, "accepted"),
            (AckStatus::Rejected, "rejected"),
            (AckStatus::Queued, "queued"),
            (AckStatus::Duplicate, "duplicate"),
            (AckStatus::Stale, "stale"),
            (AckStatus::Busy, "busy"),
        ] {
            assert_eq!(
                serde_json::to_string(&status).unwrap(),
                format!("\"{name}\"")
            );
        }
        assert!(serde_json::from_str::<AckStatus>("\"unknown\"").is_err());
    }
}
