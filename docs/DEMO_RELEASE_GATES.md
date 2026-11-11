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

## Protected native guest CI (requires independent approval)

The `native-atto-guest-acceptance.yml` workflow uses immutable PR-head checkout and requires all repository variables below before the private job can be admitted:

- `ATTO_PRIVATE_EPHEMERAL_RUNNER=true` and an authorized runner labeled `oreslang-private-ephemeral` in the **source organization**;
- `ATTO_DEMO_APPROVED_SHA`: independently reviewed exact 40-character demo head;
- `ATTO_DEMO_APPROVED_TREE`: Git commit's actual root **tree** SHA, not the commit SHA;
- `ATTO_COMPILER_EXECUTABLE_PATH`: absolute preinstalled approved binary path, outside the checkout;
- `ATTO_COMPILER_SHA256`: verified lowercase 64-character executable checksum;
- `ATTO_COMPILER_SOURCE_SHA`: approved compiler commit matching `COMPILER_REF`.

Missing approvals deliberately trigger a denial job. A positive gate is only accepted when the **real compiler** typechecks all three `.atto` files and the test's `main` exits 42 at the exact reviewed head. The source organization's zero-step Actions currently provide no such receipt.

## Existing runtime work

Use [Java/Graal actor-owned WebSocket transport PR #2061](https://github.com/ores-truffle-oreslang/oreslang-source.java/pull/2061) and [compiler issue #2051](https://github.com/ores-truffle-oreslang/oreslang-source.java/issues/2051) as the one canonical owner/lease foundation. The host-only Rust harness here is an independent protocol oracle, not a candidate new Java actor authority. [Bridge contract](WEBSOCKET_ACTOR_BRIDGE_V1.md) describes binding, negative tests, and promotion criteria. No cross-org transfer is authorized by these docs.

## Guest executable path trust boundary

The acceptance script resolves the compiler executable with `realpath -e`, checks the **resolved** path is outside the PR checkout, and hashes/invokes that same resolved executable. A seemingly external symlink pointing into the untrusted checkout must not be treated as a preinstalled trusted compiler. The source-boundary workflow includes five **mock-only negative** tests in `ci/test-atto-compiler-path-guard.sh`. These tests check admission behavior and must never be promoted as AttoLang guest execution; all guest, wire and AOT evidence remains `NOT_EXECUTED` until independently verified.

## Mandatory guest negative-mutation witness

After the exact-head and compiler SHA-256 checks, the trusted compiler must first check all three actual `.atto` sources and return guest exit code 42. The same pinned compiler must then check and execute an isolated **in-checkout temporary copy** of the guest test in which the final expression `return text.unwrap() + binary.unwrap();` is uniquely replaced with `return 43;`. The witness requires exit code 43 and verifies that the tracked checkout is clean after staging cleanup. A constant-42 wrapper must fail this witness. This is only a limited, controlled semantic mutation, not formal proof or an adversarial compiler attestation. The source-boundary CI runs a **mock-only** positive and three negative admissions; mock results never count as real guest execution.

The witness's relative import uses the staged `src/echo_contract.atto`. Actual compiler acceptance of staging is **NOT_EXECUTED** until an approved real compiler and runner become available. Do not promote `demo-status.json` based on source inspection, structural checks or mock witnesses.

## Exact-head host Rust CI evidence gate

The host-wire workflow now verifies the checked-out Git HEAD against the event's immutable PR-head SHA, requires a clean checkout, captures the root tree identity and checks exactly **15 tests** are discoverable in the compiled Rust test binary before executing them. Only after the real test process succeeds and the host executable builds with `--deny warnings` does it print `HOST_RFC6455_EXECUTED` with head/tree, test count and `rustc --version`. An admission failure (`runner_id=null`, `steps=[]`) or a missing receipt is **NOT_EXECUTED**. This gate remains host-only; neither actor guest execution nor native AOT can be inferred from it.
