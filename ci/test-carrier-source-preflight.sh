#!/usr/bin/env bash
set -euo pipefail
script="$(cd "$(dirname "$0")/.." && pwd -P)/ci/carrier-source-preflight.sh"
repo="$(mktemp -d)"
trap 'rm -rf "$repo"' EXIT
git init -q "$repo"
git -C "$repo" config user.email 'probe@example.invalid'
git -C "$repo" config user.name 'Preflight Test'
mkdir -p "$repo/src" "$repo/tests"
printf 'hello\n' > "$repo/src/echo_actor.atto"
printf 'hello\n' > "$repo/src/echo_contract.atto"
printf 'hello\n' > "$repo/tests/echo_contract_test.atto"
printf '%040d\n' 1 > "$repo/COMPILER_REF"
git -C "$repo" add .
git -C "$repo" commit -qm fixture
check_ok() { EXPECTED_DEMO_SHA="$(git -C "$repo" rev-parse HEAD)" EXPECTED_DEMO_TREE="$(git -C "$repo" rev-parse HEAD^{tree})" bash "$script" "$repo" >/dev/null; }
check_bad() {
  if EXPECTED_DEMO_SHA="$(git -C "$repo" rev-parse HEAD)" EXPECTED_DEMO_TREE="$(git -C "$repo" rev-parse HEAD^{tree})" bash "$script" "$repo" >/dev/null 2>&1; then echo "expected failure: $1" >&2; exit 1; fi
}
check_ok
printf dirty > "$repo/untracked.txt"; check_bad untracked; rm "$repo/untracked.txt"
printf dirty >> "$repo/src/echo_actor.atto"; check_bad modified; git -C "$repo" checkout -- src/echo_actor.atto
ln -s echo_actor.atto "$repo/src/evil.atto"; git -C "$repo" add src/evil.atto; git -C "$repo" commit -qm symlink; check_bad symlink
git -C "$repo" rm -q src/evil.atto; git -C "$repo" commit -qm remove-symlink; check_ok
printf secret > "$repo/.env.local"; git -C "$repo" add .env.local; git -C "$repo" commit -qm credential; check_bad credentials
git -C "$repo" rm -q .env.local; git -C "$repo" commit -qm remove-credential; check_ok
printf legacy > "$repo/src/legacy.ores"; git -C "$repo" add src/legacy.ores; git -C "$repo" commit -qm legacy; check_bad legacy
git -C "$repo" rm -q src/legacy.ores; git -C "$repo" commit -qm remove-legacy; check_ok
bad_sha="$(printf 'f%.0s' {1..40})"
if EXPECTED_DEMO_SHA="$bad_sha" EXPECTED_DEMO_TREE="$(git -C "$repo" rev-parse HEAD^{tree})" bash "$script" "$repo" >/dev/null 2>&1; then echo 'expected mismatched SHA failure' >&2; exit 1; fi
bad_tree="$(printf 'f%.0s' {1..40})"
if EXPECTED_DEMO_SHA="$(git -C "$repo" rev-parse HEAD)" EXPECTED_DEMO_TREE="$bad_tree" bash "$script" "$repo" >/dev/null 2>&1; then echo 'expected mismatched tree failure' >&2; exit 1; fi
printf 'PREFLIGHT_OFFLINE_TEST_PASS positive=4 negatives=7\n'
