#!/usr/bin/env bash
set -euo pipefail

root=$1
repository=$2
export PHASE12_RELEASE_EVIDENCE_LIBRARY_ONLY=1
source "$repository/scripts/phase12-release-evidence.sh"

reject() {
	local expected=$1
	shift
	if ("$@") >"$root/rejection.log" 2>&1; then
		printf 'unexpected acceptance: %s\n' "$*" >&2
		exit 1
	fi
	grep -Fq "$expected" "$root/rejection.log"
}

# Arrange: a complete exact artifact set, plus an unexpected root file.
mkdir "$root/downloads"
candidate=$(printf '%040d' 1)
write_expected_artifacts "$root/expected" "$candidate" 1 2 3 4 5 6 7
while read -r artifact; do mkdir "$root/downloads/$artifact"; done <"$root/expected"
validate_artifact_set "$root/downloads" "$root/expected"
touch "$root/downloads/unexpected"
# Act / Assert
reject 'unexpected root' validate_artifact_set "$root/downloads" "$root/expected"
rm "$root/downloads/unexpected"

# Arrange / Act / Assert: reject lexical paths before opening payloads.
mkdir "$root/payload"
printf 'raw measurement\n' >"$root/payload/raw.json"
jq -n --arg hash "$(hash_file "$root/payload/raw.json")" '{payload_sha256:$hash}' >"$root/payload/identity.json"
for relative in ../payload/raw.json ./raw.json sub/../raw.json sub//raw.json raw.json/; do
	reject 'normalized' validate_payload_hash "$root/payload/identity.json" "$root/payload/$relative"
done
ln -s "$root/payload" "$root/link"
reject 'symbolic link' validate_payload_hash "$root/payload/identity.json" "$root/link/raw.json"
reject 'normalized' validate_target_path "$root/"
reject 'normalized' validate_relative_path /tmp/raw.json
reject 'normalized' env PHASE12_RELEASE_EVIDENCE_LIBRARY_ONLY=0 bash "$repository/scripts/phase12-release-evidence.sh" aggregate \
	"$(git rev-parse HEAD)" "$root" "$root/../escape" 1 2 3 4 5 6 7 8
test ! -e "$root/audit-identity.json"

# Arrange: raw files are bound by an independently bound index.
rm "$root/payload/identity.json" "$root/payload/raw.json"
mkdir "$root/payload/raw" "$root/payload/logs"
while read -r case_id; do printf '{}\n' >"$root/payload/raw/$case_id.json"; done < <(jq -r '.cases[].case_id' protocol/benchmarks/phase12-v1.json)
for name in calibrate paired validate; do printf 'command passed\n' >"$root/payload/logs/$name.log"; done
for name in calibration summary validation-identity; do printf '{}\n' >"$root/payload/$name.json"; done
jq '{completed_cases:[.cases[].case_id]}' protocol/benchmarks/phase12-v1.json >"$root/payload/paired-summary.json"
(
	cd "$root/payload"
	find . -type f -printf '%P\n' | sort | xargs sha256sum
) >"$root/index"
cp "$root/index" "$root/payload/payload-files.sha256"
jq -n --arg hash "$(hash_file "$root/payload/payload-files.sha256")" '{payload_files_sha256:$hash}' >"$root/payload/manifest-entry.json"
validate_performance_inventory "$root/payload/manifest-entry.json"
raw="$root/payload/raw/world_step-fixed.json"
printf 'tampered\n' >"$raw"
reject 'hash differs' validate_performance_inventory "$root/payload/manifest-entry.json"
rm "$raw"
reject 'unavailable' validate_performance_inventory "$root/payload/manifest-entry.json"
printf '{}\n' >"$raw"
touch "$root/payload/extra"
reject 'inventory differs' validate_performance_inventory "$root/payload/manifest-entry.json"
rm "$root/payload/extra"
touch "$root/payload/raw/manifest-entry.json"
reject 'inventory differs' validate_performance_inventory "$root/payload/manifest-entry.json"
rm "$root/payload/raw/manifest-entry.json"
touch "$root/payload/raw/line
break.json"
reject 'normalized' validate_performance_inventory "$root/payload/manifest-entry.json"
rm "$root/payload/raw/line
break.json"
cat "$root/payload/payload-files.sha256" >>"$root/duplicate"
cat "$root/duplicate" >>"$root/payload/payload-files.sha256"
jq -n --arg hash "$(hash_file "$root/payload/payload-files.sha256")" '{payload_files_sha256:$hash}' >"$root/payload/manifest-entry.json"
reject 'duplicate' validate_performance_inventory "$root/payload/manifest-entry.json"
cp "$root/index" "$root/payload/payload-files.sha256"
jq -n --arg hash "$(hash_file "$root/index")" '{payload_files_sha256:$hash}' >"$root/payload/manifest-entry.json"
truncate -s 16777217 "$raw"
reject 'byte bound' validate_performance_inventory "$root/payload/manifest-entry.json"
printf '{}\n' >"$raw"
rm "$root/payload/logs/paired.log"
reject 'unavailable' validate_performance_inventory "$root/payload/manifest-entry.json"
printf 'command passed\n' >"$root/payload/logs/paired.log"
printf '{"completed_cases":[]}\n' >"$root/payload/paired-summary.json"
(
	cd "$root/payload"
	find raw logs -type f -print
	printf '%s\n' calibration.json paired-summary.json summary.json validation-identity.json
) | sort | while read -r relative; do printf '%s  %s\n' "$(hash_file "$root/payload/$relative")" "$relative"; done >"$root/payload/payload-files.sha256"
jq -n --arg hash "$(hash_file "$root/payload/payload-files.sha256")" '{payload_files_sha256:$hash}' >"$root/payload/manifest-entry.json"
reject 'paired summary is incomplete' validate_performance_inventory "$root/payload/manifest-entry.json"
printf 'raw payload controls passed\n'
