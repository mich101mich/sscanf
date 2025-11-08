#!/bin/bash -e

base_dir="$(realpath "$(dirname "$0")")"
sub_directories="sscanf_macro"
is_proc_macro=1

export base_dir sub_directories is_proc_macro

"${base_dir}/submodules/test_script/test.sh" "$@"
