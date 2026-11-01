#!/bin/sh
# Story 1.1 CI. Cargo and rustup only. Python is story 2.2 and does not gate this script.
set -eu
cd "$(dirname "$0")"

if ! rustup show active-toolchain | grep -q '1\.99\.1'; then
    echo "active toolchain is not 1.99.1" >&2
    rustup show active-toolchain >&2
    exit 1
fi

target=""
if [ "$#" -ge 1 ]; then
    target=$1
    rustup target add "$target"
fi

run_cargo() {
    if [ -n "$target" ]; then
        rustup run 1.99.1 cargo "$@" --locked --target "$target"
    else
        rustup run 1.99.1 cargo "$@" --locked
    fi
}

rustup run 1.99.1 cargo fmt --check
run_cargo clippy --workspace --all-targets -- -D warnings
run_cargo test --workspace

check_tree() {
    edges=$1
    out=$(rustup run 1.99.1 cargo tree -p sxp-kernel -e "$edges" --target all --prefix none --locked)
    lines=$(printf '%s\n' "$out" | grep -c .)
    if [ "$lines" -ne 1 ]; then
        echo "sxp-kernel $edges tree is not a single line:" >&2
        printf '%s\n' "$out" >&2
        exit 1
    fi
}

check_tree normal
check_tree build

if [ -f crates/sxp-kernel/build.rs ]; then
    echo "sxp-kernel must not have a build.rs" >&2
    exit 1
fi
