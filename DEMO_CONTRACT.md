# WebSocket demo acceptance contract

This repository remains a **bootstrap-only** demo until every gate below has executable evidence. The contract exists so future implementation cannot silently weaken transport, actor, or capability boundaries.

## Admission and transport

- Bind loopback only for the first executable milestone. Public bind requires a separate reviewed change.
- Cap the HTTP upgrade request, individual frame payload, and reconstructed message size independently.
- Reject malformed masking, reserved bits/opcodes, invalid control-frame fragmentation, invalid UTF-8 text frames, and oversized continuation sequences.
- Control frames remain processable while user-message work is backpressured; user frames must never bypass the bounded mailbox.
- Authentication is deny-by-default if enabled. An absent/invalid credential cannot be treated as anonymous success.

## Actor ownership

- One connection has exactly one owning actor unless a reviewed bounded shared-state protocol is introduced.
- The transport/FD cannot be borrowed across actor suspension points or revived after close.
- Frame/message transfer must use Oreslang ownership operations; no hidden host aliasing.
- Backpressure has an explicit bounded capacity and deterministic full/closed result.
- Cancellation, peer close, server close, actor failure, and supervisor shutdown must retire the connection exactly once.

## First executable evidence

The first implementation PR must replace the bootstrap-only CI gate with real checks that compile and execute the exact committed guest source. Required tests include: handshake success/failure, fragmented text, ping/pong under load, malformed frame, max-frame, max-message, peer disconnect, mailbox saturation/backpressure, cancellation during pending send, and graceful shutdown.

A host-only socket simulator is useful adapter evidence but is **not** Oreslang guest execution. No RFC compliance, production readiness, backend parity, or public-network safety claim is allowed without exact-head evidence.
