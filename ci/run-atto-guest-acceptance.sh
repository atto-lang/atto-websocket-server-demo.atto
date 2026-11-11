#!/usr/bin/env bash
# Fail closed: no alternate-host fallback, no simulated/compiler-free success.
set -euo pipefail
fail() { printf 'ATTO_GUEST_ACCEPTANCE_FAIL: %s\n' "$*" >&2; exit 1; }
root="$(cd "$(dirname "$0")/.." && pwd -P)"
cd "$root"
[[ "$(git rev-parse --show-toplevel)" == "$root" ]] || fail "not repository root"
for name in EXPECTED_DEMO_SHA EXPECTED_DEMO_TREE ATTO_COMPILER ATTO_COMPILER_SHA256 ATTO_COMPILER_SOURCE_SHA; do
  [[ -n "$(printenv "$name" 2>/dev/null || true)" ]] || fail "$name required"
done
[[ "$EXPECTED_DEMO_SHA" =~ ^[0-9a-f]{40}$ ]] || fail "invalid demo SHA"
[[ "$EXPECTED_DEMO_TREE" =~ ^[0-9a-f]{40}$ ]] || fail "invalid tree SHA"
[[ "$ATTO_COMPILER_SOURCE_SHA" =~ ^[0-9a-f]{40}$ ]] || fail "invalid source SHA"
[[ "$ATTO_COMPILER_SHA256" =~ ^[0-9a-f]{64}$ ]] || fail "invalid compiler SHA256"
[[ "$ATTO_COMPILER" == /* && -f "$ATTO_COMPILER" && -x "$ATTO_COMPILER" ]] || fail "preinstalled executable required"
[[ "$ATTO_COMPILER" != "$root/"* ]] || fail "compiler inside checkout rejected"
[[ "$(git rev-parse HEAD)" == "$EXPECTED_DEMO_SHA" ]] || fail "head mismatch"
[[ "$(git rev-parse HEAD^{tree})" == "$EXPECTED_DEMO_TREE" ]] || fail "tree mismatch"
[[ -z "$(git status --porcelain --untracked-files=all)" ]] || fail "dirty checkout"
[[ "$(cat COMPILER_REF)" == "$ATTO_COMPILER_SOURCE_SHA" ]] || fail "compiler source pin mismatch"
[[ "$(sha256sum "$ATTO_COMPILER" | cut -d ' ' -f 1)" == "$ATTO_COMPILER_SHA256" ]] || fail "compiler digest mismatch"
mapfile -t guests < <(git ls-files -- '*.atto')
[[ "${#guests[@]}" -eq 3 ]] || fail "expected exactly three tracked .atto files"
[[ "${guests[0]}" == src/echo_actor.atto && "${guests[1]}" == src/echo_contract.atto && "${guests[2]}" == tests/echo_contract_test.atto ]] || fail "unexpected AttoLang inventory"
[[ -z "$(git ls-files -- '*.ores')" ]] || fail "legacy source in demo"
! grep -Eq '^[[:space:]]*(pub[[:space:]]+)?fnc[[:space:]]+main\(' src/*.atto || fail "multiple main entries"
for source in src/echo_contract.atto src/echo_actor.atto tests/echo_contract_test.atto; do
  "$ATTO_COMPILER" --check "$source"
done
set +e
"$ATTO_COMPILER" --mode=jit tests/echo_contract_test.atto
code=$?
set -e
[[ "$code" -eq 42 ]] || fail "guest returned $code, expected 42"
printf 'ATTO_GUEST_EXECUTED head=%s tree=%s compiler_source=%s compiler_sha256=%s exit=42\n' \
  "$EXPECTED_DEMO_SHA" "$EXPECTED_DEMO_TREE" "$ATTO_COMPILER_SOURCE_SHA" "$ATTO_COMPILER_SHA256"
printf 'NOT_VERIFIED: guest-to-WebSocket bridge, native AOT, other backends\n'
