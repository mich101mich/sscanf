if "%DOC_OPEN%"=="" (
    cargo +nightly doc --open
    set DOC_OPEN="yes"
) else (
    cargo +nightly doc
)
