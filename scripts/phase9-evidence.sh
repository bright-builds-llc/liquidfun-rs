#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: $0 <canonical|sanitizer> <target/output-directory>" >&2
  exit 64
}

[[ $# -eq 2 ]] || usage
mode=$1
output_dir=$2

case "$mode" in
  canonical | sanitizer) ;;
  *) usage ;;
esac

case "$output_dir" in
  target/*) ;;
  *) echo "Phase 9 evidence output must be under target/" >&2; exit 64 ;;
esac
[[ "$output_dir" != *".."* && "$output_dir" != /* ]] || {
  echo "unsafe Phase 9 evidence output path" >&2
  exit 64
}
[[ "${output_dir##*/}" == "$mode" || "${output_dir##*/}" == "phase9-$mode" ]] || {
  echo "Phase 9 evidence output must end in $mode or phase9-$mode" >&2
  exit 64
}

mkdir -p "$output_dir"
rm -f "$output_dir/identity.json" "$output_dir/phase9-manifest.json"
trace="$output_dir/phase9-trace.log"
manifest="$output_dir/phase9-manifest.json"

LIQUIDFUN_PHASE9_ORACLE_MODE="$mode" \
LIQUIDFUN_PHASE9_EVIDENCE_MANIFEST="$manifest" \
  cargo test -p liquidfun-differential --test phase9_corpus -- --nocapture \
  2>&1 | tee "$trace"

grep -q 'test result: ok\.' "$trace"
if grep -Eq 'test result: FAILED|FAILED' "$trace"; then
  echo "Phase 9 trace contains a failed test marker" >&2
  exit 1
fi

test -s "$manifest"
jq -e '
  .profile == "phase9-v1" and
  (.upstream_revision | test("^[0-9a-f]{40}$")) and
  (.cases | length == 7) and
  (([.cases[].case_id] | unique | length) == 7) and
  (([.cases[].reached_branches[]] | length) == 58) and
  (([.cases[].reached_branches[]] | unique | length) == 58) and
  all(.cases[];
    (.case_id | length > 0) and
    (.reached_branches | length > 0) and
    (.consumed_policy_paths | length == 22) and
    ((.consumed_policy_paths | unique | length) == 22) and
    ([.request_sha256, .native_result_sha256, .oracle_result_sha256, .comparison_sha256]
      | all(.[]; test("^[0-9a-f]{64}$"))))
' "$manifest" >/dev/null

cargo xtask provenance check 2>&1 | tee "$output_dir/provenance.log"
cargo xtask inventory check 2>&1 | tee "$output_dir/inventory.log"
git diff --exit-code -- protocol scenarios reference COMPATIBILITY.md \
  2>&1 | tee "$output_dir/read-only.log"

hash_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

trace_sha=$(hash_file "$trace")
manifest_sha=$(hash_file "$manifest")
jq -n \
  --argjson run_id "${GITHUB_RUN_ID:-0}" \
  --arg job "${GITHUB_JOB:-$mode-local}" \
  --arg head_sha "${GITHUB_SHA:-local}" \
  --arg trace_sha "$trace_sha" \
  --arg manifest_sha "$manifest_sha" \
  '{run_id: $run_id, job: $job, head_sha: $head_sha,
    upstream_revision: "7f20402173fd143a3988c921bc384459c6a858f2",
    rust: "1.97.0", cmake: "4.3.3", ninja: "1.13.2", clang: "22.1.8",
    target: "x86_64-unknown-linux-gnu", policy: "phase9-v1",
    trace: {path: "phase9-trace.log", sha256: $trace_sha},
    manifest: {path: "phase9-manifest.json", sha256: $manifest_sha}}' \
  > "$output_dir/identity.json"
