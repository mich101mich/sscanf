if "%DOC_OPEN%"=="" (
    set DOC_OPEN="yes"
    cargo +nightly doc --no-deps --open
) else (
    cargo +nightly doc --no-deps
)
