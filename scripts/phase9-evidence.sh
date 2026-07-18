#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: $0 <canonical|sanitizer> <target/output-directory>" >&2
  exit 64
}

[[ $# -eq 2 ]] || usage
mode=$1
relative_output_dir=$2

case "$mode" in
  canonical | sanitizer) ;;
  *) usage ;;
esac

case "$relative_output_dir" in
  target/*) ;;
  *) echo "Phase 9 evidence output must be under target/" >&2; exit 64 ;;
esac
[[ "$relative_output_dir" != *".."* && "$relative_output_dir" != /* ]] || {
  echo "unsafe Phase 9 evidence output path" >&2
  exit 64
}
[[ "${relative_output_dir##*/}" == "$mode" || "${relative_output_dir##*/}" == "phase9-$mode" ]] || {
  echo "Phase 9 evidence output must end in $mode or phase9-$mode" >&2
  exit 64
}

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
repository_root=$(cd -- "$script_dir/.." && pwd -P)
cd -- "$repository_root"

target_root="$repository_root/target"
if [[ -L "$target_root" ]]; then
  echo "Phase 9 evidence target root must not be a symlink" >&2
  exit 64
fi
mkdir -p -- "$target_root"
target_root=$(cd -- "$target_root" && pwd -P)

output_dir="$repository_root/$relative_output_dir"
current="$target_root"
IFS='/' read -r -a output_components <<< "${relative_output_dir#target/}"
for component in "${output_components[@]}"; do
  candidate="$current/$component"
  if [[ -L "$candidate" ]]; then
    echo "Phase 9 evidence output path contains a symlink: $candidate" >&2
    exit 64
  fi
  if [[ -e "$candidate" && ! -d "$candidate" ]]; then
    echo "Phase 9 evidence output path contains a non-directory: $candidate" >&2
    exit 64
  fi
  if [[ ! -e "$candidate" ]]; then
    mkdir -- "$candidate"
  fi
  current=$(cd -- "$candidate" && pwd -P)
  case "$current/" in
    "$target_root"/*) ;;
    *) echo "Phase 9 evidence output escapes target/" >&2; exit 64 ;;
  esac
done
output_dir="$current"

if [[ -L "$output_dir/cases" ]]; then
  echo "Phase 9 evidence cases path must not be a symlink" >&2
  exit 64
fi
rm -rf "$output_dir/cases"
rm -f "$output_dir/identity.json" "$output_dir/phase9-manifest.json"
trace="$output_dir/phase9-trace.log"
manifest="$output_dir/phase9-manifest.json"
relative_manifest="$relative_output_dir/phase9-manifest.json"

hash_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

LIQUIDFUN_PHASE9_ORACLE_MODE="$mode" \
LIQUIDFUN_PHASE9_EVIDENCE_MANIFEST="$relative_manifest" \
  cargo test -p liquidfun-differential --test phase9_corpus -- --nocapture \
  2>&1 | tee "$trace"

cargo xtask provenance check 2>&1 | tee "$output_dir/provenance.log"
cargo xtask inventory check 2>&1 | tee "$output_dir/inventory.log"
git diff --exit-code -- protocol scenarios reference COMPATIBILITY.md \
  2>&1 | tee "$output_dir/read-only.log"
cargo xtask phase9-evidence validate-content "$mode" "$relative_output_dir"

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
