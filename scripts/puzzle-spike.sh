#!/usr/bin/env bash
set -euo pipefail

# Phase 37 determinism spike: per-step frame hashes in wasm, native debug and
# native release, compared pairwise into target/puzzle-spike/spike-report.txt.

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="$root/target/puzzle-spike"
expected_lines=900

native_hashes() {
	local destination="$1"
	shift
	cargo test -p liquidfun-wasm "$@" --lib spike_prints_per_step_frame_hashes \
		-- --ignored --nocapture | grep '^SPIKE ' > "$destination"
}

require_lines() {
	local file="$1"
	local count
	count="$(wc -l < "$file" | tr -d ' ')"
	if [ "$count" -ne "$expected_lines" ]; then
		echo "puzzle-spike: $file has $count lines, expected $expected_lines" >&2
		exit 1
	fi
}

compare_pair() {
	local label="$1"
	local left="$2"
	local right="$3"
	echo "== $label"
	(cd "$root/web" && bun scripts/puzzle-replay.ts compare "$left" "$right")
}

write_report() {
	local wasm="$out/spike-wasm.txt"
	local debug="$out/spike-native-debug.txt"
	local release="$out/spike-native-release.txt"
	{
		echo "head $(git -C "$root" rev-parse HEAD)"
		echo "rustc $(rustc --version)"
		echo "bun $(bun --version)"
		compare_pair release-vs-wasm "$release" "$wasm"
		compare_pair debug-vs-release "$debug" "$release"
		compare_pair debug-vs-wasm "$debug" "$wasm"
	} > "$out/spike-report.txt"
}

main() {
	mkdir -p "$out"
	cd "$root"
	bun scripts/web-build.ts wasm
	(cd web && bun scripts/puzzle-replay.ts spike) > "$out/spike-wasm.txt"
	native_hashes "$out/spike-native-debug.txt"
	native_hashes "$out/spike-native-release.txt" --release
	for name in spike-wasm spike-native-debug spike-native-release; do
		require_lines "$out/$name.txt"
	done
	write_report
	cat "$out/spike-report.txt"
}

main "$@"
