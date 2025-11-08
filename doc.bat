if "%DOC_OPEN%"=="" (
    cargo +nightly doc --open
    if %ERRORLEVEL%==0 set DOC_OPEN="yes"
) else (
    cargo +nightly doc
)
