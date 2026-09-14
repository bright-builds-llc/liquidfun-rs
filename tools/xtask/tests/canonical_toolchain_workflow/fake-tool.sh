#!/usr/bin/env bash
set -euo pipefail

tool=${0##*/}
printf '%s %s\n' "$tool" "$*" >>"$FIXTURE_ROOT/calls.log"
case "$tool" in
uname)
	if [[ "$1" == '-s' ]]; then printf '%s\n' Linux; else printf '%s\n' x86_64; fi
	;;
git) printf '%s\n' 1111111111111111111111111111111111111111 ;;
curl)
	[[ "${FAILURE:-}" != download ]] || exit 22
	while [[ "$1" != --output ]]; do shift; done
	cp "$FIXTURE_ROOT/payload.sh" "$2"
	if [[ "${FAILURE:-}" == checksum ]]; then printf '\nsubstituted\n' >>"$2"; fi
	;;
sudo)
	[[ "${FAILURE:-}" != install ]] || exit 42
	;;
clang | clang++ | clang-22 | clang++-22)
	if [[ "$1" == --version ]]; then
		printf 'Ubuntu clang version %s\n' "${FAKE_VERSION:-22.1.8}"
	elif [[ "$1" == -dumpmachine ]]; then
		printf '%s\n' "${FAKE_TARGET:-x86_64-pc-linux-gnu}"
	else
		[[ "${FAILURE:-}" != compile ]] || exit 43
		cat >/dev/null
		while [[ "$1" != -o ]]; do shift; done
		printf '#!/bin/sh\nexit 0\n' >"$2"
		chmod 755 "$2"
	fi
	;;
llvm-cov | llvm-profdata | llvm-cov-22 | llvm-profdata-22)
	printf 'LLVM version %s\n' "${FAKE_LLVM_VERSION:-22.1.8}"
	;;
*)
	printf 'unexpected fixture tool: %s\n' "$tool" >&2
	exit 99
	;;
esac
