# Oreslang WebSocket server demo

**Status: source-authored, NOT_EXECUTED.** No actual WebSocket listener has been executed, and no deployable server or verified guest-actor WebSocket bridge is available.

## Actual Oreslang sources

- `src/echo_contract.ores`: **real Oreslang** echo USER-message validation and pure string-echo logic. This is an importable library and deliberately contains **no `main()`**.
- `tests/echo_contract_test.ores`: **native Oreslang test fixture** importing the production `.ores` contract. Its `main()` also targets exit 42 and checks text/binary classification, control/continuation rejection, size limits, and unchanged echo data. This is not an observed compiler result.
- `src/echo_actor.ores`: **real Oreslang actor declaration** with `on_start` and `receive(ActorMail<String>)`. It echoes mailbox text to its parent output stream. A parent output stream is *not* a WebSocket connection; this actor is not yet wired to a socket.

None of these Oreslang source files imports Java, invokes a raw socket, or assumes guest NET authority. The transport remains the host-owned implementation in [oreslang-source.java](https://github.com/ores-truffle-oreslang/oreslang-source.java). Source-level `WebSocketConnection` and compiler lowering are **not** implemented.

## Architecture boundary

The host adapter must validate RFC 6455 handshake, masking, fragmentation, UTF-8, frame/control opcodes and limits **before** delivering a bounded USER message to an authenticated actor incarnation. The native actor consumes `ActorMail<String>`; only a trusted supervisor may claim a physical WebSocket connection. After actor STOP/transfer/cancellation, pending delivery and outbound authority must be generation-fenced and the host must retire physical I/O loans.

Do **not** treat `EchoActor.self.send` as a WebSocket send. The future actor-to-connection bridge needs an opaque `WebSocketConnection` capability with bounded `Future<Result<T,E>>` operations, and it must use the canonical `WebSocketActorConnectionLease` instead of competing ownership registries.

## Execution gate (not yet passed)

1. Compile/typecheck `src/echo_contract.ores` as a library and `tests/echo_contract_test.ores` as the **only** executable `main()`. Require actual guest exit 42. A compiler source commit alone is not a trusted executable provenance receipt; pin both the source SHA and compiler binary SHA-256.
2. Compile/typecheck `src/echo_actor.ores`, prove lifecycle and actor-mailbox behavior, then wire it to an authenticated guest WebSocket adapter. A compiler error is a legitimate failure, not a pass or a skipped acceptance test.
3. Execute localhost RFC 6455 tests with native guest behavior: TEXT/BINARY, masks, fragments, invalid UTF-8, max sizes, ping/pong, backpressure, STOP/transfer/cancel, peer close and resource retirement.
4. Run authorized JDK 25/GraalVM and LLVM tests on permitted **private** runners; capture exact Git-tree provenance and nonzero-step evidence before changing `demo-status.json` execution fields.


## Exact-head native compiler acceptance

`COMPILER_REF` pins the Java compiler source commit expected for this acceptance slice. `ci/run-ores-guest-acceptance.sh` requires a **preinstalled** trusted compiler executable, its SHA-256, the matching compiler source pin, the exact demo commit/tree, and a clean checkout. It then executes:

```sh
oreslang-compiler --check src/echo_contract.ores
oreslang-compiler --check src/echo_actor.ores
oreslang-compiler --check tests/echo_contract_test.ores
oreslang-compiler --mode=jit tests/echo_contract_test.ores
```

Only the final `tests/echo_contract_test.ores` invocation is expected to exit **42**. The library and actor are checked, not invoked as independent programs. The runner must record actual nonzero steps, exit status and stderr; failure to typecheck the actor fails the acceptance.

`.github/workflows/native-ores-guest-acceptance.yml` is fail-closed unless an approved private runner already contains the pinned compiler and exact source/tree variables. It never downloads private compiler source from another organization. The source-org GitHub Actions allocation currently fails before assigning a runner, so these compiler checks have **not** executed.

`ci/bootstrap-status.rs` only enforces truthful source and execution claims. `.github/workflows/bootstrap-boundary.yml` is structural and has not run an Oreslang compiler. Source-org/private-source export remains governed by [security issue #1636](https://github.com/ores-truffle-oreslang/oreslang-source.java/issues/1636); do not mirror private compiler/runtime source across orgs without the prescribed protection and independent approval.
