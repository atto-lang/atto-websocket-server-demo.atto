# Private AttoLang WebSocket CI carrier — independent security review package

**Review-only. No source transfer is authorized by this document or PR.**

Source: ores-truffle-oreslang/oreslang-websocket-server-demo.ores (immutable GitHub repository ID 1421529260). Demo PR #12 head and root tree must be freshly recaptured after every branch update.

## Proven runner capacity, not carrier authorization
- sonus-auris-test/clients-api-contract (ID 1323599063) has a source-free private Actions run 42954312907, job 130305665008 with 3 successful executed steps; logs show Rust 1.99.1. **Not an approved source carrier**; this repo hosts an unrelated project. The temporary source-free probe ref was reset to its original main.
- beamscale/bmscl-compiler (ID 1383937120) was proposed by compiler governance but a proposed allowlist entry does not authorize this demo transfer.
- Demo source main, candidate beamscale main and source-free sonus main were all observed as protected=false.

## Mandatory pre-export gate
1. Establish a dedicated private destination, record exact slug/numeric ID/visibility/collaborators and prohibit unrelated project or tenant access.
2. Protect source and carrier main with PR and CODEOWNER review, stale-review dismissal, admin enforcement, no bypass, no force/deletion, and required issuer-bound security checks.
3. Obtain exact independent ORESoftware approval of destination ID, reviewed head/tree, workflow/test plan and source retention controls on protected authority. No author self-approval.
4. Run audit-only preflight with EXPECTED_DEMO_SHA and EXPECTED_DEMO_TREE in a clean source checkout. It validates Git object types, blob hashes, paths, size caps and canonical .atto inventory. Its success **never grants** permission to transfer.
5. A complete mirror includes .github/workflows entries. Validate that GitHub Actions on the destination cannot run any workflow supplied by the untrusted mirrored branch. Execute only a protected, reviewed carrier workflow without privileged tokens or source-derived workflow triggers.
6. Only after authorization, construct an isolated branch preserving exact Git blob/tree identity; verify destination root tree and review-protected provenance. Do not use cross-org GITHUB_TOKEN checkout or secrets.
7. Execute rustc --edition=2021 --deny warnings --test native/ws_echo.rs plus binary test/run; separately execute the pinned AttoLang guest compiler with expected exit 42. Preserve job ID, runner ID, **nonzero steps**, exact source head/tree and logs.
8. Document cleanup, artifact/cache retention and inaccessible orphaned Git objects. Deleting a branch does not erase previously transferred private Git blobs.

**HOLD: no export, no test promotion, no green check and no merge.** See compiler security issues #1636/#1501 and WebSocket transport PR #2061.
