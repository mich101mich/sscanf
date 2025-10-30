#!/bin/bash -e

base_dir="$(realpath "$(dirname "$0")")"
sub_directories="sscanf_macro"
is_proc_macro=1
msrv_overrides="quote@1.0.40 glob@0.3.2 proc-macro2@1.0.101 const_format@0.2.31 unicode-width@0.1.12"

export base_dir sub_directories is_proc_macro msrv_overrides

"${base_dir}/submodules/test_script/test.sh" "$@"
