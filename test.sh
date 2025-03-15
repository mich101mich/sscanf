#!/bin/bash

set -e

########
# functions
########

TMP_FILE="/tmp/sscanf_test_out.txt"
function handle_output {
    rm -f "${TMP_FILE}"
    while IFS='' read -r line
    do
        echo "${line}" >> "${TMP_FILE}"

        echo -en "\033[2K\r"
        echo -n "$(cut -c "1-$(tput cols)" <<< "> ${line}")"
    done
    echo -en "\033[2K\r";
    tput init # Reset any coloring
}

function try_silent {
    echo "Running $*"
    unbuffer "$@" 2>&1 | handle_output
    if [[ ${PIPESTATUS[0]} -ne 0 ]]; then
        cat "${TMP_FILE}"
        return 1
    fi
}

function assert_no_change {
    DIR="$1"
    if ! git diff-files --quiet --ignore-cr-at-eol "${DIR}"; then
        >&2 echo "Changes in ${DIR} detected, aborting"
        exit 1
    fi
    if [[ -n "$(git ls-files --exclude-standard --others "${DIR}")" ]]; then
        >&2 echo "Untracked files in ${DIR} detected, aborting"
        exit 1
    fi
}

########
# setup
########
BASE_DIR="$(realpath "$(dirname "$0")")"

OVERWRITE=0
if [[ "$1" == "overwrite" ]]; then
    OVERWRITE=1
elif [[ -n "$1" ]]; then
    echo "Usage: $0 [overwrite]"
    exit 1
fi

MSRV=$(sed -n -r -e 's/^rust-version = "(.*)"$/\1/p' "${BASE_DIR}/Cargo.toml")
if [[ -z "${MSRV}" ]]; then
    >&2 echo "Could not determine minimum supported Rust version. Missing 'rust-version' in Cargo.toml"
    exit 1
fi
echo "Minimum supported Rust version: ${MSRV}"

TARGET_DIR="${BASE_DIR}/target"
MSRV_DIR="${TARGET_DIR}/msrv_${MSRV}"
MIN_VERSIONS_DIR="${TARGET_DIR}/min_versions"

for dir in "${MSRV_DIR}" "${MIN_VERSIONS_DIR}"; do
    [[ -d "${dir}" ]] && continue
    mkdir -p "${dir}"
    ln -s "../../Cargo.toml" "${dir}/Cargo.toml"
    ln -s "../../src" "${dir}/src"
    ln -s "../../tests" "${dir}/tests"
    ln -s "../../sscanf_macro" "${dir}/sscanf_macro"
done

export RUSTFLAGS="-D warnings"
export RUSTDOCFLAGS="-D warnings"

########
# main tests
########
cd "${BASE_DIR}"
try_silent rustup update
try_silent cargo update
try_silent cargo +stable test
try_silent cargo +nightly test

if [[ OVERWRITE -eq 1 ]]; then
    echo "Trybuild overwrite mode enabled"
    export TRYBUILD=overwrite

    # "overwrite" will (as the name implies) overwrite any incorrect output files in the error_message_tests.
    # There is however the problem that the stable and nightly versions might have different outputs. If they
    # are simply run one after the other, then the second one will overwrite the first one. To avoid this, we
    # use git to check if the files have changed after every step.
    assert_no_change "tests/fail/**/*.stderr" # Check for initial changes that would skew the later checks

    try_silent cargo +stable test error_message_tests -- --ignored
    assert_no_change "tests/fail/**/*.stderr"

    try_silent cargo +nightly test error_message_tests -- --ignored
    assert_no_change "tests/fail/**/*.stderr"
else
    try_silent cargo +stable test error_message_tests -- --ignored
    try_silent cargo +nightly test error_message_tests -- --ignored
fi
try_silent cargo +nightly doc --no-deps
try_silent cargo +nightly clippy -- -D warnings
try_silent cargo +stable fmt --check


########
# sscanf_macro subdirectory
########
cd "${BASE_DIR}/sscanf_macro"
try_silent cargo +nightly clippy -- -D warnings
try_silent cargo +stable fmt --check

########
# minimum supported rust version
########
cd "${MSRV_DIR}"
try_silent rustup install "${MSRV}"
ORIGINAL_RUSTFLAGS="${RUSTFLAGS}"
RUSTFLAGS="${RUSTFLAGS} --cfg msrv_build"
try_silent cargo "+${MSRV}" test --tests # only run --tests, which excludes the doctests from Readme.md
RUSTFLAGS="${ORIGINAL_RUSTFLAGS}"

########
# minimum versions
########
cd "${MIN_VERSIONS_DIR}"
try_silent cargo +nightly -Z minimal-versions update

try_silent cargo +stable test
try_silent cargo +nightly test

########
echo "All tests passed!"
