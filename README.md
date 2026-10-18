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
