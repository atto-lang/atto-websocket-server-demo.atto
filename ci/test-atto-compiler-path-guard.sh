#!/usr/bin/env bash
# Test ONLY the native guest harness's admission guard with a synthetic compiler.
# Never claim this script compiles AttoLang or proves guest execution.
set -euo pipefail
script="$(cd "$(dirname "$0")/.." && pwd -P)/ci/run-atto-guest-acceptance.sh"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
repo="$work/demo"
mkdir -p "$repo/src" "$repo/tests" "$repo/ci" "$repo/tools"
cp "$script" "$repo/ci/run-atto-guest-acceptance.sh"
printf 'fnc f(): int { return 1; }\n' > "$repo/src/echo_contract.atto"
printf 'define actor Mock as\nend\n' > "$repo/src/echo_actor.atto"
printf 'pub fnc main(): int { return 42; }\n' > "$repo/tests/echo_contract_test.atto"
printf '%040d\n' 1 > "$repo/COMPILER_REF"
cat > "$repo/tools/mock-compiler" <<'MOCK'
#!/usr/bin/env bash
if [[ "$1" == --check ]]; then exit 0; fi
exit 42
MOCK
chmod 755 "$repo/tools/mock-compiler"
git init -q "$repo"
git -C "$repo" config user.email 'test@example.invalid'
git -C "$repo" config user.name 'Guest Guard Test'
git -C "$repo" add .
git -C "$repo" commit -qm fixture
ln -s "$repo/tools/mock-compiler" "$work/outside-looking-compiler"
head="$(git -C "$repo" rev-parse HEAD)"
tree="$(git -C "$repo" rev-parse HEAD^{tree})"
digest="$(sha256sum "$repo/tools/mock-compiler" | cut -d ' ' -f 1)"
pin="$(cat "$repo/COMPILER_REF")"
check_bad() {
  local name="$1" expected="$2" path="$3" sha="$4" hash="$5" output
  if output="$(EXPECTED_DEMO_SHA="$sha" EXPECTED_DEMO_TREE="$tree" ATTO_COMPILER="$path" ATTO_COMPILER_SHA256="$hash" ATTO_COMPILER_SOURCE_SHA="$pin" bash "$repo/ci/run-atto-guest-acceptance.sh" 2>&1)"; then
    printf 'GUARD_NEGATIVE_FAILURE: %s unexpectedly returned success\n' "$name" >&2
    exit 1
  fi
  if [[ "$output" != *"$expected"* ]]; then
    printf 'GUARD_NEGATIVE_FAILURE: %s wrong reason: %s\n' "$name" "$output" >&2
    exit 1
  fi
}
check_bad outside-symlink 'resolved compiler inside checkout rejected' "$work/outside-looking-compiler" "$head" "$digest"
check_bad inside-direct 'resolved compiler inside checkout rejected' "$repo/tools/mock-compiler" "$head" "$digest"
check_bad wrong-head 'head mismatch' /bin/true "$(printf 'f%.0s' {1..40})" "$digest"
check_bad missing-binary 'compiler path does not resolve' "$work/missing-compiler" "$head" "$digest"
# Digest check must remain independent of syntactic path admission.
cp "$repo/tools/mock-compiler" "$work/real-external-mock"
chmod 755 "$work/real-external-mock"
check_bad wrong-digest 'compiler digest mismatch' "$work/real-external-mock" "$head" "$(printf 'a%.0s' {1..64})"
printf 'ATTO_GUEST_HARNESS_GUARD_TEST_PASS mock_only=1 negatives=5 guest_NOT_EXECUTED=1\n'
