# AttoLang WebSocket demo — evidence and promotion gates

**Status: source-authored / compiler and AttoLang WebSocket integration NOT_EXECUTED.** This PR replaces the obsolete empty-bootstrap maturity gate. New native code uses `.atto`; the repository name retains its historical `.ores` suffix as an identity, not a source file.

## Evidence tiers

1. **Structural**: source inventory, `demo-status.json`, no symlinks/Python tooling, exact Git HEAD source policy.
2. **Rust host-only RFC 6455**: compile `native/ws_echo.rs` and run its unit plus localhost TCP protocol tests, with nonzero executed CI steps. These tests demonstrate Rust loopback transport behavior **only**, not an AttoLang guest or actor binding.
3. **AttoLang parser/typechecker/guest execution**: run `ci/run-atto-guest-acceptance.sh` with approved preinstalled compiler pinned by `COMPILER_REF` and executable SHA-256; all three `.atto` sources must check, and the one imported test `main` must exit 42. Record exact HEAD and tree, CI runner, steps and logs. The script fails on missing compiler or unsupported syntax.
4. **Guest-to-wire integration**: implement and prove compiler/runtime-owned, generation-fenced `WebSocketConnection` or `WebSocketMessage` bridge; ActorMail cannot carry raw socket handles. Demonstrate actual AttoLang echo with full loopback RFC 6455 negative suite.
5. **AOT and parity**: repeat guest-over-wire tests on Java/GraalVM Native Image, LLVM, .NET NativeAOT, Wasm and Forge only where each backend has implemented the same transport capability. Record independent receipts.

## Negative protocol matrix

Handshake: absent/duplicate/malformed headers, invalid key/version, unsupported extensions, nonlocal Origin, oversized headers, wrong endpoint. Frames: client-unmasked, bad RSV, invalid opcode, nonminimal lengths, fragmented control, oversize, invalid UTF-8, stray continuation, concurrent fragmentation. Lifecycle: close code/reason, ping interleaved with fragments, EOF, read/write timeout, bounded concurrent connections and cleanup. Actor integration additionally requires stale generation, STOP, cancellation, fairness and mailbox backpressure tests.

## Truthful evidence and governance

Compiler latest-main source pin at authoring: `6d8eb70048eafc9a84c5aebe9c1bb06104f16e76`; this is **not** a claim the binary was built or executed. A separate executable digest must be explicitly supplied. Runner `steps=[]`, `null`, runner-null, queued or skipped => **NOT_EXECUTED**. Do not merge solely on host-only or source-inspection receipts. Require independent semantic approval and pin head SHA for merge.

Do not mirror private repo/compiler trees to an alternate org without independently approved secure destination and source/carrier permissions. No automatic cross-org checkout token, Git tree transfer, or implicit trust bypass is installed.
