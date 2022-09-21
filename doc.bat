if "%DOC_OPEN%"=="" (
    set DOC_OPEN="yes"
    cargo +nightly doc --open
) else (
    cargo +nightly doc
)
