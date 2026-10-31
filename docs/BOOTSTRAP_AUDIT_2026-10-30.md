# WebSocket demo audit — 2026-10-30

This is a **structural source hardening change**, not guest execution or transport conformance.

## Observed baseline

- The default branch is bootstrap-only and has no `.ores` source, listener, or RFC 6455 guest proof.
- The prior status workflow executed a Python script inside CI, contrary to the Oreslang no-Python policy.
- The previous status validator accepted partial maturity transitions and did not require a clean, exact-head checkout. The separate boundary workflow checks `.ores` and symlinks, but did not tie all maturity assertions to the tracked Git index.
- Draft PR #2 overlaps this bootstrap scope; reconcile only after reviewing both diffs and CI evidence.

## Hardening in this PR

- Replace the CI Python maturity script with a dependency-free Rust gate.
- Require a clean, exact-head Git tree; reject symlinks/submodules, tracked `.ores` files, Python tooling/shebangs, privileged Actions triggers, and build-system manifests during bootstrap.
- Pin the entire machine-readable demo status to `bootstrap`, with every guest/transport/exact-head execution bit `false`. Any intentional promotion must deliberately replace this gate **and** prove the new source with a compiler-backed test.
- Add Rust unit tests for negative paths, revision admission and workflow triggers. The tests themselves are **HOST_STRUCTURAL_ONLY**.
- Run the gate on pull requests, pushes to `main`, and manual dispatch, with read-only checkout and immutable action reference.

## Still blocked

WebSocket transport and actor-owned frame/close semantics remain unimplemented in this repository. Before any guest/runtime claim, implement and actually execute an Oreslang actor and frame adapter with bounded upgrade/frame/message limits, client masking, fragmentation, UTF-8 validation, ping/pong despite user backpressure, cancellation, race-safe single retirement, and generation fencing. Run each negative against the exact source/compiler head on an authorized trusted runner. `steps=[]` or runner-admission failure means **NOT_EXECUTED**. Do not mirror private source without separate documented transfer authorization and independent approval.

The Phoenix-style HTTP repository is a **separate**, unmerged loopback HTTP source candidate; its tests do not establish WebSocket behavior or Phoenix Channels wire compatibility.
