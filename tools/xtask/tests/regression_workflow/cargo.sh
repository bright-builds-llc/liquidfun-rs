#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >>"$FAKE_COMMAND_LOG"
case "$*" in
"xtask safety-evidence validate-regressions --emit-execution-list")
	printf 'execution-list\n' >>"$FAKE_EVENT_LOG"
	command cat "$FAKE_EXECUTION_LIST"
	;;
test\ -p\ liquidfun\ --test\ regressions\ --all-features\ --\ case_v1\ --exact)
	test "$LIQUIDFUN_REGRESSION_ID" = "case-v1"
	test "$LIQUIDFUN_REGRESSION_INPUT" = "scenarios/regressions/case.bin"
	test "$LIQUIDFUN_REGRESSION_INPUT_SHA256" != ""
	test "$LIQUIDFUN_REGRESSION_TARGET" = "world_mutation"
	test "$LIQUIDFUN_REGRESSION_GENERATOR" = "cargo-fuzz-0.13.2"
	test "$LIQUIDFUN_REGRESSION_TOOLCHAIN" = "nightly-2026-07-15"
	test "$LIQUIDFUN_REGRESSION_ORIGINAL_CANDIDATE" = "1111111111111111111111111111111111111111"
	test "$LIQUIDFUN_REGRESSION_FIX_COMMIT" = "2222222222222222222222222222222222222222"
	test "$LIQUIDFUN_REGRESSION_ORACLE_IDENTITY" = "$(jq -r '.[0].provenance.oracle_identity // ""' "$FAKE_EXECUTION_LIST")"
	test "$LIQUIDFUN_REGRESSION_TOLERANCE_IDENTITY" = "$(jq -r '.[0].provenance.tolerance_identity // ""' "$FAKE_EXECUTION_LIST")"
	test "$LIQUIDFUN_REGRESSION_FIRST_DIVERGENCE" = "checkpoint-1/world.bodies/exact"
	test "$LIQUIDFUN_REGRESSION_FAILURE_CLASS" = "$(jq -r '.[0].provenance.failure_class' "$FAKE_EXECUTION_LIST")"
	printf 'test:%s\n' "$LIQUIDFUN_REGRESSION_ID" >>"$FAKE_EVENT_LOG"
	if [[ "$FAKE_RESULT_MODE" == "oversized-log" ]]; then
		head -c 17825792 /dev/zero
	fi
	if [[ "$FAKE_RESULT_MODE" == "failed-test" ]]; then
		printf 'selected test failed\n'
		exit 101
	fi
	if [[ "$FAKE_RESULT_MODE" == "zero-tests" ]]; then
		printf 'test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n'
	else
		printf 'test case_v1 ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n'
	fi
	;;
"xtask safety-evidence validate-regression-results --candidate $FAKE_CANDIDATE --results target/phase12-regressions/$FAKE_CANDIDATE")
	evidence="$FAKE_REPOSITORY/target/phase12-regressions/$FAKE_CANDIDATE"
	test -s "$evidence/completion.json"
	test ! -e "$evidence/identity.json"
	test ! -e "$evidence/producer-identity.json"
	printf 'validate-results\n' >>"$FAKE_EVENT_LOG"
	case "$FAKE_RESULT_MODE" in
	valid | zero-tests) ;;
	omitted)
		jq '.results = []' "$evidence/completion.json" >"$evidence/mutated.json"
		mv "$evidence/mutated.json" "$evidence/completion.json"
		exit 9
		;;
	duplicated)
		jq '.results += [.results[0]]' "$evidence/completion.json" >"$evidence/mutated.json"
		mv "$evidence/mutated.json" "$evidence/completion.json"
		exit 9
		;;
	unregistered)
		jq '.results[0].regression_id = "unregistered"' "$evidence/completion.json" >"$evidence/mutated.json"
		mv "$evidence/mutated.json" "$evidence/completion.json"
		exit 9
		;;
	*) exit 98 ;;
	esac
	completion_sha256=$(sha256sum "$evidence/completion.json" | awk '{print $1}')
	jq -n \
		--arg candidate_sha "$FAKE_CANDIDATE" \
		--arg regression_manifest_sha256 "$(printf 'a%.0s' {1..64})" \
		--arg completion_sha256 "$completion_sha256" \
		'{schema_version: 1, candidate_sha: $candidate_sha, regression_manifest_sha256: $regression_manifest_sha256, completion_sha256: $completion_sha256}' \
		>"$evidence/identity.json"
	;;
*) exit 99 ;;
esac
