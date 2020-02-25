#! /bin/bash
rustup component add clippy --toolchain nightly-x86_64-unknown-linux-gnu
cargo clippy -- -Aclippy::needless_return -Aclippy::redundant_closure_call -Aclippy::match_single_binding -Dwarnings
