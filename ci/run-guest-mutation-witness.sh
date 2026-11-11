#!/usr/bin/env bash
# Fail-closed actual-execution witness for the accepted AttoLang guest.
# Caller already verified the approved compiler, exact head/tree and baseline exit 42.
set -euo pipefail
fail() { printf 'ATTO_GUEST_MUTATION_FAIL: %s\n' "$*" >&2; exit 1; }
[[ $# -eq 2 ]] || fail 'usage: run-guest-mutation-witness.sh CANONICAL_COMPILER REPO_ROOT'
compiler="$1"
root="$(cd "$2" && pwd -P)" || fail 'invalid repository root'
[[ "$compiler" == /* && -f "$compiler" && -x "$compiler" ]] || fail 'missing compiler executable'
[[ "$(git -C "$root" rev-parse --show-toplevel)" == "$root" ]] || fail 'not repository root'
guest="$root/tests/echo_contract_test.atto"
contract="$root/src/echo_contract.atto"
[[ -f "$guest" && -f "$contract" && ! -L "$guest" && ! -L "$contract" ]] || fail 'expected real guest and contract files'
anchor='  return text.unwrap() + binary.unwrap();'
[[ "$(grep -Fxc "$anchor" "$guest" || true)" == 1 ]] || fail 'baseline return anchor must be unique'
# Preserve relative import topology. No guest mutation may touch the tracked source.
stage="$(mktemp -d "$root/.atto-execution-witness.XXXXXXXX")" || fail 'staging failed'
trap 'rm -rf -- "$stage"' EXIT
mkdir -p "$stage/src" "$stage/tests"
cp -- "$contract" "$stage/src/echo_contract.atto"
awk -v anchor="$anchor" '{
  if ($0 == anchor) {print "  return 43;"; replaced++} else print
} END {if (replaced != 1) exit 91}' "$guest" > "$stage/tests/echo_contract_test.atto" || fail 'exact mutation failed'
[[ "$(grep -Fxc '  return 43;' "$stage/tests/echo_contract_test.atto" || true)" == 1 ]] || fail 'mutation absent'
"$compiler" --check "$stage/tests/echo_contract_test.atto" || fail 'mutant failed compiler check'
set +e
"$compiler" --mode=jit "$stage/tests/echo_contract_test.atto"
mutant_exit=$?
set -e
[[ "$mutant_exit" -eq 43 ]] || fail "mutant exited $mutant_exit, expected 43; cannot prove actual guest evaluation"
rm -rf -- "$stage"
trap - EXIT
[[ -z "$(git -C "$root" status --porcelain=v1 --untracked-files=all)" ]] || fail 'guest mutation left a dirty checkout'
printf 'ATTO_GUEST_NEGATIVE_MUTATION_PASS expected_baseline=42 expected_mutant=43\n'
