#!/bin/bash -e

base_dir="$(realpath "$(dirname "$0")")"
is_proc_macro=1

export base_dir is_proc_macro

"${base_dir}/submodules/test_script/test.sh" "$@"
