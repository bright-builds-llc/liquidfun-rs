#!/usr/bin/env bash
set -euo pipefail

# Upstream's own installer source, reviewed at this immutable commit. Package
# transport uses the installer's signed APT repository; exact patch versions are
# checked below so repository advancement cannot silently change our compiler.
readonly installer_url='https://raw.githubusercontent.com/opencollab/llvm-jenkins.debian.net/eeed6742908255f0eeb12bb8e314366eff3c0a21/llvm.sh'
readonly installer_sha256='9474ecd78b52aba6e923976b1e9773f5613027cc7e237b9956986cb536e02a36'
readonly compiler_bin='/usr/lib/llvm-22/bin'

fail() {
	printf 'canonical-clang: %s\n' "$*" >&2
	exit 1
}

[[ $# == 0 ]] || fail 'usage: bash scripts/install-canonical-clang.sh'
[[ $(uname -s) == Linux && $(uname -m) == x86_64 ]] || fail 'requires Linux x86_64'
candidate_sha=$(git rev-parse HEAD)
[[ "$candidate_sha" =~ ^[0-9a-f]{40}$ ]] || fail 'invalid source commit'
[[ "${CANDIDATE_SHA:-$candidate_sha}" == "$candidate_sha" ]] || fail 'candidate differs from checkout'
run_id=${GITHUB_RUN_ID:-local}
run_attempt=${GITHUB_RUN_ATTEMPT:-local}
job=${GITHUB_JOB:-local}
[[ "$run_id" =~ ^(local|[0-9]+)$ && "$run_attempt" =~ ^(local|[0-9]+)$ ]] || fail 'invalid provider run identity'
[[ "$job" =~ ^[A-Za-z0-9_-]+$ ]] || fail 'invalid provider job identity'

# Never overwrite earlier attempts, including partial or failed installations.
for directory in target target/canonical-clang; do
	[[ ! -L "$directory" ]] || fail "symlinked evidence directory: $directory"
	mkdir -p "$directory"
done
attempt=$(mktemp -d target/canonical-clang/attempt.XXXXXXXX)
readonly attempt
exec > >(tee "$attempt/install.log") 2>&1
trap 'result=$?; printf "exit_code=%s\n" "$result" > "$attempt/status.txt"' EXIT
printf 'candidate_sha=%s\ninstaller_url=%s\ninstaller_sha256=%s\n' \
	"$candidate_sha" "$installer_url" "$installer_sha256" >"$attempt/acquisition.txt"

printf 'Downloading immutable LLVM installer into %s\n' "$attempt"
curl --proto '=https' --tlsv1.2 --fail --location --silent --show-error \
	--connect-timeout 30 --max-time 120 --output "$attempt/llvm.sh" "$installer_url"
printf '%s  %s\n' "$installer_sha256" "$attempt/llvm.sh" | sha256sum --check --strict
printf 'Installing compiler and matching LLVM coverage/sanitizer tools\n'
timeout 1200 sudo bash "$attempt/llvm.sh" 22
timeout 600 sudo apt-get install -y llvm-22 llvm-22-tools libclang-rt-22-dev

export PATH="$compiler_bin:$PATH"
printf 'Verifying exact compiler, tool and target identities\n'
for tool in clang clang++ llvm-cov llvm-profdata; do
	[[ -x "$compiler_bin/$tool" ]] || fail "missing canonical tool: $tool"
	"$compiler_bin/$tool" --version >"$attempt/$tool.version.txt"
	grep -E '(^|[[:space:]])version 22\.1\.8([[:space:]]|$)' "$attempt/$tool.version.txt"
done
for tool in clang-22 clang++-22 llvm-cov-22 llvm-profdata-22; do
	command -v "$tool" >/dev/null || fail "missing versioned canonical tool: $tool"
	"$tool" --version >"$attempt/$tool.version.txt"
	grep -E '(^|[[:space:]])version 22\.1\.8([[:space:]]|$)' "$attempt/$tool.version.txt"
done
for compiler in clang clang++ clang-22 clang++-22; do
	target=$("$compiler" -dumpmachine)
	[[ "$target" == x86_64-pc-linux-gnu ]] || fail "unexpected $compiler target: $target"
	printf '%s=%s\n' "$compiler" "$target" >>"$attempt/targets.txt"
done

# These are real compile/link/run checks through the same drivers CMake uses.
printf 'int main(void) { return 0; }\n' | clang-22 -Werror -x c - -o "$attempt/probe-c"
timeout 10 "$attempt/probe-c"
printf '#include <iostream>\nint main() { std::cout << ""; return 0; }\n' |
	clang++-22 -Werror -x c++ - -o "$attempt/probe-cxx"
timeout 10 "$attempt/probe-cxx"

# GitHub PATH and success identity are withheld until every probe passes.
if [[ -n "${GITHUB_PATH:-}" ]]; then
	printf '%s\n' "$compiler_bin" >>"$GITHUB_PATH"
fi
printf '{"candidate_sha":"%s","installer_url":"%s","installer_sha256":"%s","run_id":"%s","run_attempt":"%s","job":"%s","clang":"22.1.8","llvm":"22.1.8","target":"x86_64-pc-linux-gnu","compile_link_run":"passed"}\n' \
	"$candidate_sha" "$installer_url" "$installer_sha256" "$run_id" "$run_attempt" "$job" >"$attempt/identity.json"
printf 'Canonical compiler verified: %s/identity.json\n' "$attempt"
