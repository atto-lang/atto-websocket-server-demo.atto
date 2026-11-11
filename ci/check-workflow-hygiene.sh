#!/usr/bin/env bash
# Fail-closed canary for observed CI workflow corruption, not a YAML parser.
set -euo pipefail
workflow="${1:-.github/workflows/upstream-source-hygiene.yml}"
die() { printf 'WORKFLOW_HYGIENE_FAILURE: %s\n' "$*" >&2; exit 1; }
[[ -f "$workflow" && ! -L "$workflow" ]] || die 'missing or symlinked workflow'
test "$(wc -c < "$workflow")" -le 16384 || die 'workflow size budget exceeded'
require_line() { grep -Fqx -- "$1" "$workflow" || die "missing exact reviewed line: $1"; }
one_step() {
  local count
  count="$(grep -Fxc -- "      - name: $1" "$workflow" || true)"
  [[ "$count" == 1 ]] || die "expected exactly one instance of step: $1"
}
require_line "name: Upstream source hygiene (structural only)"
require_line "on:"
require_line "  pull_request:"
require_line "  push:"
require_line "    branches: [main]"
require_line "  workflow_dispatch:"
require_line "          grep -Eq '^[0-9a-f]{40}$' COMPILER_REF"
require_line '          test "$(wc -c < COMPILER_REF)" -eq 41'
require_line "        run: bash ci/test-source-policy.sh"
require_line "        run: bash ci/source-policy.sh"
one_step "Verify source-tip audit and optional SHA pin"
one_step "Reject Python tooling"
one_step "Report structural-only classification"
one_step "Test adversarial source-policy regressions"
one_step "Enforce exact Git-index source policy"
if grep -Eq '^[[:space:]]{0,2}COMPILER_REF[[:space:]]*$' "$workflow"; then
  die 'orphan COMPILER_REF text at document root'
fi
echo 'WORKFLOW_HYGIENE_CANARY_PASS steps=5 (YAML parser and CI NOT_EXECUTED)'
