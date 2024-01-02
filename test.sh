#!/bin/bash

OVERWRITE=0
if [[ "$1" == "overwrite" ]]; then
    OVERWRITE=1
elif [[ -n "$1" ]]; then
    echo "Usage: $0 [overwrite]"
    exit 1
fi

TMP_FILE="/tmp/sscanf_test_out.txt"

function handle_output {
    rm -f "${TMP_FILE}"
    while read -r line
    do
        echo "${line}" >> "${TMP_FILE}"

        # make sure line is not longer than the terminal width
        WIDTH=$(tput cols) # read this again in case the terminal was resized
        WIDTH=$((WIDTH - 3)) # leave space for the "..."
        TRIMMED_LINE=$(echo "> ${line}" | sed "s/\(.\{${WIDTH}\}\).*/\1.../")
        echo -en "\033[2K\r${TRIMMED_LINE}"
        tput init # trimmed line may have messed up coloring
    done
    echo -ne "\033[2K\r";
}

function try_silent {
    echo "Running $*"
    unbuffer "$@" | handle_output
    if [[ ${PIPESTATUS[0]} -ne 0 ]]; then
        cat "${TMP_FILE}"
        return 1
    fi
}

BASE_DIR="$(realpath "$(dirname "$0")")"
OUT_DIRS="${BASE_DIR}/test_dirs"
MSRV_DIR="${OUT_DIRS}/msrv"
MIN_VERSIONS_DIR="${OUT_DIRS}/min_versions"

for dir in "${MSRV_DIR}" "${MIN_VERSIONS_DIR}"; do
    [[ -d "${dir}" ]] && continue
    mkdir -p "${dir}"
    ln -s "${BASE_DIR}/Cargo.toml" "${dir}/Cargo.toml"
    ln -s "${BASE_DIR}/src" "${dir}/src"
    ln -s "${BASE_DIR}/tests" "${dir}/tests"
    ln -s "${BASE_DIR}/sscanf_macro" "${dir}/sscanf_macro"
done

# main tests
(
    cd "${BASE_DIR}" || (echo "Failed to cd to ${BASE_DIR}"; exit 1)
    try_silent cargo update || exit 1
    try_silent cargo +stable test || exit 1
    try_silent cargo +nightly test || exit 1

    if [[ OVERWRITE -eq 1 ]]; then
        echo "Trybuild overwrite mode enabled"
        export TRYBUILD=overwrite
        try_silent cargo +stable test error_message_tests -- --ignored || exit 1
        git add tests/fail # stage overwrite changes first, in case `nightly` would undo them
        try_silent cargo +nightly test error_message_tests -- --ignored || exit 1
    else
        try_silent cargo +stable test error_message_tests -- --ignored || exit 1
        try_silent cargo +nightly test error_message_tests -- --ignored || exit 1
    fi
    export RUSTDOCFLAGS="-D warnings"
    try_silent cargo +nightly doc --no-deps || exit 1
    try_silent cargo +nightly clippy -- -D warnings || exit 1
    try_silent cargo +stable fmt --check || exit 1
) || exit 1

(
    cd "${BASE_DIR}/sscanf_macro" || (echo "Failed to cd to ${BASE_DIR}/sscanf_macro"; exit 1)
    try_silent cargo +nightly clippy -- -D warnings || exit 1
    try_silent cargo +stable fmt --check || exit 1
) || exit 1

# minimum supported rust version
(
    cd "${MSRV_DIR}" || (echo "Failed to cd to ${MSRV_DIR}"; exit 1)
    try_silent cargo +1.56.0 test || exit 1
) || exit 1

# minimum versions
(
    cd "${MIN_VERSIONS_DIR}" || (echo "Failed to cd to ${MIN_VERSIONS_DIR}"; exit 1)
    try_silent cargo +nightly -Z minimal-versions update || exit 1

    try_silent cargo +stable test || exit 1
    try_silent cargo +nightly test || exit 1
) || exit 1

echo "All tests passed!"
