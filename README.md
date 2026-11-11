# AttoLang WebSocket echo demo

**Status: source-authored, guest-to-wire integration unverified.** This repository now contains canonical `.atto` echo contract, actor and guest acceptance test sources, alongside a separate dependency-free Rust loopback RFC 6455 transport harness. These are **two separate execution tiers**: passing Rust host-transport tests would **not** prove a compiled AttoLang actor handles a WebSocket connection.

## Components

- `src/echo_contract.atto`: transport-independent, bounded echo policy for already-validated USER messages.
- `src/echo_actor.atto`: an AttoLang `EchoActor` that consumes an `ActorMail<String>` from its actor mailbox. `self.send` is **not a socket write**.
- `tests/echo_contract_test.atto`: imports production contract and expects exit code **42** after checking positive and negative cases.
- `native/ws_echo.rs`: dependency-free, localhost-only Rust RFC 6455 echo/wire harness with explicit bounds, no AttoLang guest integration. Builds with stable `rustc`.

## Host-only loopback demo (no AttoLang guest)

```sh
mkdir -p target
rustc --edition=2021 --test native/ws_echo.rs -o target/ws-tests
./target/ws-tests
rustc --edition=2021 native/ws_echo.rs -o target/ws-echo
./target/ws-echo
# Connect a WebSocket client to ws://127.0.0.1:9001/ws
```

Binds **only** to 127.0.0.1, port 9001; never expose this demo to a public network. The harness requires client masking and strict upgrade, rejects unsupported extensions and nonlocal browser origins, validates fragmented text UTF-8, handles ping/pong/close, uses bounded frames/messages, caps active connections, and has read/write timeouts. It is *not* an authenticated, production or actor-integrated server.

## AttoLang compiler acceptance (requires externally approved compiler)

```sh
EXPECTED_DEMO_SHA="$(git rev-parse HEAD)" \
EXPECTED_DEMO_TREE="$(git rev-parse HEAD^{tree})" \
ATTO_COMPILER=/absolute/path/to/approved/oreslang-compiler \
ATTO_COMPILER_SHA256="PASTE_APPROVED_64_CHARACTER_LOWERCASE_SHA256" \
ATTO_COMPILER_SOURCE_SHA="$(cat COMPILER_REF)" \
bash ci/run-atto-guest-acceptance.sh
```

Only run with an independently approved preinstalled compiler outside this checkout. The script fails closed for missing/mismatched binary, source pin or exact-head tree. It invokes real compiler `--check` on all three `.atto` files and executes the guest test using `--mode=jit`, requiring exit 42. Do not treat a script's existence or structural CI as evidence it has run.

## Missing binding / next engineering milestone

The compiler's `define websocket actor`, `WebSocketMessage`, and opaque WebSocket connection APIs are design targets, **not proved executable** on latest main. A transport bridge must move validated USER messages into an active actor generation and route its bounded replies back to the right connection, while preserving per-actor ownership, NET permission, cancellation, CONTROL priority, close semantics and backpressure. Do not call `self.send` a network send or run the Rust echo loop in place of a guest.

See [release gates](docs/DEMO_RELEASE_GATES.md) and [machine-readable status](demo-status.json). Legacy `.ores` remains supported by the language for older fixtures, but this repo's new source inventory is **`.atto` only**. Keep the existing repository identity during migration.
