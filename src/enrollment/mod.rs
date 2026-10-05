//! `agent/enroll` 与 `agent/credentials:renew` 两条 edge seam 的报文。
//!
//! 这是**跨进程 seam 报文**的唯一一份定义：网关（接收端）与 agentd（发送端）都 `use` 这里，
//! 不再各自复制。报文引用的**领域类型**（`HostProfile` / `AgentIdentity` / `CredentialBundle`
//! / `InitialConfig` / `PolicyBinding`）是两侧共同底座，仍留在 `wist-contracts`；这里 re-export，
//! 调用方可以从 `wist_api::enrollment` 一处取齐整条 seam。
//!
//! ## 版本并存约定（v1 / v2 …）
//!
//! 一条 seam 的**每一版**是独立一组 `{route, req, resp}`：各自一个版本子模块（`v1`、`v2`…）
//! 与各自的 handler。**不**用一个结构体 + `api_version` 分支硬扛两版。
//!
//! - **非加性（breaking）变更** → 新开一版 + 新路由（`/api/v2/…`）；老版本子模块**冻结、只增不删**。
//! - **纯加字段** → 才在同版本内演进，并按 `upgrade-order.md` §4 **接收端先升**
//!   （其余情况接收端多为 `deny_unknown_fields`，非加性一律走新版本）。
//! - `CURRENT` 指出新代码该用的版本；**路由并存 / 版本协商**留待真正出现 v2 时再接。
//!
//! 约定详见 `wist-design/doc/design/foundation/api-seam-inventory.md` §7。

/// 版本无关的**领域类型**（被多条 seam 复用），来自 `wist-contracts`。
pub use wist_contracts::enrollment::{
    AgentIdentity, AgentIdentityStatus, CredentialBundle, HostProfile, InitialConfig, PolicyBinding,
};

pub mod v1;
pub use v1::*;

/// 当前线上版本。新代码从这里取版本口径；并存期由路由/协商决定。
pub const CURRENT: &str = v1::API_VERSION;

#[cfg(test)]
mod tests {
    use super::*;

    /// `CURRENT` 必须与 v1 报文里填的 `api_version` 一致（不至两处各说各话）。
    #[test]
    fn current_matches_the_live_wire_version() {
        let request = v1::EnrollmentRequest::new(
            "t".to_string(),
            "cr".to_string(),
            "csr".to_string(),
            HostProfile {
                node_id: "n".to_string(),
                hostname: "h".to_string(),
                os: "linux".to_string(),
                arch: "x86_64".to_string(),
                machine_id: "m".to_string(),
                cloud_instance_id: None,
                k8s_node_uid: None,
                ip_addresses: vec![],
            },
            "caps".to_string(),
            "2026-09-28T00:00:00Z".to_string(),
        );
        assert_eq!(request.api_version, CURRENT);
    }

    /// 编译期身份断言：本模块 re-export 的领域类型**就是** `wist-contracts` 的那几个
    /// （若有人另抄一份，下面的函数体将类型不匹配而编译失败）。
    #[test]
    fn domain_types_are_the_contracts_ones() {
        #[allow(dead_code)]
        fn assert_identity() {
            fn host(v: wist_contracts::enrollment::HostProfile) -> HostProfile {
                v
            }
            fn agent(v: wist_contracts::enrollment::AgentIdentity) -> AgentIdentity {
                v
            }
            fn agent_status(
                v: wist_contracts::enrollment::AgentIdentityStatus,
            ) -> AgentIdentityStatus {
                v
            }
            fn cred(v: wist_contracts::enrollment::CredentialBundle) -> CredentialBundle {
                v
            }
            fn init(v: wist_contracts::enrollment::InitialConfig) -> InitialConfig {
                v
            }
            fn policy(v: wist_contracts::enrollment::PolicyBinding) -> PolicyBinding {
                v
            }
            let _ = (host, agent, agent_status, cred, init, policy);
        }
        assert_identity();
    }
}
