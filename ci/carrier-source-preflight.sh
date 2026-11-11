#!/usr/bin/env bash
# Audit-only preflight for an independently approved, future private CI mirror.
# NEVER transfers source and NEVER authorizes a carrier or workflow execution.
set -euo pipefail
fail() { printf 'CARRIER_SOURCE_AUDIT_FAIL: %s\n' "$*" >&2; exit 1; }
[[ $# -eq 1 ]] || fail 'usage: carrier-preflight.sh PATH_TO_REPO'
root="$(cd "$1" && pwd -P)" || fail 'repository path does not exist'
cd "$root"
[[ "$(git rev-parse --show-toplevel 2>/dev/null)" == "$root" ]] || fail 'not an exact Git repository root'
for key in EXPECTED_DEMO_SHA EXPECTED_DEMO_TREE; do
  [[ -n "$(printenv "$key" 2>/dev/null || true)" ]] || fail "$key is required"
done
[[ "$EXPECTED_DEMO_SHA" =~ ^[0-9a-f]{40}$ ]] || fail 'invalid expected commit SHA'
[[ "$EXPECTED_DEMO_TREE" =~ ^[0-9a-f]{40}$ ]] || fail 'invalid expected root tree SHA'
[[ "$(git rev-parse HEAD)" == "$EXPECTED_DEMO_SHA" ]] || fail 'head SHA mismatch'
[[ "$(git rev-parse HEAD^{tree})" == "$EXPECTED_DEMO_TREE" ]] || fail 'root tree SHA mismatch'
[[ -z "$(git status --porcelain=v1 --untracked-files=all)" ]] || fail 'dirty checkout or untracked files'
files=0
bytes=0
atto=()
while IFS= read -r -d '' entry; do
  metadata="${entry%%$'\t'*}"
  path="${entry#*$'\t'}"
  read -r mode type blob <<< "$metadata"
  [[ "$path" != *$'\n'* && "$path" != *$'\r'* && "$path" != *$'\t'* ]] || fail 'control characters in tracked path'
  [[ "$type" == blob && ( "$mode" == 100644 || "$mode" == 100755 ) ]] || fail "nonregular Git object or unsafe mode at $path"
  [[ -f "$path" && ! -L "$path" ]] || fail "missing or symlinked tracked file: $path"
  [[ "$(git hash-object -- "$path")" == "$blob" ]] || fail "blob differs from HEAD: $path"
  path_lower="${path,,}"
  case "$path_lower" in
    *.ores|*.pem|*.key|*.p12|*.pfx|*.kdbx|*.asc|*.gpg|*.age|*.env|*.env.*|.env|.env.*|.npmrc|.pypirc|.netrc|id_rsa|id_ed25519|*/id_rsa|*/id_ed25519|*/.env|*/.env.*|*/.npmrc|*/.pypirc|*/.netrc|*/.ssh/*|*/.aws/*|.ssh/*|.aws/*)
      fail "private key, credential or legacy guest extension forbidden: $path" ;;
  esac
  if [[ "$path_lower" == *.atto ]]; then atto+=("$path"); fi
  size="$(git cat-file -s "$blob")"
  [[ "$size" =~ ^[0-9]+$ ]] || fail "invalid blob size: $path"
  (( size <= 1048576 )) || fail "oversized blob: $path"
  bytes=$((bytes+size))
  (( bytes <= 8388608 )) || fail 'total tracked bytes exceed 8 MiB'
  files=$((files+1))
  (( files <= 128 )) || fail 'more than 128 files in source mirror'
done < <(git ls-tree -r -z HEAD)
[[ "${#atto[@]}" == 3 ]] || fail 'unexpected .atto file count'
[[ "${atto[0]}" == src/echo_actor.atto && "${atto[1]}" == src/echo_contract.atto && "${atto[2]}" == tests/echo_contract_test.atto ]] || fail 'unexpected canonical .atto inventory'
[[ -f COMPILER_REF ]] || fail 'compiler source pin absent'
[[ "$(wc -c < COMPILER_REF)" == 41 ]] || fail 'invalid compiler source pin length'
grep -Eq '^[0-9a-f]{40}$' COMPILER_REF || fail 'invalid compiler source pin'
printf 'CARRIER_SOURCE_AUDIT_INSPECT_ONLY head=%s tree=%s blobs=%s bytes=%s\n' "$EXPECTED_DEMO_SHA" "$EXPECTED_DEMO_TREE" "$files" "$bytes"
echo 'PRIVATE_SOURCE_TRANSFER_NOT_AUTHORIZED; require independently approved protected source and destination, trusted carrier workflow, and retention plan'
