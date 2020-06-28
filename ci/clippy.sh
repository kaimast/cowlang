#! /bin/bash
rustup component add clippy --toolchain nightly-x86_64-unknown-linux-gnu
cargo clippy -- -Aclippy::needless_return -Aclippy::redundant_closure_call -Aclippy::unused_unit -Aclippy::let_unit_value -Aclippy::unit_arg -Aclippy::match_single_binding -Dwarnings -Aunused_braces
