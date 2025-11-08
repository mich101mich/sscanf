#!/bin/bash -e

test_dir="$(realpath "$(dirname "$0")")"
out_file="${test_dir}/tmp_expanded.rs"

cargo expand --test tmp | head -n -5 >"${out_file}"

# Remove lines starting with '#' as they are compiler-internal directives
sed -i '/^#/d' "${out_file}"
