#!/usr/bin/env bash
# Shared command helpers for scripts that run on macOS and Linux.
# Source this file. Do not execute it directly.

sha256_digest() {
	local file=$1
	if [[ ! -f "$file" || -L "$file" ]]; then
		printf 'sha256: need a regular file, got %s\n' "$file" >&2
		return 1
	fi

	local digest=''
	if command -v sha256sum >/dev/null 2>&1; then
		digest=$(sha256sum -- "$file" | awk 'NR == 1 { print $1; exit }')
	fi
	if [[ ! "$digest" =~ ^[0-9a-f]{64}$ ]] && command -v shasum >/dev/null 2>&1; then
		if [[ -n "$digest" ]]; then
			printf 'sha256: sha256sum did not return a 64-digit digest for %s; trying shasum\n' "$file" >&2
		fi
		digest=$(shasum -a 256 -- "$file" | awk 'NR == 1 { print $1; exit }')
	fi
	if [[ ! "$digest" =~ ^[0-9a-f]{64}$ ]]; then
		printf 'sha256: cannot hash %s. Install sha256sum or shasum. Hasher output was: %s\n' \
			"$file" "${digest:-empty}" >&2
		return 1
	fi
	printf '%s\n' "$digest"
}

sha256_require() {
	local expected=$1
	local file=$2
	if [[ ! "$expected" =~ ^[0-9a-f]{64}$ ]]; then
		printf 'sha256: expected digest must be 64 lowercase hex characters, got %s\n' "$expected" >&2
		return 1
	fi
	local actual
	actual=$(sha256_digest "$file") || return 1
	if [[ "$actual" != "$expected" ]]; then
		printf 'sha256 mismatch for %s\n  expected %s\n  actual   %s\n' "$file" "$expected" "$actual" >&2
		return 1
	fi
}

run_bounded() {
	local seconds=$1
	shift
	if [[ ! "$seconds" =~ ^[1-9][0-9]*$ ]]; then
		printf 'run_bounded: deadline must be a positive integer number of seconds, got %s\n' "$seconds" >&2
		return 1
	fi
	"$@" &
	local pid=$!
	local elapsed=0
	while kill -0 "$pid" 2>/dev/null; do
		if ((elapsed >= seconds)); then
			kill "$pid" 2>/dev/null || true
			wait "$pid" 2>/dev/null || true
			printf 'run_bounded: timed out after %ss: %s\n' "$seconds" "$*" >&2
			return 124
		fi
		sleep 1
		elapsed=$((elapsed + 1))
	done
	wait "$pid"
}
