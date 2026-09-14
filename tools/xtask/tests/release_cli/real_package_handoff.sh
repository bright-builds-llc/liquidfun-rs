#!/usr/bin/env bash
set -euo pipefail
root=$1
source_root=$2
xtask=$3
repository_root="$root/repository"
mkdir "$repository_root"
git -C "$source_root" archive HEAD | tar -x -C "$repository_root"
git -C "$repository_root" init -q
git -C "$repository_root" add Cargo.toml Cargo.lock crates LICENSE README.md THIRD_PARTY_NOTICES.md reference/platform .cargo
git -C "$repository_root" -c user.name=Test -c user.email=test@example.invalid commit -qm 'Isolated package test source'
cd "$repository_root"
test ! -f third_party/liquidfun/liquidfun/Box2D/CMakeLists.txt
candidate=$(git rev-parse HEAD)
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
