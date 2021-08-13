if "%RUSTDOCFLAGS%"=="" (
    cargo +stable test && cargo +stable test --all-features && cargo +nightly test && cargo +nightly test --all-features
) else (
    echo Cannot run tests in doc terminal
)