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
rm -rf "$output_dir/cases"
rm -f "$output_dir/identity.json" "$output_dir/phase9-manifest.json"
trace="$output_dir/phase9-trace.log"
manifest="$output_dir/phase9-manifest.json"

hash_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

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
  .schema_version == 2 and
  .case_record_schema_version == 1 and
  .profile == "phase9-v1" and
  (.upstream_revision | test("^[0-9a-f]{40}$")) and
  (.semantic_manifest_sha256 | test("^[0-9a-f]{64}$")) and
  (.cases | length == 7) and
  (([.cases[].case_id] | unique | length) == 7) and
  (([.cases[].reached_branches[]] | length) == 58) and
  (([.cases[].reached_branches[]] | unique | length) == 58) and
  all(.cases[];
    (.case_id | length > 0) and
    (.reached_branches | length > 0) and
    (.witnesses | length == (.reached_branches | length)) and
    (.witness_binding_sha256 | test("^[0-9a-f]{64}$")) and
    (.consumed_policy_paths | length == 22) and
    ((.consumed_policy_paths | unique | length) == 22) and
    (.retained_rigid == {
      comparator: "phase8-v1",
      phase6_policy_sha256: "7f10df148852866fd20d11b8d27adcddc0ad463ac3d3d716a8946ca5c8f1c63a",
      phase7_policy_sha256: "fd772b2cf523a6d40bf978bc4d0da18a4564181a93e6b2bdeb8e4d40d5613311",
      phase8_policy_sha256: "2843ca40bec5b1c680135664c58c12a8388a7a9e86ad77f8ef5a268f3f15a6bf",
      outcome: "match",
      comparison_sha256: .retained_rigid.comparison_sha256
    }) and
    ([.request_sha256, .native_result_sha256, .oracle_result_sha256,
      .complete_comparison_sha256, .retained_rigid.comparison_sha256]
      | all(.[]; test("^[0-9a-f]{64}$"))))
' "$manifest" >/dev/null

semantic_manifest=$(jq -c '.cases' "$manifest")
[[ $(printf '%s' "$semantic_manifest" | {
  if command -v sha256sum >/dev/null 2>&1; then sha256sum; else shasum -a 256; fi
} | awk '{print $1}') == "$(jq -r '.semantic_manifest_sha256' "$manifest")" ]]

while IFS=$'\t' read -r path expected_sha; do
  payload="$output_dir/$path"
  [[ -f "$payload" && ! -L "$payload" ]]
  [[ $(hash_file "$payload") == "$expected_sha" ]]
done < <(
  jq -r '.cases[] |
    [.request_path, .request_sha256],
    [.native_result_path, .native_result_sha256],
    [.oracle_result_path, .oracle_result_sha256],
    [.complete_comparison_path, .complete_comparison_sha256] |
    @tsv' "$manifest"
)

cargo xtask provenance check 2>&1 | tee "$output_dir/provenance.log"
cargo xtask inventory check 2>&1 | tee "$output_dir/inventory.log"
git diff --exit-code -- protocol scenarios reference COMPATIBILITY.md \
  2>&1 | tee "$output_dir/read-only.log"

trace_sha=$(hash_file "$trace")
manifest_sha=$(hash_file "$manifest")
identity_files_tmp="${output_dir%/*}/.$mode-identity-files.$$.jsonl"
trap 'rm -f "$identity_files_tmp"' EXIT
while IFS= read -r path; do
  relative=${path#"$output_dir/"}
  [[ "$relative" != "$path" && -f "$path" && ! -L "$path" ]]
  jq -cn --arg path "$relative" --arg sha256 "$(hash_file "$path")" \
    '{path: $path, sha256: $sha256}' >> "$identity_files_tmp"
done < <(find "$output_dir" -type f ! -name identity.json | LC_ALL=C sort)
identity_files=$(jq -s '.' "$identity_files_tmp")
jq -n \
  --argjson run_id "${GITHUB_RUN_ID:-0}" \
  --arg job "${GITHUB_JOB:-$mode-local}" \
  --arg head_sha "${GITHUB_SHA:-local}" \
  --arg trace_sha "$trace_sha" \
  --arg manifest_sha "$manifest_sha" \
  --argjson files "$identity_files" \
  '{run_id: $run_id, job: $job, head_sha: $head_sha,
    upstream_revision: "7f20402173fd143a3988c921bc384459c6a858f2",
    rust: "1.97.0", cmake: "4.3.3", ninja: "1.13.2", clang: "22.1.8",
    target: "x86_64-unknown-linux-gnu", policy: "phase9-v1",
    trace: {path: "phase9-trace.log", sha256: $trace_sha},
    manifest: {path: "phase9-manifest.json", sha256: $manifest_sha},
    files: $files}' \
  > "$output_dir/identity.json"
