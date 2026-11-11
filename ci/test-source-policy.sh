#!/usr/bin/env bash
# Adversarial hermetic tests of the exact policy gate used in CI.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
test_repo="$(mktemp -d)"
trap 'rm -rf "$test_repo"' EXIT
git init -q "$test_repo"
git -C "$test_repo" config user.name "Policy test"
git -C "$test_repo" config user.email "policy@example.invalid"
mkdir -p "$test_repo/docs" "$test_repo/src"
printf '00_abc\n' > "$test_repo/src/hello.atto"
printf 'a%.0s' {1..40} > "$test_repo/SOURCE_REF"
printf '\n' >> "$test_repo/SOURCE_REF"
git -C "$test_repo" add .
check_ok() {
  SOURCE_POLICY_ROOT="$test_repo" SOURCE_POLICY_MODE="${1:-strict}" bash "$root/ci/source-policy.sh" > /dev/null
}
check_bad() {
  if SOURCE_POLICY_ROOT="$test_repo" SOURCE_POLICY_MODE="${1:-strict}" bash "$root/ci/source-policy.sh" > /dev/null 2>&1; then
    echo "TEST_FAILURE: expected source policy rejection" >&2; exit 1
  fi
}
check_ok
mkdir -p "$test_repo/nested space"
printf 'placeholder\n' > "$test_repo/nested space/unsafe.PY"
git -C "$test_repo" add .
check_bad
git -C "$test_repo" rm -q -f -- "nested space/unsafe.PY"
printf 'placeholder\n' > "$test_repo/src/escape.ipynb"
git -C "$test_repo" add .
check_bad
git -C "$test_repo" rm -q -f -- src/escape.ipynb
printf '#!/usr/bin/env python3\n' > "$test_repo/runner"
git -C "$test_repo" add .
check_bad
git -C "$test_repo" rm -q -f -- runner
ln -s hello.atto "$test_repo/src/alias.atto"
git -C "$test_repo" add .
check_bad
git -C "$test_repo" rm -q -f -- src/alias.atto
printf 'bad\n' > "$test_repo/SOURCE_REF"
git -C "$test_repo" add .
check_bad
printf 'a%.0s' {1..40} > "$test_repo/SOURCE_REF"
printf '\n' >> "$test_repo/SOURCE_REF"
git -C "$test_repo" add .
check_ok
mkdir -p "$test_repo/scripts"
printf 'legacy\n' > "$test_repo/scripts/benchmark.py"
git -C "$test_repo" add .
check_bad
check_ok rest-legacy
printf 'new\n' > "$test_repo/scripts/new_tool.py"
git -C "$test_repo" add .
check_bad rest-legacy
# Exact-HEAD identity is a separate admission invariant, not a compiler receipt.
git -C "$test_repo" rm -q -f -- scripts/benchmark.py scripts/new_tool.py
git -C "$test_repo" commit -qm 'record clean policy test head'
pinned_head="$(git -C "$test_repo" rev-parse HEAD)"
EXPECTED_DEMO_SHA="$pinned_head" check_ok
EXPECTED_DEMO_SHA="$(printf '0%.0s' {1..40})" check_bad
EXPECTED_DEMO_SHA="${pinned_head^^}" check_bad

# SOURCE_REF is precisely one canonical lowercase 40-hex line.
printf 'a%.0s' {1..40} > "$test_repo/SOURCE_REF"
printf '\r\n' >> "$test_repo/SOURCE_REF"
git -C "$test_repo" add SOURCE_REF
check_bad
printf 'a%.0s' {1..40} > "$test_repo/SOURCE_REF"
printf '\n' >> "$test_repo/SOURCE_REF"
git -C "$test_repo" add SOURCE_REF
check_ok
printf 'bad\n' > "$test_repo/COMPILER_REF"
git -C "$test_repo" add COMPILER_REF
check_bad
printf 'b%.0s' {1..40} > "$test_repo/COMPILER_REF"
printf '\n' >> "$test_repo/COMPILER_REF"
git -C "$test_repo" add COMPILER_REF
check_ok
printf 'SOURCE_POLICY_NEGATIVE_TESTS_PASS scenarios=14\n'
