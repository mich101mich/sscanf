#!/bin/bash -e

cd "$(dirname "$0")"

export CARGO_TARGET_DIR="../target/sscanf_macro"

output_dir="../target/sscanf_macro/coverage"
mkdir -p "$output_dir"
cargo llvm-cov --lcov --output-path "$output_dir/lcov.info"
