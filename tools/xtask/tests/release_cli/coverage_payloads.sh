#!/usr/bin/env bash
set -euo pipefail

root=$1
repository=$2
export PHASE12_RELEASE_EVIDENCE_LIBRARY_ONLY=1
source "$repository/scripts/phase12-release-evidence.sh"
candidate=$(printf '%040d' 1)
artifact="$root/leaves.json"
summary="$root/summary.json"

write_summary() {
	jq -n --arg candidate "$candidate" --arg hash "$(hash_file "$artifact")" '
		{schema_version:1, candidate_commit:$candidate, evidence_kind:"differential_coverage",
		 parity_authority:false, toolchain_identity:"semantic-leaf-v1",
		 artifact_path:"leaves.json", artifact_sha256:$hash}' >"$summary"
}

# Arrange: explicit exercised and missed lists, bound by the summary digest.
jq -n '{schema_version:1, parity_authority:false,
 exercised:["leaf-a","leaf-b"], missed:[]}' >"$root/valid.json"
cp "$root/valid.json" "$artifact"
write_summary
# Act / Assert: the production validator accepts valid string entries.
validate_coverage_payload "$summary" "$candidate" differential_coverage

# Arrange: this payload boundary does not define the expected leaf inventory.
jq '.exercised=[]' "$root/valid.json" >"$artifact"
write_summary
# Act / Assert: preserve empty-set policy; producer validation owns completeness.
validate_coverage_payload "$summary" "$candidate" differential_coverage

for mutation in '.exercised=[42]' '.exercised=[null]' '.exercised=[true]' \
	'.exercised=[[]]' '.exercised=[{}]' '.exercised=[""]' \
	'.exercised=["leaf-a","leaf-a"]' '.missed=["leaf-c"]' \
	'.missed=["leaf-a"]' '.missed=[""]' '.missed=[42]' \
	'.exercised={a:"leaf-a"}' '.missed={}' '.exercised="leaf-a"' \
	'.exercised=null' '.missed=null' 'del(.exercised)' 'del(.missed)'; do
	# Arrange: rebind the digest so only the semantic mutation causes rejection.
	jq "$mutation" "$root/valid.json" >"$artifact"
	write_summary
	# Act / Assert
	if (validate_coverage_payload "$summary" "$candidate" differential_coverage) >"$root/rejection.log" 2>&1; then
		printf 'unexpected coverage acceptance: %s\n' "$mutation" >&2
		exit 1
	fi
	grep -Fq 'differential coverage payload is malformed' "$root/rejection.log"
done
printf 'differential coverage payload controls passed\n'
