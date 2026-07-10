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
