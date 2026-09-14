#!/usr/bin/env bash
set -euo pipefail
root=$1
repository_root=$2
source "$repository_root/scripts/phase12-release-evidence/common.sh"
fail() {
	printf '%s\n' "$1" >&2
	exit 64
}
candidate=1111111111111111111111111111111111111111
run_metadata="$root/platform-run.json"
jq -n --arg candidate "$candidate" '{id:7,head_sha:$candidate,repository:{full_name:"test/repository"},
 path:".github/workflows/platform.yml",status:"completed",conclusion:"success",event:"workflow_dispatch"}' >"$run_metadata"
validate_platform_run "$candidate" 7 test/repository "$run_metadata"
for filter in '.id=8' '.head_sha="2222222222222222222222222222222222222222"' \
	'.repository.full_name="unrelated/repository"' '.path=".github/workflows/ci.yml"' \
	'.status="in_progress"' '.conclusion="failure"'; do
	jq "$filter" "$run_metadata" >"$root/unrelated-run.json"
	if (validate_platform_run "$candidate" 7 test/repository "$root/unrelated-run.json"); then
		fail "accepted unrelated producer: $filter"
	fi
done
package="$root/phase12-package-7-$candidate"
mkdir -p "$package" "$root/output"
printf 'test-only package bytes\n' >"$package/liquidfun.crate"
jq -n --arg candidate "$candidate" --arg hash "$(hash_file "$package/liquidfun.crate")" \
	'{schema_version:1, candidate_commit:$candidate, archive_sha256:$hash,
    archive_bytes:24, package:"liquidfun",version:"0.1.0",created_with_toolchain:"1.97.0"}' \
	>"$package/package-identity.json"
import_platform_package "$candidate" "$root/output" 7 \
	"$package/liquidfun.crate" "$package/package-identity.json"
cmp "$package/liquidfun.crate" "$root/output/package/liquidfun.crate"
cmp "$package/package-identity.json" "$root/output/package/package-identity.json"
for mutation in candidate hash bytes missing symlink ancestor run traversal; do
	destination="$root/$mutation"
	mkdir "$destination"
	cp "$package/package-identity.json" "$root/saved-identity.json"
	archive="$package/liquidfun.crate"
	identity="$package/package-identity.json"
	run=7
	case "$mutation" in
	candidate)
		jq '.candidate_commit="2222222222222222222222222222222222222222"' "$identity" >"$root/mutated"
		mv "$root/mutated" "$identity"
		;;
	hash)
		jq '.archive_sha256="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"' "$identity" >"$root/mutated"
		mv "$root/mutated" "$identity"
		;;
	bytes) printf 'mutated' >>"$archive" ;;
	missing) mv "$identity" "$root/missing" ;;
	symlink)
		mv "$archive" "$root/archive"
		ln -s "$root/archive" "$archive"
		;;
	ancestor)
		ln -s "$package" "$root/link"
		archive="$root/link/liquidfun.crate"
		identity="$root/link/package-identity.json"
		;;
	run) run=8 ;;
	traversal) archive="$package/../${package##*/}/liquidfun.crate" ;;
	esac
	if (import_platform_package "$candidate" "$destination" "$run" "$archive" "$identity"); then
		fail "accepted $mutation"
	fi
	test ! -e "$destination/cheap-identity.json"
	cp "$root/saved-identity.json" "$package/package-identity.json"
	if [[ "$mutation" == symlink ]]; then rm "$package/liquidfun.crate"; fi
	printf 'test-only package bytes\n' >"$package/liquidfun.crate"
done

# Synthetic Cargo failures test the release shell boundary, not release readiness.
for mutation in same changed missing failed mutable; do
	destination="$root/dry-$mutation"
	import_platform_package "$candidate" "$destination" 7 "$package/liquidfun.crate" "$package/package-identity.json"
	cargo() {
		[[ "$*" == 'publish -p liquidfun --dry-run' ]] || fail 'unexpected Cargo operation'
		mkdir -p "$CARGO_TARGET_DIR/package/tmp-crate"
		case "$mutation" in
		failed)
			cp "$package/liquidfun.crate" "$CARGO_TARGET_DIR/package/tmp-crate/liquidfun-0.1.0.crate"
			return 1
			;;
		same) cp "$package/liquidfun.crate" "$CARGO_TARGET_DIR/package/tmp-crate/liquidfun-0.1.0.crate" ;;
		changed) printf 'different' >"$CARGO_TARGET_DIR/package/tmp-crate/liquidfun-0.1.0.crate" ;;
		mutable)
			cp "$package/liquidfun.crate" "$CARGO_TARGET_DIR/package/tmp-crate/liquidfun-0.1.0.crate"
			printf 'changed' >>"$destination/package/liquidfun.crate"
			;;
		missing) ;;
		esac
	}
	if [[ "$mutation" == same ]]; then
		verify_publication_archive "$destination"
	elif (verify_publication_archive "$destination"); then
		fail "accepted dry-run $mutation"
	fi
	test ! -e "$destination/cheap-identity.json"
done
