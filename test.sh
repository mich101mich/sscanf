#!/bin/bash

function try_silent {
    echo "Running $@"
    unbuffer "$@" > /tmp/sscanf_test_out.txt || (cat /tmp/sscanf_test_out.txt && return 1)
}

# main tests
pushd ~/projects/sscanf
try_silent cargo update || exit 1
try_silent cargo +stable test || exit 1
try_silent cargo +nightly test || exit 1
try_silent cargo +nightly doc --no-deps || exit 1
popd

# old rustc version
pushd ~/projects/sscanf_old_rustc
try_silent cargo +1.56.0 test -- --skip failing_tests || exit 1
popd

# minimum version
pushd ~/projects/sscanf_min_version
try_silent cargo +nightly -Z minimal-versions update || exit 1

try_silent cargo +stable test -- --skip failing_tests || exit 1
try_silent cargo +nightly test -- --skip failing_tests || exit 1
popd
