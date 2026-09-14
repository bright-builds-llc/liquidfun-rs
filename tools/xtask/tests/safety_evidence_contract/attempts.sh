#!/usr/bin/env bash
set -euo pipefail
root=$1
repository=$2
export TEST_CANDIDATE=1111111111111111111111111111111111111111
export GITHUB_RUN_ID=123 GITHUB_RUN_ATTEMPT=2 GITHUB_WORKFLOW=test-only GITHUB_JOB=test-only
unset RUSTC RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER
mkdir "$root/bin"
for tool in git uname rustc cargo clang++-22 llvm-profdata-22 llvm-cov-22 cmake ctest; do
	cp "$repository/tools/xtask/tests/safety_evidence_contract/attempt-tool.sh" "$root/bin/$tool"
	chmod +x "$root/bin/$tool"
done

# An actual deadline, not a fake tool exit, retains timeout's 124 result.
mkdir "$root/real-timeout"
if (
	export PHASE12_MIRI_LIBRARY_ONLY=1
	source "$repository/scripts/phase12-miri.sh"
	begin_attempt "$root/real-timeout" "$TEST_CANDIDATE"
	run_attempt_command deadline 1 sleep 3
) >"$root/deadline.log" 2>&1; then
	echo 'actual timeout accepted' >&2
	exit 1
fi
jq -e '.command_exit_code == 124 and .classification == "timeout" and .elapsed_seconds >= 1' "$root/real-timeout/diagnostics/commands.jsonl"
export PATH="$root/bin:$PATH"

prepare_fixture() {
	local destination=$1
	mkdir -p "$destination/scripts"
	cp "$repository"/scripts/phase12-{miri,rust-sanitizers,coverage}.sh "$destination/scripts/"
	if [[ -f "$repository/scripts/phase12-attempt.sh" ]]; then cp "$repository/scripts/phase12-attempt.sh" "$destination/scripts/"; fi
	cp "$repository"/rust-toolchain*.toml "$destination/"
	mkdir -p "$destination/reference"
	printf '{"entries":[{"id":"fixture.leaf","evidence":{"differentially_validated":{"status":"evidenced"}}}]}' >"$destination/reference/compatibility.json"
	for preset in oracle-debug oracle-release; do
		mkdir -p "$destination/target/reference/$preset"
		touch "$destination/target/reference/$preset/liquidfun-reference"
		chmod +x "$destination/target/reference/$preset/liquidfun-reference"
	done
	touch "$destination/target/reference/oracle-debug/liquidfun-reference-protocol-tests"
	chmod +x "$destination/target/reference/oracle-debug/liquidfun-reference-protocol-tests"
	while read -r source; do
		mkdir -p "$destination/${source%/*}"
		touch "$destination/$source"
	done < <(sed -n 's/^[[:space:]]*\(crates\/[^ ]*\.rs\).*$/\1/p' "$repository/scripts/phase12-miri.sh")
}

for variant in miri rust-sanitizers coverage-rust coverage-cpp coverage-differential; do
	producer=$variant
	mode=run
	relative="target/phase12-$producer/$TEST_CANDIDATE"
	if [[ "$producer" == coverage-* ]]; then
		mode=${producer#coverage-}
		producer=coverage
		relative="target/phase12-coverage/$TEST_CANDIDATE/$mode"
	fi
	for outcome in fail timeout killed oversized; do
		# Arrange / Act: run the actual producer with controlled tool outcomes.
		fixture="$root/$variant-$outcome"
		prepare_fixture "$fixture"
		if TEST_OUTCOME="$outcome" bash "$fixture/scripts/phase12-$producer.sh" "$mode" "$TEST_CANDIDATE" >"$fixture/outer.log" 2>&1; then
			echo 'failed fixture accepted' >&2
			exit 1
		fi
		evidence="$fixture/$relative"
		# Assert: diagnostic terminal exists, identity does not, and retry cannot replace bytes.
		test ! -e "$evidence/identity.json"
		jq -e '.status == "failed" and .exit_code != 0 and .run_attempt == 2' "$evidence/diagnostics/terminal.json"
		(
			cd "$evidence/diagnostics"
			sha256sum -c checksums.sha256
		)
		if [[ "$outcome" == timeout ]]; then jq -e 'select(.command_exit_code == 124) | .classification == "command_failed"' "$evidence/diagnostics/commands.jsonl"; fi
		if [[ "$outcome" == killed ]]; then jq -e 'select(.command_exit_code == 137) | .classification == "command_failed"' "$evidence/diagnostics/commands.jsonl"; fi
		if [[ "$outcome" == fail ]]; then jq -e '.command_exit_code == 42' "$evidence/diagnostics/commands.jsonl"; fi
		if [[ "$outcome" == oversized ]]; then jq -e '.classification == "log_limit" and .retained_bytes <= 16777216' "$evidence/diagnostics/commands.jsonl"; fi
		before=$(sha256sum "$evidence/diagnostics/checksums.sha256")
		if TEST_OUTCOME=success bash "$fixture/scripts/phase12-$producer.sh" "$mode" "$TEST_CANDIDATE" >>"$fixture/retry.log" 2>&1; then
			echo 'reused output accepted' >&2
			exit 1
		fi
		test "$(sha256sum "$evidence/diagnostics/checksums.sha256")" = "$before"
		(
			cd "$evidence/diagnostics"
			sha256sum -c checksums.sha256
		)
	done
	fixture="$root/$variant-success"
	prepare_fixture "$fixture"
	TEST_OUTCOME=success bash "$fixture/scripts/phase12-$producer.sh" "$mode" "$TEST_CANDIDATE" >"$fixture/outer.log" 2>&1
	test -s "$fixture/$relative/identity.json"
	jq -e '.status == "passed" and .exit_code == 0' "$fixture/$relative/diagnostics/terminal.json"
	# The release consumer verifies the actual successful producer diagnostics.
	(
		export PHASE12_RELEASE_EVIDENCE_LIBRARY_ONLY=1
		source "$repository/scripts/phase12-release-evidence.sh"
		validate_attempt_inventory "$fixture/$relative/identity.json"
	)
	cp "$fixture/$relative/identity.json" "$fixture/identity-original.json"
	jq '.run_attempt += 1' "$fixture/identity-original.json" >"$fixture/$relative/identity.json"
	if (
		export PHASE12_RELEASE_EVIDENCE_LIBRARY_ONLY=1
		source "$repository/scripts/phase12-release-evidence.sh"
		validate_attempt_inventory "$fixture/$relative/identity.json"
	) >"$fixture/stale.log" 2>&1; then
		echo 'stale attempt accepted' >&2
		exit 1
	fi
	grep -Fq 'diagnostic terminal identity differs' "$fixture/stale.log"
	cp "$fixture/identity-original.json" "$fixture/$relative/identity.json"
	# Execute the checked-in workflow verification block against this actual producer output.
	workflow=safety
	case "$variant" in
	miri) verification='Verify identity-last Miri output' ;;
	rust-sanitizers) verification='Verify identity-last Rust sanitizer output' ;;
	coverage-rust)
		workflow=coverage
		verification='Verify identity-last Rust coverage output'
		;;
	coverage-cpp)
		workflow=coverage
		verification='Verify identity-last C++ coverage output'
		;;
	coverage-differential)
		workflow=coverage
		verification='Verify identity-last differential output'
		;;
	esac
	awk -v name="$verification" '
	  $0 == "      - name: " name { selected=1; next }
	  selected && /^      - name:/ { exit }
	  selected && /^          / { print substr($0,11) }
	' "$repository/.github/workflows/$workflow.yml" >"$fixture/verify.sh"
	test -s "$fixture/verify.sh"
	(
		cd "$fixture"
		CANDIDATE_SHA="$TEST_CANDIDATE" bash -euo pipefail verify.sh
	)
done
