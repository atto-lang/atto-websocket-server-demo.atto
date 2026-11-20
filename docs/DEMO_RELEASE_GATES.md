# Demo validation gates

Status snapshot: 2026-10-29. This is an evidence checklist, not a passing test report.

## Scope
WebSocket bootstrap.

## Tests required before promotion
Implement native actor echo; test handshake, fragmentation, malformed frames, limits and disconnect with pinned compiler.

Record exact demo and compiler commit SHAs, compiler executable SHA-256, backend/target, executed commands, exit/stdout/stderr, CI runner and step-level results. Unrun, skipped and zero-step runs are **NOT_EXECUTED**, never test passes. Include positive/negative scenarios, bounded-resource behavior and cancellation/teardown where applicable.

## Limitations
README-only: no running WebSocket server.

No production readiness or cross-backend parity is asserted by these documentation changes. Keep native behavior in Oreslang guest source where promised; distinguish host checks from executed guest checks. Do not mirror private Git trees to another organization without explicit approved destination, secure policy and exact-tree provenance.
