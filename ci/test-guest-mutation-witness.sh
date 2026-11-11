#!/usr/bin/env bash
# Synthetic Bash admission tests only; never count as an AttoLang compiler receipt.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd -P)"
script="$root/ci/run-guest-mutation-witness.sh"
sandbox="$(mktemp -d)"
trap 'rm -rf -- "$sandbox"' EXIT
repo="$sandbox/demo"
mkdir -p "$repo/src" "$repo/tests"
printf 'pub fnc echo_text(String x): String { return x; }\n' > "$repo/src/echo_contract.atto"
cat > "$repo/tests/echo_contract_test.atto" <<'GUEST'
import fnc {echo_text} from "../src/echo_contract.atto";
pub fnc main(): int {
  return text.unwrap() + binary.unwrap();
}
GUEST
git init -q "$repo"
git -C "$repo" config user.name 'Synthetic Guard Test'
git -C "$repo" config user.email 'synthetic@example.invalid'
git -C "$repo" add .
git -C "$repo" commit -qm synthetic
cat > "$sandbox/faithful-mock" <<'MOCK'
#!/usr/bin/env bash
if [[ "$1" == --check ]]; then exit 0; fi
if grep -Fq '  return 43;' "$2"; then exit 43; fi
exit 42
MOCK
cat > "$sandbox/constant-42-mock" <<'MOCK'
#!/usr/bin/env bash
if [[ "$1" == --check ]]; then exit 0; fi
exit 42
MOCK
cat > "$sandbox/bad-check-mock" <<'MOCK'
#!/usr/bin/env bash
if [[ "$1" == --check ]]; then exit 2; fi
exit 43
MOCK
chmod +x "$sandbox/faithful-mock" "$sandbox/constant-42-mock" "$sandbox/bad-check-mock"
bash "$script" "$sandbox/faithful-mock" "$repo" >/dev/null
reject() {
  local name="$1" command="$2"
  if bash "$script" "$command" "$repo" >/dev/null 2>&1; then
    echo "ATTO_GUEST_MUTATION_MOCK_TEST_FAIL: $name accepted" >&2
    exit 1
  fi
}
reject constant-42 "$sandbox/constant-42-mock"
reject check-failure "$sandbox/bad-check-mock"
printf '  return 43;\n' >> "$repo/tests/echo_contract_test.atto"
reject dirty-source "$sandbox/faithful-mock"
printf 'ATTO_GUEST_MUTATION_HARNESS_TEST_PASS mock_only=1 positive=1 negatives=3 guest_NOT_EXECUTED=1\n'
