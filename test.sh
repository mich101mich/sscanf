#!/bin/bash

mkdir -p target

function try_silent {
    echo "Running $@"
    unbuffer "$@" > target/out.txt || (cat target/out.txt && return 1)
}

# main tests
try_silent cargo update || exit 1
try_silent cargo +stable test || exit 1
try_silent cargo +nightly test || exit 1
try_silent cargo +nightly doc --no-deps || exit 1

# old rustc version
try_silent cargo +1.56.0 test --target-dir target/old_rustc -- --skip failing_tests || exit 1

# minimum version
pushd ~/projects/sscanf
./test
popd