//! 选择类 label 的**准入规则**（键闭集 / 值文法 / `env` 闭集 / `app` 依赖 / 数量上限）。
//!
//! ## 为什么放在 seam crate
//!
//! 同一份规则两侧都要判：网关在**写**的时候拦（配置与手工覆盖），agentd 在**收**的时候再判一遍
//! （纵深防御：篡改/写坏的响应不该被贴到序列上）。两处各写一份必然漂移 —— 漂移的后果是
//! 「网关放行的值 agentd 拒收」，表现为指标**静默没有 label**，最难查。
//!
//! 规则正文见 `wist-agentd/docs/design/metrics-data-spec.md` §3.4；本模块只做可执行化，
//! 不引入新语义。

use std::collections::BTreeMap;

/// 键闭集（§3.4）。不在此列的键一律拒 —— label 键没有「自定义扩展」这一说。
pub const KEYS: &[&str] = &[
    "app", "env", "cluster", "role", "shard", "tier", "region", "zone",
];

/// `env` 闭集。环境是要被聚合的轴，不允许自由发挥（`production` / `Prod` 会静默分裂）。
pub const ENVS: &[&str] = &["prod", "staging", "dev", "test"];

/// `role` 词表（§3.4 的基础词 + §13 各系统实际用到的静态角色）。
///
/// 为什么是一份**基础**词表而不是每 app 自由扩展：`role` 会出现在查询与告警表达式里，
/// 让一份策展文件引入任意词等于让 `{role=…}` 静默分裂（`broker` / `brokers` 各成一条线）。
/// 新增角色 = 改这里 + 规范 §3.4 登记（一次代码变更），代价是一次发布，
/// 换来的是「列出来的词一定有意义」。
pub const ROLES: &[&str] = &[
    "arbiter",
    "broker",
    "config",
    "control-plane",
    "controller",
    "coordinator",
    "data",
    "ingest",
    "keeper",
    "learner",
    "mongos",
    "observer",
    "pd",
    "primary",
    "proxy",
    "replica",
    "sentinel",
    "tidb",
    "tikv",
    "worker",
];

/// `tier` 闭集。
pub const TIERS: &[&str] = &["hot", "warm", "cold"];

/// 一条序列上同时出现的选择类 label 上限（§3.4 规则 7）。
pub const MAX_LABELS: usize = 6;

/// 值的最大长度（§12.2：小写、≤ 32 字符）。
pub const MAX_VALUE_LEN: usize = 32;

/// 准入失败的原因。`code` 是稳定词表，会出现在管理面 400 与日志里。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LabelError {
    /// 超过 [`MAX_LABELS`]。
    TooMany { got: usize },
    /// 键不在闭集。
    UnknownKey(String),
    /// 值不合法（空 / 超长 / 含非法字符）。
    BadValue { key: String, value: String },
    /// `env` 不在闭集。
    BadEnv(String),
    /// `role` / `tier` 不在词表。
    BadVocabulary { key: String, value: String },
    /// 有 `role` / `shard` / `tier` 却没有 `app`（§3.4 规则 6：没有登记表可校验，只允许
    /// `env` / `cluster` / `region` / `zone`）。
    OrphanKey(String),
}

impl LabelError {
    /// 稳定错误码（日志 / API `detail` 用）。
    pub fn code(&self) -> &'static str {
        match self {
            LabelError::TooMany { .. } => "too_many_metric_labels",
            LabelError::UnknownKey(_) => "unknown_metric_label",
            LabelError::BadValue { .. } => "invalid_metric_label_value",
            LabelError::BadEnv(_) => "invalid_metric_env",
            LabelError::BadVocabulary { .. } => "invalid_metric_label_value",
            LabelError::OrphanKey(_) => "metric_label_requires_app",
        }
    }
}

impl std::fmt::Display for LabelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LabelError::TooMany { got } => {
                write!(f, "选择类 label 最多 {MAX_LABELS} 对（给了 {got} 对）")
            }
            LabelError::UnknownKey(key) => {
                write!(f, "`{key}` 不在键闭集（{}）", KEYS.join(" / "))
            }
            LabelError::BadValue { key, value } => write!(
                f,
                "`{key}` 的值 {value:?} 不合法：只允许小写字母 / 数字 / `-` / `_` / `.`，且不超过 {MAX_VALUE_LEN} 字符"
            ),
            LabelError::BadEnv(value) => {
                write!(f, "`env` 的值 {value:?} 不在闭集（{}）", ENVS.join(" / "))
            }
            LabelError::BadVocabulary { key, value } => {
                write!(f, "`{key}` 的值 {value:?} 不在该键的词表里")
            }
            LabelError::OrphanKey(key) => {
                write!(f, "`{key}` 需要先有 `app`（没有登记表时无法校验键组合）")
            }
        }
    }
}

/// 值文法：非空、≤ [`MAX_VALUE_LEN`]、只允许小写字母 / 数字 / `-` / `_` / `.`。
///
/// 故意不接受大写与空格：选择类 label 会进查询与告警表达式的引号里，大小写混用会让
/// `{env="Prod"}` 与 `{env="prod"}` 变成两条不同的序列，而人眼几乎看不出差别。
pub fn is_valid_value(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_VALUE_LEN
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '-' | '_' | '.'))
}

/// 准入一组选择类 label（键 / 值 / `env` / `role` / `tier` / `app` 依赖 / 数量）。
///
/// 空表合法：等于「这台机器没有可用的选择类值」，调用方不得补默认值。
pub fn validate_labels(labels: &BTreeMap<String, String>) -> Result<(), LabelError> {
    if labels.len() > MAX_LABELS {
        return Err(LabelError::TooMany { got: labels.len() });
    }
    for (key, value) in labels {
        if !KEYS.contains(&key.as_str()) {
            return Err(LabelError::UnknownKey(key.clone()));
        }
        if !is_valid_value(value) {
            return Err(LabelError::BadValue {
                key: key.clone(),
                value: value.clone(),
            });
        }
        match key.as_str() {
            "env" if !ENVS.contains(&value.as_str()) => {
                return Err(LabelError::BadEnv(value.clone()));
            }
            "role" if !ROLES.contains(&value.as_str()) => {
                return Err(LabelError::BadVocabulary {
                    key: key.clone(),
                    value: value.clone(),
                });
            }
            "tier" if !TIERS.contains(&value.as_str()) => {
                return Err(LabelError::BadVocabulary {
                    key: key.clone(),
                    value: value.clone(),
                });
            }
            _ => {}
        }
    }
    // 键组合：`role` / `shard` / `tier` 的含义随 `app` 变（§3.4 规则 6 / §13），
    // 没有 `app` 就没有登记表可校验 —— 宁可拒，也不写一个无法解释的值。
    if !labels.contains_key("app") {
        for key in ["role", "shard", "tier"] {
            if labels.contains_key(key) {
                return Err(LabelError::OrphanKey(key.to_string()));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect()
    }

    #[test]
    fn a_minimal_pair_is_valid() {
        assert_eq!(
            validate_labels(&labels(&[("env", "prod"), ("cluster", "kafka-prod")])),
            Ok(())
        );
    }

    #[test]
    fn an_empty_set_is_valid() {
        assert_eq!(validate_labels(&BTreeMap::new()), Ok(()));
    }

    #[test]
    fn unknown_keys_are_rejected() {
        assert_eq!(
            validate_labels(&labels(&[("service", "kafka")])),
            Err(LabelError::UnknownKey("service".to_string()))
        );
    }

    #[test]
    fn env_must_come_from_the_closed_set() {
        assert_eq!(
            validate_labels(&labels(&[("env", "production")])),
            Err(LabelError::BadEnv("production".to_string()))
        );
    }

    #[test]
    fn values_are_lowercase_and_length_limited() {
        assert!(validate_labels(&labels(&[("cluster", "Kafka-Prod")])).is_err());
        assert!(validate_labels(&labels(&[("cluster", "kafka prod")])).is_err());
        assert!(validate_labels(&labels(&[("cluster", "")])).is_err());
        assert!(validate_labels(&labels(&[("cluster", &"a".repeat(33))])).is_err());
        assert!(validate_labels(&labels(&[("cluster", &"a".repeat(32))])).is_ok());
        assert!(validate_labels(&labels(&[("cluster", "kafka-prod_1.a")])).is_ok());
    }

    #[test]
    fn role_and_tier_are_vocabulary_bounded() {
        assert!(validate_labels(&labels(&[("app", "kafka"), ("role", "broker")])).is_ok());
        assert_eq!(
            validate_labels(&labels(&[("app", "kafka"), ("role", "leader")])),
            Err(LabelError::BadVocabulary {
                key: "role".to_string(),
                value: "leader".to_string()
            })
        );
        assert!(validate_labels(&labels(&[("app", "es"), ("tier", "hot")])).is_ok());
        assert!(validate_labels(&labels(&[("app", "es"), ("tier", "frozen")])).is_err());
    }

    #[test]
    fn role_without_app_is_rejected() {
        assert_eq!(
            validate_labels(&labels(&[("role", "broker")])),
            Err(LabelError::OrphanKey("role".to_string()))
        );
        // 没有 `app` 时 `region` / `zone` 仍然可用：它们不随 app 变。
        assert!(validate_labels(&labels(&[("env", "dev"), ("region", "cn-north")])).is_ok());
    }

    #[test]
    fn at_most_six_pairs() {
        let six = labels(&[
            ("app", "kafka"),
            ("env", "prod"),
            ("cluster", "c"),
            ("role", "broker"),
            ("region", "cn-north"),
            ("zone", "az1"),
        ]);
        assert!(validate_labels(&six).is_ok());
        let mut seven = six;
        seven.insert("shard".to_string(), "0".to_string());
        assert_eq!(validate_labels(&seven), Err(LabelError::TooMany { got: 7 }));
    }
}
