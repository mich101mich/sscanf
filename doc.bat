if "%RUSTDOCFLAGS%"=="" (
    set RUSTDOCFLAGS=--cfg doc_cfg
    cargo +nightly doc --all-features --open
) else (
    cargo +nightly doc --all-features
)
