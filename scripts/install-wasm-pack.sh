#!/usr/bin/env bash
set -euo pipefail

# Official wasm-pack 0.15.0 Linux musl release. `cargo install` compiles this
# crate in release mode and took about a minute on every Pages run.
readonly version='0.15.0'
readonly archive="wasm-pack-v${version}-x86_64-unknown-linux-musl.tar.gz"
readonly url="https://github.com/wasm-bindgen/wasm-pack/releases/download/v${version}/${archive}"
readonly sha256='c09f971ecaed9a2efc80fdcea7a00ef6b53c7fadc8c57d1f61b53a6aa66b668a'

fail() {
	printf 'install-wasm-pack: %s\n' "$*" >&2
	exit 1
}

[[ $# == 0 ]] || fail 'usage: bash scripts/install-wasm-pack.sh'
[[ $(uname -s) == Linux && $(uname -m) == x86_64 ]] || fail 'requires Linux x86_64'

install_dir=${WASM_PACK_INSTALL_DIR:-"$HOME/.local/bin"}
mkdir -p "$install_dir"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

printf 'Downloading wasm-pack %s\n' "$version"
curl --proto '=https' --tlsv1.2 --fail --location --silent --show-error \
	--connect-timeout 30 --max-time 120 \
	--output "$tmp/$archive" "$url"
printf '%s  %s\n' "$sha256" "$tmp/$archive" | sha256sum --check --strict
tar -xzf "$tmp/$archive" -C "$tmp"
install -m 0755 "$tmp/wasm-pack-v${version}-x86_64-unknown-linux-musl/wasm-pack" "$install_dir/wasm-pack"

if [[ -n "${GITHUB_PATH:-}" ]]; then
	printf '%s\n' "$install_dir" >>"$GITHUB_PATH"
fi

version_text=$("$install_dir/wasm-pack" --version)
[[ "$version_text" == "wasm-pack ${version}" ]] || fail "unexpected version: $version_text"
printf 'installed %s\n' "$version_text"
