[private]
default:
    @just --list

check:
    cargo xtask check

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

differential-minimize:
    cargo xtask differential minimize --scenario empty-world --preset oracle-debug --session-profile one-shot

fixture-stage artifact_id:
    cargo xtask differential fixture stage --scenario empty-world --preset oracle-debug --session-profile one-shot --artifact-kind reviewed-trace --artifact-id {{quote(artifact_id)}}

fixture-review artifact_id reviewer reviewed_at review_status="approved":
    cargo xtask differential fixture review --artifact-id {{quote(artifact_id)}} --reviewer {{quote(reviewer)}} --reviewed-at {{quote(reviewed_at)}} --review-status {{quote(review_status)}}

fixture-promote artifact_id:
    cargo xtask differential fixture promote --artifact-id {{quote(artifact_id)}}
