# Oreslang WebSocket server demo

**Status: source-authored, NOT_EXECUTED.** No actual WebSocket listener has been executed, and no deployable server or verified guest-actor WebSocket bridge is available.

## Actual Oreslang sources

- `src/echo_contract.ores`: **real Oreslang** echo USER-message validation and pure string-echo logic. Its `main(): int` is designed to return **42** after verifying positive and negative opcode/payload-size cases. This is an authored compiler acceptance target, **not an observed exit code**.
- `src/echo_actor.ores`: **real Oreslang actor declaration** with `on_start` and `receive(ActorMail<String>)`. It echoes mailbox text to its parent output stream. A parent output stream is *not* a WebSocket connection; this actor is not yet wired to a socket.

Neither source file imports Java, invokes a raw socket, or assumes guest NET authority. The transport remains the host-owned implementation in [oreslang-source.java](https://github.com/ores-truffle-oreslang/oreslang-source.java). Source-level `WebSocketConnection` and compiler lowering are **not** implemented.

## Architecture boundary

The host adapter must validate RFC 6455 handshake, masking, fragmentation, UTF-8, frame/control opcodes and limits **before** delivering a bounded USER message to an authenticated actor incarnation. The native actor consumes `ActorMail<String>`; only a trusted supervisor may claim a physical WebSocket connection. After actor STOP/transfer/cancellation, pending delivery and outbound authority must be generation-fenced and the host must retire physical I/O loans.

Do **not** treat `EchoActor.self.send` as a WebSocket send. The future actor-to-connection bridge needs an opaque `WebSocketConnection` capability with bounded `Future<Result<T,E>>` operations, and it must use the canonical `WebSocketActorConnectionLease` instead of competing ownership registries.

## Execution gate (not yet passed)

1. Pin and execute a real Oreslang compiler on `src/echo_contract.ores`. Expect guest exit 42 and retain exact compiler SHA, command, exit status, and step receipt.
2. Compile/typecheck `src/echo_actor.ores`, prove lifecycle and actor-mailbox behavior, then wire it to an authenticated guest WebSocket adapter. A compiler error is a legitimate failure, not a pass or a skipped acceptance test.
3. Execute localhost RFC 6455 tests with native guest behavior: TEXT/BINARY, masks, fragments, invalid UTF-8, max sizes, ping/pong, backpressure, STOP/transfer/cancel, peer close and resource retirement.
4. Run authorized JDK 25/GraalVM and LLVM tests on permitted **private** runners; capture exact Git-tree provenance and nonzero-step evidence before changing `demo-status.json` execution fields.

`ci/bootstrap-status.rs` only enforces truthful source and execution claims. `.github/workflows/bootstrap-boundary.yml` is structural and has not run an Oreslang compiler. Source-org/private-source export remains governed by [security issue #1636](https://github.com/ores-truffle-oreslang/oreslang-source.java/issues/1636); do not mirror private compiler/runtime source across orgs without the prescribed protection and independent approval.
