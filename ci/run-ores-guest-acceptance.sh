#!/usr/bin/env bash
# Run a PREINSTALLED pinned Oreslang compiler; never mirror the private runtime.
set -euo pipefail
fail() { printf 'ORES_GUEST_ACCEPTANCE_FAIL: %s\n' "$*" >&2; exit 1; }
root="$(cd "$(dirname "$0")/.." && pwd -P)"
cd "$root"
[[ "$(git rev-parse --show-toplevel)" == "$root" ]] || fail "repository root mismatch"

for name in EXPECTED_DEMO_SHA EXPECTED_DEMO_TREE ORESLANG_COMPILER ORESLANG_COMPILER_SHA256 ORESLANG_COMPILER_SOURCE_SHA; do
  [[ -n "$(printenv "$name" 2>/dev/null || true)" ]] || fail "$name is required"
done
[[ "$EXPECTED_DEMO_SHA" =~ ^[a-f0-9]{40}$ ]] || fail "invalid demo SHA"
[[ "$EXPECTED_DEMO_TREE" =~ ^[a-f0-9]{40}$ ]] || fail "invalid demo tree"
[[ "$ORESLANG_COMPILER_SOURCE_SHA" =~ ^[a-f0-9]{40}$ ]] || fail "invalid compiler source SHA"
[[ "$ORESLANG_COMPILER_SHA256" =~ ^[a-f0-9]{64}$ ]] || fail "invalid compiler binary sha256"
[[ "$ORESLANG_COMPILER" = /* ]] || fail "compiler path must be absolute"
[[ -f "$ORESLANG_COMPILER" && -x "$ORESLANG_COMPILER" ]] || fail "pinned compiler is not executable"
[[ "$ORESLANG_COMPILER" != "$root/"* ]] || fail "compiler must be preinstalled outside checkout"
[[ "$(git rev-parse HEAD)" == "$EXPECTED_DEMO_SHA" ]] || fail "demo HEAD not approved"
[[ "$(git rev-parse HEAD^{tree})" == "$EXPECTED_DEMO_TREE" ]] || fail "demo tree not approved"
[[ -z "$(git status --porcelain --untracked-files=all)" ]] || fail "demo checkout not clean"
[[ "$(cat COMPILER_REF)" == "$ORESLANG_COMPILER_SOURCE_SHA" ]] || fail "compiler source pin mismatch"
computed="$(sha256sum "$ORESLANG_COMPILER" | cut -d ' ' -f 1)"
[[ "$computed" == "$ORESLANG_COMPILER_SHA256" ]] || fail "compiler binary SHA-256 mismatch"

[[ -f src/echo_contract.ores && -f src/echo_actor.ores && -f tests/echo_contract_test.ores ]] ||
  fail "missing source fixture"
[[ "$(git ls-files -- '*.ores' | wc -l | tr -d ' ')" == 3 ]] || fail "unexpected Oreslang source inventory"
if grep -Eq '^[[:space:]]*(pub[[:space:]]+)?(fnc|routine)[[:space:]]+main[[:space:]]*\(' src/echo_contract.ores src/echo_actor.ores; then
  fail "library or actor defines competing main()"
fi
grep -Eq '^[[:space:]]*pub[[:space:]]+fnc[[:space:]]+main[[:space:]]*\(' tests/echo_contract_test.ores ||
  fail "no executable Oreslang entry"

printf 'ORES_GUEST_APPROVED_HEAD=%s\n' "$EXPECTED_DEMO_SHA"
printf 'ORES_GUEST_APPROVED_TREE=%s\n' "$EXPECTED_DEMO_TREE"
printf 'ORES_COMPILER_SOURCE_PIN=%s\n' "$ORESLANG_COMPILER_SOURCE_SHA"
printf 'ORES_COMPILER_BINARY_SHA256=%s\n' "$computed"

# --check actually parses and type-checks the imported Oreslang source graph.
"$ORESLANG_COMPILER" --check src/echo_contract.ores
"$ORESLANG_COMPILER" --check src/echo_actor.ores
"$ORESLANG_COMPILER" --check tests/echo_contract_test.ores

# OresMain propagates integer main() to process status. 42 is expected here.
set +e
"$ORESLANG_COMPILER" --mode=jit tests/echo_contract_test.ores
result=$?
set -e
[[ "$result" -eq 42 ]] || fail "native Oreslang exit $result, expected 42"
printf 'ORES_GUEST_CONTRACT_EXECUTED exit=42; actor typechecked, WebSocket wire NOT_EXECUTED\n'
