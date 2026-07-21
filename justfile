[private]
default:
    @just --list

check:
    cargo xtask check

markdown-check:
    mdformat --check .

fmt:
    cargo fmt --all --check

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

build:
    cargo build --all-targets --all-features

test:
    cargo test --all-features

package-verify:
    cargo xtask package verify

inventory-check:
    cargo xtask inventory check

upstream-verify:
    cargo xtask upstream verify

oracle-debug:
    cargo xtask upstream configure --preset oracle-debug
    cargo xtask upstream build --preset oracle-debug

differential-compare:
    cargo xtask differential compare --scenario empty-world --preset oracle-debug --session-profile one-shot

differential-reuse:
    cargo xtask differential compare --scenario empty-world --preset oracle-debug --session-profile reuse

differential-replay:
    cargo xtask differential replay --scenario empty-world --preset oracle-debug --session-profile one-shot

math-probes-debug:
    cargo xtask differential compare --scenario math-probes --preset oracle-debug --session-profile one-shot

math-probes-release:
    cargo xtask differential compare --scenario math-probes --preset oracle-release --session-profile one-shot

math-probes-replay:
    cargo xtask differential replay --scenario math-probes --preset oracle-debug --session-profile one-shot

math-probes-determinism:
    cargo xtask differential verify-determinism --scenario math-probes --preset oracle-debug --runs 2

collision-probes-debug:
    cargo xtask differential compare --scenario collision-probes --preset oracle-debug --session-profile one-shot

collision-probes-release:
    cargo xtask differential compare --scenario collision-probes --preset oracle-release --session-profile one-shot

collision-probes-replay:
    cargo xtask differential replay --scenario collision-probes --preset oracle-debug --session-profile one-shot

collision-probes-determinism:
    cargo xtask differential verify-determinism --scenario collision-probes --preset oracle-debug --runs 2

rigid-world-debug:
    cargo xtask differential compare --scenario rigid-world --preset oracle-debug --session-profile one-shot

rigid-world-release:
    cargo xtask differential compare --scenario rigid-world --preset oracle-release --session-profile one-shot

rigid-world-replay:
    cargo xtask differential replay --scenario rigid-world --preset oracle-debug --session-profile one-shot

rigid-world-determinism:
    cargo xtask differential verify-determinism --scenario rigid-world --preset oracle-debug --runs 2

phase10-evidence-canonical output="target/phase10-evidence-local/canonical":
    LIQUIDFUN_PHASE10_ORACLE_MODE=canonical bash scripts/phase10-evidence.sh canonical {{quote(output)}}

phase10-evidence-sanitizer output="target/phase10-evidence-local/sanitizer":
    UBSAN_OPTIONS=halt_on_error=1:print_stacktrace=1 ASAN_OPTIONS=abort_on_error=1:halt_on_error=1 LIQUIDFUN_PHASE10_ORACLE_MODE=sanitizer bash scripts/phase10-evidence.sh sanitizer {{quote(output)}}

phase10-evidence-validate canonical="target/phase10-evidence-local/canonical" sanitizer="target/phase10-evidence-local/sanitizer":
    cargo xtask phase10-evidence validate --mode local --canonical-dir {{quote(canonical)}} --sanitizer-dir {{quote(sanitizer)}}

rigid-world-minimize:
    cargo xtask differential minimize --scenario rigid-world --preset oracle-debug --session-profile one-shot

rigid-fixture-stage artifact_id:
    cargo xtask differential fixture stage --scenario rigid-world --preset oracle-debug --session-profile one-shot --artifact-kind reviewed-trace --artifact-id {{quote(artifact_id)}}

rigid-fixture-review artifact_id reviewer reviewed_at review_status="approved":
    cargo xtask differential fixture review --artifact-id {{quote(artifact_id)}} --reviewer {{quote(reviewer)}} --reviewed-at {{quote(reviewed_at)}} --review-status {{quote(review_status)}}

rigid-fixture-promote artifact_id:
    cargo xtask differential fixture promote --artifact-id {{quote(artifact_id)}}

differential-minimize:
    cargo xtask differential minimize --scenario empty-world --preset oracle-debug --session-profile one-shot

fixture-stage artifact_id:
    cargo xtask differential fixture stage --scenario empty-world --preset oracle-debug --session-profile one-shot --artifact-kind reviewed-trace --artifact-id {{quote(artifact_id)}}

fixture-review artifact_id reviewer reviewed_at review_status="approved":
    cargo xtask differential fixture review --artifact-id {{quote(artifact_id)}} --reviewer {{quote(reviewer)}} --reviewed-at {{quote(reviewed_at)}} --review-status {{quote(review_status)}}

fixture-promote artifact_id:
    cargo xtask differential fixture promote --artifact-id {{quote(artifact_id)}}
