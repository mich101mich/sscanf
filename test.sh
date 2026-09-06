#!/bin/bash -e

base_dir="$(realpath "$(dirname "$0")")"
is_proc_macro=1
disable_coverage=1 # FIXME: Coverage is currently broken due to build directory changes

export base_dir is_proc_macro disable_coverage

"${base_dir}/submodules/test_script/test.sh" "$@"
