//! `agent/status` seam —— **v1** 基线。
//!
//! agentd 周期性上报自身状态（CPU/内存/工作状态/发现策略版本/证书状态/机器画像…），网关收下并落库。
//! 冻结基线：只做**加性**兼容不动它；需要非加性变更就新开 `v2`。
//! 约定见 `wist-design/doc/design/foundation/api-seam-inventory.md` §7。

use serde::{Deserialize, Serialize};

use wist_contracts::capability_report::{InvestigationCapabilities, LogsCapabilities};

use super::{AgentLocalWork, AgentUplinkState, HostProfile};

/// 本版本的线上版本号（与路由 `/api/v1/…` 一致）。
pub const API_VERSION: &str = wist_contracts::API_VERSION_V1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentWorkState {
    Paused,
    Resumed,
}

/// 工作状态变化（非告警、非失败）：暂停/恢复各上报一次。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentWorkStateChange {
    pub input_id: String,
    pub state: AgentWorkState,
    pub reason: String,
    pub at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentStatusReport {
    pub agent_id: String,
    pub instance_id: String,
    pub version: String,
    /// Own resident-set size in bytes reported by the agent.
    #[serde(default)]
    pub memory_bytes: Option<u64>,
    /// Agent 进程自身 CPU 占用，**单核口径**（100% = 占满一个核；多线程进程可 >100）。
    ///
    /// 它只统计 agent 进程**自己**的 CPU 时间（`getrusage(RUSAGE_SELF)`），不含它拉起的子进程。
    #[serde(default)]
    pub cpu_percent: Option<f64>,
    /// Agent 所在机器的**逻辑核数**（`available_parallelism`）。
    ///
    /// 为什么必须和 `cpu_percent` 同一份上报带上来：`cpu_percent` 是单核口径，
    /// 而运维看图时真正常问的是「这台机器被它占了百分之几」——那是 `cpu_percent / 核数`。
    /// 少了核数，右侧那个数既算不出来、也无法复核（4 核上的 13% 和 64 核上的 13% 完全不是一回事）。
    ///
    /// 为什么让 agent 报原始值、而不是它自己算好整机占比：沿用本仓已有的取舍
    /// （与 `discovery_policy_version`、事实摘要 digest 同理）——agent 只交**原始事实**
    /// （自己的 CPU 时间、自己的核数），换算只留一处，在网关。agent 自算的派生值
    /// 一旦算法退化，下游没有任何一层能发现。
    #[serde(default)]
    pub cpu_cores: Option<u32>,
    /// Measured round-trip latency to the admin control plane in milliseconds.
    #[serde(default)]
    pub admin_latency_ms: Option<u64>,
    /// 自上次上报以来的工作状态变化（paused/resumed），非告警、非失败。
    #[serde(default)]
    pub work_state_changes: Option<Vec<AgentWorkStateChange>>,
    /// 本机**实际生效**的发现方向策略版本；`None` = 还没拿到策略表（在用内建默认周期）。
    ///
    /// 为什么必须由 agent 上报、而不是网关自己记账：网关知道自己**发布**了哪一版，
    /// 但不知道某台机器**拉到并应用**了哪一版 —— 拉取可能失败、可能还没到轮询节拍、
    /// 也可能拿到后被夹取。而运维要回答的正是那句「我改了策略，哪些机器还没生效」。
    #[serde(default)]
    pub discovery_policy_version: Option<i64>,
    /// 本机**工作内容视图**（`state/work.json` 的子集）：我手里有哪些工作、各自在采哪些文件、
    /// 一次性工作做到哪一步。
    ///
    /// 为什么必须由 agent 上报：网关知道自己**授权**了什么，但「真的在采哪些文件」只有 agent
    /// 知道（本机手工加的输入、暂停、某条来源今天接不接得了）。网关侧只存最近一份。
    /// `None` = 这台 agent 还没报过（旧版本 agent 不发这个字段）。
    #[serde(default)]
    pub local_work: Option<AgentLocalWork>,
    /// 本机**实际生效**的采集输出状态（见 [`AgentUplinkState`]）。
    ///
    /// 为什么必须由 agent 上报：网关知道自己**下发**了「启用 + 目标」，但不知道 agent
    /// **生效**成了什么 —— grant 可能还没拉到、可能被本机总闸拦住、可能目标连不上。
    /// 运维要回答的正是那句「这台为什么不上送」。
    ///
    /// `None` = 这台 agent 还没报过（旧版本 agent 不发这个字段）—— 落库后保持上一次的值
    /// （与 `local_work` 同口径）。
    #[serde(default)]
    pub uplink_state: Option<AgentUplinkState>,
    /// 本机**客户端证书**状态（mTLS）；还没有证书时 `None`。
    ///
    /// 为什么要 agent 上报：证书与到期时间只有本机知道（服务端在握手期就验完了，
    /// 而**过期证书根本进不来**）；而「哪些机器快到期 / 已过期需重装」正是运维要提前看到的
    /// （见 `docs/design/agent-identity-mtls.md` §5.5）。
    ///
    /// `None` = 这台 agent 还没报过 / 没证书 —— 落库后保持上一次的值（与其他可选字段同口径）。
    #[serde(default)]
    pub certificate_status: Option<AgentCertificateStatus>,
    /// 机器画像（机器名 / `node_id` / `machine_id` / 网卡地址）—— 注册表里「这是哪台机器」的展示来源。
    ///
    /// 为什么由状态上报带（而不是只靠注册）：**凭证书首触重建**登记时，机器画像全是空的
    /// （证书只承载稳定身份），原设计指望「后续状态上报补齐」，但那条通道以前没有这些字段 ——
    /// 于是经证书注册的机器在管理面永远只剩一个 ID。这里把注册时的 `HostProfile` 原样带上。
    ///
    /// `None` = 老版本 agentd 没带 —— 落库后保持上一次的值（与 `local_work` / `uplink_state` 同口径）。
    #[serde(default)]
    pub machine_profile: Option<HostProfile>,
    /// 本机**能承接的日志形态 / 导出器**（D7 能力协商）。
    ///
    /// 为什么要 agent 报、而不是网关按 `version` 推：版本号只能回答“新旧”，回答不了
    /// “这一版在**这个环境**里真能不能采”（导出器依赖的二进制可能没装、单层 glob 支持是
    /// 一个具体能力而不是一个版本）。而回滚场景里最容易出错的就是“新目录 + 回滚后的 agent” ——
    /// 网关必须看**它自己说了什么**。
    ///
    /// `None` = 旧版本 agentd 没报：网关按“未知”处理 —— 依赖新能力（glob / 新导出器）的单元
    /// **不算可采**，宁可少派也不派下去报 unsupported。
    #[serde(default)]
    pub logs_capabilities: Option<LogsCapabilities>,
    /// 本机**调查能力**（见 [`InvestigationCapabilities`]）：能跑哪些调查面、是不是 root。
    ///
    /// 为什么要 agent 报、而不是网关按版本推：调查是**一次性只读操作**，网关必须在
    /// **入队前**就知道这台机器认不认识这个面 —— 否则会把命令发下去换一个 `unsupported`，
    /// 管理面上只会显示成「命令失败」，而真正原因是「这台机器不会 / 没权限」。
    ///
    /// `None` = 旧版本 agentd 没报：网关按「未知」处理 —— **只允许**它自己确认支持的面
    /// （目前没有），宁可让菜单置灰，也不发一条注定失败的调查。
    #[serde(default)]
    pub investigation_capabilities: Option<InvestigationCapabilities>,
    /// 本机对网关下发的**一次性命令**的处理结果（见 [`AgentCommandAck`]）。
    ///
    /// 空 = 本次没有要确认的命令，字段不出现。**加性字段**：本结构是
    /// `deny_unknown_fields`，所以部署顺序必须 **gateway 先于 agentd**（与本文件其它
    /// 新增字段同一条规则，见 `CHANGELOG` 的升级提示）。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub command_acks: Vec<AgentCommandAck>,
}

/// agent 本地客户端证书状态（上报给网关，供页面/告警展示）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentCertificateStatus {
    /// 证书到期时刻（RFC3339）。
    pub not_after: String,
    /// 距到期的剩余秒数（已过期为负）。
    pub remaining_seconds: i64,
    /// `valid` / `renew_due` / `expired`。
    pub state: String,
    /// agent 本机**最近一次续签判定**的结果（§5.5）。
    ///
    /// `None` = 老版本 agentd 不发（或本机台账一时读不到）—— 网关落库时**保留上一次的值**
    /// （与本报告里 `local_work` / `uplink_state` 同一口径）。
    #[serde(default)]
    pub last_renewal: Option<AgentCredentialRenewal>,
}

/// agent 本机**最近一次续签判定**的结果（§5.5）。
///
/// 续签是后台动作，**不记录就等于静默**：agentd 把本地台账（`identity/renewal.json`）原样带上来，
/// 网关只存 / 展示，不重算。与 [`AgentCertificateStatus`] 同口径：`None` = 还没报过。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentCredentialRenewal {
    /// `not_due` / `renewed` / `failed` / `needs_reinstall` / `revoked`（与 agentd 本地台账同口径）。
    pub outcome: String,
    /// 本次判定时刻（RFC3339）。
    pub checked_at: String,
    /// 人读细节（失败原因 / 续到了什么时候…）；无内容时为空串。
    pub detail: String,
    /// 续签后证书的到期时刻（RFC3339）；无证书时为空串。
    pub not_after: String,
}

/// 状态上报的**回执**。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentStatusAck {
    pub agent_id: String,
    pub instance_id: String,
    pub acknowledged_at: String,
    /// 网关下发的待执行命令；没有命令时字段不出现。
    ///
    /// 为什么放在回执里：agent 没有入站监听面，状态上报（3s）是唯一由网关**回话**的通道。
    /// 这里放的是**通用命令信封**（`kind` + 可选参数），而不是每条命令一个字段 ——
    /// 新增一种命令只加一个 kind 词表值，不再改报文形状。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub commands: Vec<AgentCommand>,
}

/// 网关下发给 agent 的一次性命令。
///
/// 与「期望状态」（工作 / 上送 / 策略）刻意分开：那些是可重拉的幂等快照，
/// 这里是**一次执行**的动作，有明确的 `command_id`、结果与去重语义。
///
/// `command_id` 是去重键：agent 把处理过的命令写进本地台账，网关在没有收到
/// `command_acks` 时会重发同一条；agent 见到同一个 id 不重复跑，只补结果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentCommand {
    pub command_id: String,
    /// 网关受理时刻（墙钟毫秒）：agent 用它判过期。
    pub requested_at_ms: i64,
    /// 命令词表（见 [`command_kind`]）。**开放字符串**：未知值由 agent 回
    /// `unsupported`，而不是让整份回执解析失败。
    pub kind: String,
    /// 命令参数（JSON 文本；空 = 无参数）。不同 `kind` 自己解析，未知字段不进入本层。
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub params: String,
}

/// agent 对一条命令的处理结果（随下一次状态上报回传）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentCommandAck {
    pub command_id: String,
    /// 见 [`command_outcome`]。
    pub outcome: String,
    /// 人读细节（失败原因 / 探针统计…）；无内容时省略。
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub detail: String,
}

/// 命令词表（`AgentCommand.kind`）。新增命令只在这里加一个值 + 一个执行分支。
pub mod command_kind {
    /// 立刻重采一次并上报事实（见 agentd `refresh-facts`）。
    pub const REFRESH_FACTS: &str = "refresh_facts";
    /// 一次性只读调查（见 agentd `investigate`）：当场查一个问题并给证据表。
    ///
    /// `params` 是题目自己的 JSON（`topic` + 题目选项）；不认识的值由 agent 回 `unsupported`。
    /// 完整报告**不走这里**：ack 只回结论摘要，报告本体走数据面 ` IVSREP: ` 帧。
    pub const INVESTIGATE: &str = "investigate";
}

/// 命令结果词表（`AgentCommandAck.outcome`）。
pub mod command_outcome {
    /// 执行完成，效果已生效（重采：事实已上报）。
    pub const SUCCEEDED: &str = "succeeded";
    /// 执行了，但没达到目标（重采：事实上报失败）。
    pub const FAILED: &str = "failed";
    /// 执行了，但本机没有可用的上送通道（重采：输出是 `file`）。
    pub const NO_UPLINK: &str = "no_uplink";
    /// 没执行：本地最小间隔未到。
    pub const THROTTLED: &str = "throttled";
    /// 没执行：命令过期（网关下发后很久才被拿到）。
    pub const STALE: &str = "stale";
    /// 不认识这种 `kind`（旧 agent 收到新命令）。
    pub const UNSUPPORTED: &str = "unsupported";
    /// **仅网关侧**：未执行的旧命令被同 kind 的新命令覆盖。agent 不会回这个值。
    pub const SUPERSEDED: &str = "superseded";
    /// **仅网关侧**：命令超过待执行窗口（旧 agent 不认、或长期离线），网关不再下发。
    pub const EXPIRED: &str = "expired";
}

#[cfg(test)]
mod tests {
    use super::{
        AgentCertificateStatus, AgentCommandAck, AgentStatusAck, AgentStatusReport, AgentWorkState,
        AgentWorkStateChange, command_outcome,
    };

    #[test]
    fn agent_work_state_uses_snake_case_on_the_wire() {
        assert_eq!(
            serde_json::to_string(&AgentWorkState::Paused).unwrap(),
            "\"paused\""
        );
        assert_eq!(
            serde_json::to_string(&AgentWorkState::Resumed).unwrap(),
            "\"resumed\""
        );
        let change = AgentWorkStateChange {
            input_id: "app".to_string(),
            state: AgentWorkState::Paused,
            reason: "spool_over_limit".to_string(),
            at: "2026-09-27T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&change).expect("encode");
        let back: AgentWorkStateChange = serde_json::from_str(&json).expect("decode");
        assert_eq!(back, change);

        // 与同族的 AgentStatusReport 一致：拒绝未知字段，避免字段漂移静默通过。
        let mutated = json.replacen('{', "{\"extra\":1,", 1);
        assert!(serde_json::from_str::<AgentWorkStateChange>(&mutated).is_err());
    }

    #[test]
    fn a_minimal_agent_status_report_decodes_and_extra_keys_fail() {
        // 旧版 agent 只发三个必填字段；其余全是 #[serde(default)]。
        let json = r#"{"agent_id":"a","instance_id":"i","version":"0.1.5"}"#;
        let report: AgentStatusReport = serde_json::from_str(json).expect("decode");
        assert_eq!(report.memory_bytes, None);
        assert_eq!(report.cpu_percent, None);
        assert_eq!(report.work_state_changes, None);
        assert_eq!(report.discovery_policy_version, None);
        assert!(report.local_work.is_none());
        assert!(report.uplink_state.is_none());
        assert!(report.certificate_status.is_none());
        // 机器画像同样是可选的：旧版 agent 不带 -> None（网关落库时保持上一次的值）。
        assert!(report.machine_profile.is_none());
        // 命令结果同样可选（旧 agent 不带 = 没有要确认的命令）。
        assert!(report.command_acks.is_empty());

        assert!(
            serde_json::from_str::<AgentStatusReport>(
                r#"{"agent_id":"a","instance_id":"i","version":"v","nope":1}"#
            )
            .is_err()
        );
    }

    /// 机器画像随状态上报带上时能如实解码（机器名 / node_id / 网卡地址）。
    #[test]
    fn an_agent_status_report_carries_the_machine_profile() {
        let json = r#"{
            "agent_id":"a","instance_id":"i","version":"0.1.24",
            "machine_profile":{
                "node_id":"node-1","hostname":"host-1","os":"linux","arch":"x86_64",
                "machine_id":"mid-1","cloud_instance_id":null,"k8s_node_uid":null,
                "ip_addresses":["en0 192.168.1.5/24","10.8.0.2"]
            }
        }"#;
        let report: AgentStatusReport = serde_json::from_str(json).expect("decode");
        let profile = report.machine_profile.expect("machine_profile present");
        assert_eq!(profile.hostname, "host-1");
        assert_eq!(profile.node_id, "node-1");
        assert_eq!(profile.machine_id, "mid-1");
        assert_eq!(
            profile.ip_addresses,
            vec!["en0 192.168.1.5/24".to_string(), "10.8.0.2".to_string()]
        );
    }

    /// 证书状态里的「最近一次续签」是可选的，且拒绝未知字段：
    /// 旧 agentd 少发它必须仍能解码（上线顺序不一，网关不能因此 400）。
    #[test]
    fn certificate_status_last_renewal_is_optional_and_round_trips() {
        let legacy =
            r#"{"not_after":"2026-11-04T00:00:00Z","remaining_seconds":100,"state":"valid"}"#;
        let status: AgentCertificateStatus = serde_json::from_str(legacy).expect("decode");
        assert!(status.last_renewal.is_none());

        let json = r#"{"not_after":"2026-11-04T00:00:00Z","remaining_seconds":100,"state":"valid",
            "last_renewal":{"outcome":"renewed","checked_at":"2026-10-08T00:00:00Z",
            "detail":"credential renewed","not_after":"2026-11-04T00:00:00Z"}}"#;
        let status: AgentCertificateStatus = serde_json::from_str(json).expect("decode");
        assert_eq!(
            status.last_renewal.as_ref().expect("renewal").outcome,
            "renewed"
        );

        let encoded = serde_json::to_string(&status).expect("encode");
        let back: AgentCertificateStatus = serde_json::from_str(&encoded).expect("decode");
        assert_eq!(back, status);

        // 字段漂移（`last_renewal` 里多出未知键）必须显形。
        let drifted = r#"{"not_after":"x","remaining_seconds":0,"state":"valid",
            "last_renewal":{"outcome":"renewed","checked_at":"t","detail":"","not_after":"",
            "extra":1}}"#;
        assert!(serde_json::from_str::<AgentCertificateStatus>(drifted).is_err());
    }

    /// D7：能力清单可选；旧 agentd 不带时必须是 `None`（网关据此按“未知”处理）。
    #[test]
    fn logs_capabilities_are_optional_and_round_trip() {
        let legacy = r#"{"agent_id":"a","instance_id":"i","version":"0.8.0"}"#;
        let report: AgentStatusReport = serde_json::from_str(legacy).expect("decode");
        assert!(report.logs_capabilities.is_none());

        let json = r#"{"agent_id":"a","instance_id":"i","version":"0.9.0",
            "logs_capabilities":{
                "file_inputs":["absolute-path","glob"],
                "multiline_modes":["none","indented"],
                "exporters":["journalctl-kernel"]
            }}"#;
        let report: AgentStatusReport = serde_json::from_str(json).expect("decode");
        let logs = report.logs_capabilities.expect("logs_capabilities");
        assert_eq!(logs.exporters, vec!["journalctl-kernel".to_string()]);
        assert!(logs.file_inputs.contains(&"glob".to_string()));

        // 字段漂移仍要显形（与同族结构一致）。
        let drifted = r#"{"agent_id":"a","instance_id":"i","version":"v",
            "logs_capabilities":{"file_inputs":[],"nope":1}}"#;
        assert!(serde_json::from_str::<AgentStatusReport>(drifted).is_err());
    }

    /// 回执里可以带一批通用命令；旧网关不带这个字段时仍然是合法回执。
    #[test]
    fn a_status_ack_carries_optional_commands() {
        let legacy = r#"{"agent_id":"a","instance_id":"i","acknowledged_at":"t"}"#;
        let ack: AgentStatusAck = serde_json::from_str(legacy).expect("decode");
        assert!(ack.commands.is_empty());

        let json = r#"{"agent_id":"a","instance_id":"i","acknowledged_at":"t",
            "commands":[{"command_id":"c-1","requested_at_ms":1700000000000,
            "kind":"refresh_facts"}]}"#;
        let ack: AgentStatusAck = serde_json::from_str(json).expect("decode");
        assert_eq!(ack.commands.len(), 1);
        let command = &ack.commands[0];
        assert_eq!(command.command_id, "c-1");
        assert_eq!(command.requested_at_ms, 1_700_000_000_000);
        assert_eq!(command.kind, "refresh_facts");
        assert!(command.params.is_empty());

        // 没有命令时字段不出现：旧 agent（若未来开始解析回执）不会被空数组干扰。
        let encoded = serde_json::to_string(&AgentStatusAck {
            agent_id: "a".to_string(),
            instance_id: "i".to_string(),
            acknowledged_at: "t".to_string(),
            commands: Vec::new(),
        })
        .expect("encode");
        assert!(!encoded.contains("commands"));

        // 命令里的字段漂移要显形。
        let drifted = r#"{"agent_id":"a","instance_id":"i","acknowledged_at":"t",
            "commands":[{"command_id":"c-1","requested_at_ms":1,
            "kind":"refresh_facts","nope":1}]}"#;
        assert!(serde_json::from_str::<AgentStatusAck>(drifted).is_err());
    }

    /// 未知 kind 是**合法字符串**（旧 agent 要靠它回 `unsupported`），不是解析错误。
    #[test]
    fn an_unknown_command_kind_still_decodes() {
        let json = r#"{"agent_id":"a","instance_id":"i","acknowledged_at":"t",
            "commands":[{"command_id":"c-9","requested_at_ms":1,"kind":"future_command"}]}"#;
        let ack: AgentStatusAck = serde_json::from_str(json).expect("decode");
        assert_eq!(ack.commands[0].kind, "future_command");
    }

    /// 新 agent 报告里的命令结果可选；旧 agent 不带时字段不出现。
    #[test]
    fn an_agent_status_report_can_carry_command_results() {
        let legacy = r#"{"agent_id":"a","instance_id":"i","version":"0.9.0"}"#;
        let report: AgentStatusReport = serde_json::from_str(legacy).expect("decode");
        assert!(report.command_acks.is_empty());

        let json = r#"{"agent_id":"a","instance_id":"i","version":"0.9.11",
            "command_acks":[{"command_id":"c-1","outcome":"succeeded",
            "detail":"probes=6 failed=0"}]}"#;
        let report: AgentStatusReport = serde_json::from_str(json).expect("decode");
        assert_eq!(
            report.command_acks,
            vec![AgentCommandAck {
                command_id: "c-1".to_string(),
                outcome: command_outcome::SUCCEEDED.to_string(),
                detail: "probes=6 failed=0".to_string(),
            }]
        );
    }
}
