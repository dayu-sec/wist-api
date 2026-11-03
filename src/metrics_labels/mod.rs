//! `agent/metrics-labels` seam（v1）：选择类 label 的下发与管理面报文。
//!
//! 与指标数据本身无关：这里只搬「哪些键值要贴到该 agent 的指标序列上」。
//! 值的来源与优先级（手工 > 推断 > 部署默认）见
//! `wist-agentd/docs/design/metrics-selection-labels.md`。
//!
//! 本 seam 的报文只用基本类型组装，不内嵌共享领域类型；版本并存约定同
//! [`crate::enrollment`]（见 `wist-design/doc/design/foundation/api-seam-inventory.md` §7）。
//!
//! ## 「加性兼容」在这里是什么、不是什么
//!
//! 报文都是 `deny_unknown_fields`，所以**给现有报文加字段不是加性变更** ——
//! 旧对端会整份解析失败（agent 侧的后果是「保留旧缓存、再也不更新」，而且只在 5 分钟
//! 一次的日志里留一行）。真正加性的做法是**新开端点 / 新开 `v2`**，与会话里其它 seam 一致。
//! 想扩这个 seam 的字段时先看这句话，别直接往结构体里加。

pub mod rules;
pub mod v1;
pub use rules::{KEYS, LabelError, MAX_LABELS, validate_labels};
pub use v1::*;

/// 当前线上版本。新代码从这里取版本口径；并存期由路由/协商决定。
pub const CURRENT: &str = v1::API_VERSION;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_matches_the_live_wire_version() {
        assert_eq!(CURRENT, "v1");
        assert_eq!(CURRENT, v1::API_VERSION);
    }
}
