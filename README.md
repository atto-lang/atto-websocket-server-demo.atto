> **AI agent instructions (organization-wide):** Before working on code, issues, or pull requests, read and follow the [organization `AGENTS.md`](https://github.com/ores-truffle-oreslang/.github/blob/main/AGENTS.md), including its **merge-first stealth-development policy**. Honor any repository-local `AGENTS.md`, explicit holds, and GitHub-enforced rules.

# Oreslang WebSocket server demo

## AttoLang migration and current contracts (2026-11-18)

**AttoLang (formerly Oreslang)** is the public language name. This `ores-truffle-oreslang/oreslang-websocket-server-demo.ores` repository is the legacy named checkout; new corresponding work is routed to **[atto-lang/atto-websocket-server-demo.atto](https://github.com/atto-lang/atto-websocket-server-demo.atto)**. Existing history, issue/PR records, CI results, commands, and implementation status are not assumed identical across the two repos. The content and example commands below document this checkout, and legacy `.ores`, `oreslang-*` names and ABI/wire interfaces remain compatibility surfaces until proven migrations.

**[atto-spec](https://github.com/atto-lang/atto-spec)** is the canonical normative language specification (not old Oreslang README files); [atto-platform-conformance](https://github.com/atto-lang/atto-platform-conformance) owns portable positive/negative guest tests, and [atto-common-source](https://github.com/atto-lang/atto-common-source) owns portable `.atto` library code. Source design targets include `snake_case`, explicit `borrow!`/`take!`/`share!`/`copy!`/`copy!!`, [implicit argument borrowing](https://github.com/atto-lang/atto-spec/blob/main/language/implicit-argument-borrow-v1.md), [file/module `define const` with permitted one-time nonthrowing `os.read_env`](https://github.com/atto-lang/atto-spec/blob/main/language/file-module-const-v1.md) and [bitwise shifts `<<<`/`>>>`/`ashr` versus `<<`/`>>` stream operations](https://github.com/atto-lang/atto-spec/blob/main/language/bitwise-and-stream-operators-v1.md). These are intended semantics, not evidence this old project executes them.

No Python scripts, generators or test/CI harnesses for new AttoLang work; prefer Rust tools when necessary. Compiler/runtime parity requires exact commits/tree, toolchain, actual positive and negative compiled guest tests, and explicit unsupported/failed/not-executed results. Host-only tests or zero-step Actions cannot certify language support.

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
