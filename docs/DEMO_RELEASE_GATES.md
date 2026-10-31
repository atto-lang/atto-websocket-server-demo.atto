# WebSocket Oreslang-source validation gates

Status snapshot: 2026-10-31. **SOURCE_AUTHORED / NOT_EXECUTED** is a source inventory state, not an execution receipt.

## Source inventory (must remain .ores)

- `src/echo_contract.ores`: Oreslang USER echo validation and pure payload echo.
- `src/echo_actor.ores`: Oreslang `EchoActor` / `ActorMail<String>` receive declaration.
- `tests/echo_contract_test.ores`: independently imported Oreslang contract test, targets process exit 42.

Only these three `.ores` sources satisfy the initial authored milestone. Neither Rust structural checks nor host Java probes compile or execute them.

## Executable evidence required

1. **Compiler/typechecker:** pin compiler commit, distribution and executable SHA-256, backend/target and exact demo source SHA/tree; compile both contract/test and actor. A rejected actor source is a failure, not a skip or successful placeholder.
2. **Guest logic:** typecheck the library `src/echo_contract.ores` (which has **no main**) and execute only `tests/echo_contract_test.ores` (the sole executable main) with the pinned compiler. Demand **exit 42** and record exact stdout/stderr/return status and actual nonzero CI steps. A source text grep or manifest check does not count.
3. **Guest WebSocket implementation:** implement `WebSocketServer` and opaque `WebSocketConnection` bindings with generation-fenced actor identity, scoped NET permissions, `Future<Result<T,E>>` operations, bounded receive/send, and no raw Socket/Java host-object export. `EchoActor.self.send` currently targets actor output, **not** a WebSocket peer.
4. **RFC6455 wire:** compile and run a native `.ores` echo actor connected to a real local peer. Verify upgrade negotiation, masked frames, fragmented UTF-8/text, binary payloads, ping/pong, close, malformed frames, size limits, actor STOP/transfer/cancellation, CONTROL priority, and physical loan retirement.
5. **Backends:** independently prove Java/Graal Native Image and LLVM; then WASM, .NET, and Forge. Do not assert backend parity from one runtime's authored tests.

## Pinned runner command and failure admission

`COMPILER_REF` fixes the expected Oreslang compiler **source** SHA. `ci/run-ores-guest-acceptance.sh` checks the preinstalled compiler executable's SHA-256 separately, and checks the exact Git demo commit/tree before any `.ores` compilation. The only approved commands are the actual compiler `--check` operations on the library, actor, and imported test, followed by `--mode=jit tests/echo_contract_test.ores`, which must exit 42. A missing compiler or mismatched pin fails immediately; neither a Java host-only test nor a Rust structural check can stand in for guest proof.

The approval variables are `ORESLANG_DEMO_APPROVED_SHA`, `ORESLANG_DEMO_APPROVED_TREE`, `ORESLANG_COMPILER_EXECUTABLE_PATH`, `ORESLANG_COMPILER_SHA256`, `ORESLANG_COMPILER_SOURCE_SHA`, and `ORESLANG_PRIVATE_EPHEMERAL_RUNNER=true`. Only an independently approved private runner with the verified preinstalled compiler may run the new workflow. No cross-org private-source checkout or artifact download is provided.

## Governance and nonclaims

`demo-status.json` reports `source-authored`, `native_guest_present=true`, and all compiler, CI and wire verification flags false. The Rust checker enforces that no unexecuted state can claim production readiness. Structural CI merely checks tracked files and forbidden source extensions; it does not prove .ores type-checking.

The private Oreslang compiler/runtime tree may not be exported to an alternate GitHub org until [Java source issue #1636](https://github.com/ores-truffle-oreslang/oreslang-source.java/issues/1636) is satisfied with enforced source/carrier protection and independent approval. A token-free Git-tree mirror alone does not satisfy the policy.

Promotion requires exact-head nonzero runner steps, executed native Oreslang guests, independent review, and full log/receipt provenance. Never count queued, skipped, runner-null or steps-null GitHub jobs as passing.
