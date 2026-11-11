# WebSocket-to-AttoLang actor bridge v1 — integration target

**Status: design contract / NOT_IMPLEMENTED in this demo.** This is a binding acceptance specification, not a claim that `define websocket actor` is executable on Java/Graal current main. Implement against [compiler PR #2061](https://github.com/ores-truffle-oreslang/oreslang-source.java/pull/2061) / [issue #2051](https://github.com/ores-truffle-oreslang/oreslang-source.java/issues/2051), which already establish the canonical `WebSocketActorConnectionLease` and a host-to-actor mailbox path in their unmerged draft. Never fork a second ownership ledger.

## Single-owner state machine

`HTTP_PENDING → OPEN_UNBOUND → BOUND(actor id, context, generation, connection epoch) → CLOSING → RETIRED`.

- The runtime, never user code, authenticates the HTTP upgrade and mints an opaque, non-Sendable connection capability with scoped NET authority.
- Bind is legal only when the socket has **no outstanding raw host read, active writer, queued outbound/user messages or incomplete frame fragment**. Bind consumes the unbound lease; only one actor generation has write/read authority.
- Every admission to a mailbox, reply slot, write queue and close transition checks the exact actor incarnation and connection epoch. Stale generations, STOP, supervisor transfer and callback-after-reuse are rejected.
- An actor cannot invoke its own `receive` dispatcher or fabricate an `ActorMail`. The VM creates the envelope and executes the turn.
- `self.send` currently publishes actor **output**, not WebSocket network bytes. Binding must explicitly subscribe to or claim this actor's output channel and serialize each validated text response to the associated connection; never assume an implicit socket send.
- For BINARY, preserve typed `Array<byte>`/owned buffer messages rather than coercing bytes to String. TEXT must be validated as UTF-8 after complete fragmentation and before actor admission. Control frames stay in the privileged transport.
- No raw file descriptor/JDK Socket/host closure is exposed to guest code. There is no reflection or dynamic-link fallback; imports and guest handlers must remain AOT-compatible.

## Scheduling and backpressure

- Bound frame and **reassembled** USER message to 65,536 bytes, enforce 16 or fewer live sockets in this demo, and maintain independent bounded inbound/outbound queues measured in both **bytes and items**.
- Once mailbox/user-queue admission fails, close the session or apply a documented bounded pause; never drop messages silently and never block an AttoLang actor carrier on network I/O.
- A reserved CONTROL lane processes ping/pong/close while USER work is saturated; retransmits may not bypass byte-credit or connection-generation fencing.
- Cancellation, actor STOP, close handshake, timeout and I/O failure revoke logical authority once. Physical socket, buffers and actor delivery tickets retire only after outstanding loans/continuations quiesce. Generation reuse is forbidden before physical retirement.

## Required proof at promotion

1. **Guest parser and typechecker**: real pinned compiler accepts the three `.atto` files; one `main` exits 42. Negative fixtures reject synthetic `ActorMail` and internal `self.receive` calls.
2. **Guest over wire**: connect a real RFC 6455 peer on loopback, send masked fragmented TEXT and binary; distinguish guest echo from host Rust echo with a guest-only effect/receipt.
3. **Failures**: unmasked/RSV/invalid opcode, nonminimal lengths, oversized >65,536-byte payload, malformed UTF-8, missing/duplicate upgrade fields, HTTP CL/TE smuggling, invalid close, ping interleaving.
4. **Ownership races**: bind during host read, double bind, stale actor generation, transfer with in-flight ticket, STOP before reply, close-vs-write, callback after slot reuse, full mailbox/USER queue with CONTROL progress.
5. **AOT**: Java/Graal Native Image completed source guest-over-wire run and independently executed LLVM and .NET NativeAOT equivalents where supported, then remaining backends. No host-only or source-text pass substitutes for guest execution.

### Canonical vs demo scope

The Rust `native/ws_echo.rs` harness supplies a simple standalone host-only wire oracle and loopback security tests. The real bridge belongs in the compiler/runtime with its preexisting `WebSocketActorConnectionLease`, scheduler, capability checks and private runner protections. This demo should consume those APIs only **after** they compile and work at a reviewed mainline compiler head.
