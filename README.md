# Oreslang WebSocket server demo

**Status: empty-demo bootstrap.** No actual WebSocket listener, tested Oreslang guest actor, or deployable server has been committed.

## Integration contract

- [Java reference compiler](https://github.com/ores-truffle-oreslang/oreslang-source.java) supplies the existing actor/transport runtime boundary; unsupported WebSocket capabilities must fail closed.
- [oreslang-rpc](https://github.com/ores-truffle-oreslang/oreslang-rpc) owns optional typed request/reply contract identities. Never silently conflate RPC messages with transport frames.
- A connection must be owned by one actor or an explicit bounded shared state protocol. Close/stop and stale-generation attempts must be deterministic.
- Specify max frame/message size, fragmentation behavior, backpressure, authentication policy, and whether control frames are accepted while USER work is overloaded.

## First executable milestone

1. Define an Oreslang-source echo actor and a bounded frame/message adapter with explicit close semantics, only for features supported by the pinned compiler.
2. Add real localhost WebSocket handshake, fragmented-frame, malformed-frame, max-size and peer-disconnect negative tests.
3. Capture guest execution vs host adapter test tiers separately; no claim of RFC compliance until exercised.
4. Use only source-org or independently approved private CI.

Tracked by private [organization integration registry PR #79](https://github.com/ores-truffle-oreslang/oreslang-platform-conformance/pull/79).

## Repository ownership and migration (2026-11-08)

This original `ores-truffle-oreslang/oreslang-websocket-server-demo.ores` repository is still in the Oreslang organization. The public name of the language is **AttoLang**; retain working `.ores` source and existing package, CLI and repository references until an individually verified migration.

- **This repository contains/should own:** WebSocket demo source and protocol/actor acceptance cases.
- **Do not place here:** native full-duplex server claims without real transport receipts.
- **Responsibility split:** [atto-lang](https://github.com/atto-lang) owns the language, portable source libraries and demonstrators; [atto-co](https://github.com/atto-co) owns production multi-tenant cloud infrastructure, deployment, registry, identity and billing. The presence of similarly named `atto-*` repos does not certify transferred source identity or guest parity.
- **Proposed first destination:** `atto-lang/oreslang-websocket-server-demo.ores` via owner-approved GitHub transfer of the **existing numeric repo ID and complete history**, preserving the old repo name. No direct legacy-org transfer to `atto-co` is authorized; later managed-product carve-outs need separate approval.

**Safe transfer steps:** coordinate in [issue #26](https://github.com/ores-truffle-oreslang/.github/issues/26) and use private [transfer runbook draft #25](https://github.com/ores-truffle-oreslang/.github/pull/25). Identify source/target admins, an independent security reviewer and rollback owner; capture exact head SHA/tree, numeric ID, branch protections/rulesets, PRs, releases, packages, permissions, Actions/runners, secrets, webhooks, GitHub Apps, Pages and dependencies. Never mirror private source without authorization. After real transfer, verify same ID/history/tree and permissions, migrate consumer references separately, then test the exact new head with executed CI steps. Zero-step jobs (`steps=[]`/`null`) are `NOT_EXECUTED`, and host-only/static tests do not prove native guest execution or production server capability.
