#!/bin/bash

set -ex

export ASAN_OPTIONS="detect_odr_violation=0 detect_leaks=0"

# Run address sanitizer
RUSTFLAGS="-Dwarnings -Z sanitizer=address --cfg all_tests" \
cargo test --tests --target x86_64-unknown-linux-gnu --no-default-features --features std,serde,borsh,arbitrary,quickcheck

# Run leak sanitizer
RUSTFLAGS="-Dwarnings -Z sanitizer=leak --cfg all_tests" \
cargo test --tests --target x86_64-unknown-linux-gnu --no-default-features --features std,serde,borsh,arbitrary,quickcheck

# Run memory sanitizer
RUSTFLAGS="-Dwarnings -Z sanitizer=memory --cfg all_tests" \
cargo -Zbuild-std test --tests --target x86_64-unknown-linux-gnu --no-default-features --features std,serde,borsh,arbitrary,quickcheck

# Run thread sanitizer
RUSTFLAGS="-Dwarnings -Z sanitizer=thread --cfg all_tests" \
cargo -Zbuild-std test --tests --target x86_64-unknown-linux-gnu --no-default-features --features std,serde,borsh,arbitrary,quickcheck
