#!/usr/bin/env bash
set -euo pipefail
root=$1
source_root=$2
xtask=$3
repository_root="$root/repository"
candidate=$(git -C "$source_root" rev-parse HEAD)
git clone --shared --no-checkout "$source_root" "$repository_root"
git -C "$repository_root" checkout --detach "$candidate"
cd "$repository_root"
[[ -z "$(git status --porcelain)" ]] || {
	printf 'package source is dirty\n' >&2
	exit 1
}
test ! -f third_party/liquidfun/liquidfun/Box2D/CMakeLists.txt
[[ "$(git rev-parse HEAD)" == "$candidate" ]]
package="$repository_root/target/phase12-package-7-$candidate"
export CARGO_TARGET_DIR="$repository_root/target/build"
export LIQUIDFUN_XTASK_ROOT="$repository_root"
"$xtask" package create-artifact --archive "$package/liquidfun.crate" \
	--identity "$package/package-identity.json" --candidate-commit "$candidate"
source "$source_root/scripts/phase12-release-evidence/common.sh"
fail() {
	printf '%s\n' "$1" >&2
	exit 64
}
cargo() {
	[[ "$*" != *create-artifact* && "${1:-}" != package ]] || fail 'second archive creation'
	command cargo "$@"
}
output="$repository_root/target/release"
import_platform_package "$candidate" "$output" 7 "$package/liquidfun.crate" "$package/package-identity.json"
host=$(rustc -vV | sed -n 's/^host: //p')
"$xtask" package verify-artifact --archive "$output/package/liquidfun.crate" \
	--identity "$output/package/package-identity.json" --toolchain 1.97.0 --target "$host"
verify_publication_archive "$output"
cmp "$package/liquidfun.crate" "$output/package/liquidfun.crate"
cmp "$package/package-identity.json" "$output/package/package-identity.json"
printf 'real Cargo-only create/import/verify/publish-dry-run bytes passed for %s\n' "$host"

# These provider envelopes are test-only joins over the real archive; they do not
# represent remote/native execution or authorize release acceptance.
source "$source_root/scripts/phase12-release-evidence/producer_validation.sh"
hash=$(hash_file "$output/package/liquidfun.crate")
for target in msrv x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu aarch64-apple-darwin x86_64-pc-windows-msvc; do
	job=native tier=d2_supported platform=$target
	if [[ "$target" == msrv ]]; then job=msrv tier=d1_canonical platform=x86_64-unknown-linux-gnu; fi
	metadata="$root/test-only-$target.json"
	jq -n --arg candidate "$candidate" --arg target "$platform" --arg job "$job" \
		--arg hash "$hash" --arg tier "$tier" \
		'{schema_version:1,candidate_sha:$candidate,target:$target,run_id:7,job:$job,
		 workflow:"Platform release candidate", runner:"synthetic-test-only",compiler:"synthetic-test-only",
		 archive_sha256:$hash,scalar_mode:"strict_f32",tier:$tier,recorded_at_unix:1}' >"$metadata"
	verification="$root/test-only-verification.json"
	printf '%s\n' '{"status":"verified","package_isolation":true,"rustdoc":true,"platform_smoke":true}' >"$verification"
	validate_platform_payload "$metadata" "$verification" "$candidate" "$platform" 7 "$job" "$hash" "$tier"
	if (validate_platform_payload "$metadata" "$verification" "$candidate" "$platform" 8 "$job" "$hash" "$tier"); then
		fail "accepted unrelated green platform run for $target"
	fi
	if (validate_platform_payload "$metadata" "$verification" "$candidate" "$platform" 7 "$job" "$(printf '%064d' 0)" "$tier"); then
		fail "accepted substituted archive for $target"
	fi
done
