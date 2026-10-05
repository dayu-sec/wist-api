# wist-api

Cross-process **API seam** messages for the `wist` control plane (edge ↔ gateway ↔ center).

[![crates.io](https://img.shields.io/crates/v/wist-api.svg)](https://crates.io/crates/wist-api)
[![docs.rs](https://img.shields.io/docsrs/wist-api/latest.svg)](https://docs.rs/wist-api)
[![Downloads](https://img.shields.io/crates/d/wist-api.svg)](https://crates.io/crates/wist-api)
[![CI](https://github.com/dayu-sec/wist-api/actions/workflows/ci.yml/badge.svg)](https://github.com/dayu-sec/wist-api/actions/workflows/ci.yml)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

A **seam** is a runtime coupling point between two independently deployed processes:

```
seam = { route, request body, response body, owner, compat, api_version }
```

Runtime coupling happens on seams, not on the crate graph — a crate is only the carrier of a message.
`wist-api` owns the **wire messages** of each seam so both endpoints `use` a single definition instead of
keeping a copy each (the classic drift bug: same seam, two structs, one of them silently older).

## Modules

| Module         | Seam                                                                       | Endpoints          |
| -------------- | -------------------------------------------------------------------------- | ------------------ |
| `enrollment`   | `agent/enroll`, `agent/credentials:renew`                                   | gateway ↔ agentd   |
| `agent_status` | `agent/status`                                                              | gateway ↔ agentd   |
| `work`         | `agent/work:poll`, `agent/work:ack`, `agent/work:result`                    | gateway ↔ agentd   |
| `agent_uplink` | `agent/uplink:poll`                                                         | gateway ↔ agentd   |
| `gateway`      | `agent/action-plan`, `agent/action-results`, `agent/facts`, `agent/discovery-policies` | gateway ↔ agentd |

Each module follows the same versioned layout — `mod.rs` (version-independent domain re-exports,
`pub use v1::*`, `CURRENT`) + `v1.rs` (the frozen v1 baseline). Adding a `v2` means adding `v2.rs`
and a new route, never branching inside a struct (`api-seam-inventory.md` §7).

Seam messages reference **domain types** shared by more than one seam (`HostProfile` is used by both
`agent/enroll` and `agent/status`). Those stay in [`wist-contracts`](../wist-contracts) — the common base
of every participant — and are re-exported here so callers can pull a whole seam from one place.

## Related crates

- [`wist-contracts`](../wist-contracts) — shared domain / data-plane contract objects.
- [`wist-gateway`](../wist-gateway) — the receiving side of the `agent/*` seams.
- [`wist-agentd`](../wist-agentd) — the sending side of the `agent/*` seams.

## License

[Apache-2.0](LICENSE)
