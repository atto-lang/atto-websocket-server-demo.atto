#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd -P)"
checker="$root/ci/check-workflow-hygiene.sh"
source="$root/.github/workflows/upstream-source-hygiene.yml"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
cp "$source" "$tmp/test.yml"
bash "$checker" "$tmp/test.yml" >/dev/null
expect_bad() {
  if bash "$checker" "$tmp/test.yml" >/dev/null 2>&1; then
    echo "WORKFLOW_CANARY_MUTATION_ESCAPED: $1" >&2; exit 1
  fi
}
printf '      - name: Reject Python tooling\n' >> "$tmp/test.yml"
expect_bad duplicated-step
cp "$source" "$tmp/test.yml"
sed -i "/grep -Eq.*COMPILER_REF/d" "$tmp/test.yml"
expect_bad missing-ref-check
cp "$source" "$tmp/test.yml"
printf ' COMPILER_REF\n' >> "$tmp/test.yml"
expect_bad orphan-line
echo 'WORKFLOW_HYGIENE_MUTATION_TEST_PASS positive=1 negatives=3'
