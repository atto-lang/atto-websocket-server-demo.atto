> **AI agent instructions (organization-wide):** Before working on code, issues, or pull requests, read and follow the [organization `AGENTS.md`](https://github.com/atto-lang/.github/blob/main/AGENTS.md), including its **merge-first stealth-development policy**. Honor any repository-local `AGENTS.md`, explicit holds, and GitHub-enforced rules.

# atto-websocket-server-demo.atto

WebSocket server example demonstrating asynchronous connections and actors in AttoLang.

## Canonical naming and documentation migration

[AttoLang](https://github.com/atto-lang) is the sole canonical project name; configure repository remotes as `https://github.com/atto-lang/atto-websocket-server-demo.atto.git` and refer to `.atto` in new examples. [atto-spec](https://github.com/atto-lang/atto-spec) owns normative grammar, ownership, lifetime, effects and actor contracts; [atto-platform-conformance](https://github.com/atto-lang/atto-platform-conformance) tracks executed verification. Distinguish design proposals from tested compiler support and do not advertise zero-step CI as guest conformance.

## Demonstration focus

WebSocket connections, channel events, and message handling.

## Running the example

Inspect the checked-in source, project manifest, and workflows for supported compiler backends and actual run commands. Source files use the intended `.atto` extension for AttoLang; backend support must be confirmed rather than presumed.

## Expected verification

Examples should document their dependencies, expected output, negative/error cases, and exact compiler/runtime versions used to verify execution. Host-side tests do not establish AttoLang guest execution.

## Status

This README describes the demonstration's intended purpose. Current executable coverage and feature support should be verified against code and CI.

## AttoLang ecosystem

- [AttoLang](https://github.com/atto-lang)
- [Specification](https://github.com/atto-lang/atto-spec)
- [Platform conformance](https://github.com/atto-lang/atto-platform-conformance)

## Repository ownership and contribution routing

**Belongs here:** Runnable WebSocket handshake/frame processing example, actor-backed connections, bounded queues, ping/pong and shutdown behavior.

**Belongs elsewhere:** Generic HTTP routing in atto-http-routing, RPC protocols in atto-rpc and production gateway services in atto-co.

**Security and portability boundary:** Reject invalid frames/origins and overlarge payloads, enforce backpressure, prevent message-to-closure transport and handle disconnect/cancel races.

AttoLang is the canonical project identity; all new examples and documentation use `.atto` and the `atto-lang` organization. This README defines scope, not feature readiness.

**Acceptance:** name exact source SHA/tree, runtime/toolchain, positive and negative checks and execution receipt before claiming support. Host-only tests and empty CI steps do not prove guest execution.

## Legacy migration and verification gates

The historical predecessor was `ores-truffle-oreslang/oreslang-websocket-server-demo.ores`. This repository is now maintained in `atto-lang`; a similarly named repository or imported history does not by itself establish preservation of the original numeric repository ID, source parity or compiler support. Preserve working legacy `.ores` inputs, old package names and CLI references until compatibility and replacement paths have been verified; new source follows the canonical `.atto` specification.

Do not claim native full-duplex WebSocket support without real transport execution receipts. [atto-lang](https://github.com/atto-lang) owns language, tooling and portable examples/libraries; [atto-co](https://github.com/atto-co) owns separately governed hosted deployment, tenant infrastructure, registry, identity and billing. Any further original-ID transfer or managed-product carve-out requires separate owner authorization; no direct legacy-org move to atto-co is authorized by this document.

Before an actual transfer, coordinate with the [migration issue](https://github.com/ores-truffle-oreslang/.github/issues/26) and authorized reviewers; inventory numeric ID, exact HEAD/tree, history, access, branch protections/rulesets, open PRs, packages, releases, CI/runners, secrets, webhooks, apps, Pages, dependencies and rollback ownership. Never disclose or mirror private code without authorization. After transfer, verify ID/tree/history and security controls, then execute exact-head tests before changing consumer references. Empty CI steps are NOT_EXECUTED; host-only checks are not compiled guest evidence.
