# Copilot Instructions for `sscanf` Codebase

## Overview
The `sscanf` project is a Rust library that provides functionality similar to the `sscanf` function in C. It includes procedural macros for parsing strings based on format strings and custom types. The project is structured into multiple components, including the main library (`src/`), macro definitions (`sscanf_macro/`), and various test modules.

## Key Components
- **`src/`**: Contains the core library code, including traits like `FromScanf` and utilities for parsing.
- **`sscanf_macro/`**: Defines procedural macros like `sscanf!` and `sscanf_unescaped!`.
- **`tests/`**: Includes test cases for various scenarios, organized into subdirectories like `derive/`, `fail/`, and `types/`.
- **`submodules/test_script/`**: Contains scripts for running tests and verifying compatibility with different Rust versions.

## Developer Workflows
### Checking for compile errors
Use Cargo to check the project:
```bat
cargo check
```

### Running Tests
Run all tests, including doctests (linux/wsl only):
```sh
./test.sh
```

### Generating Documentation
Generate and open the documentation (windows):
```bat
doc.bat
```

## Style Guide
- Follow Rust's standard style guidelines.
- Inside documentation comments (lines starting with `///`):
  - Use valid markdown.
  - Ensure that lines are no longer than 120 characters.
  - Format paragraphs so that they fill out the 120 character limit as much as possible.
  - Keep intra-document links as simple as possible, e.g. ``[`MyType`]`` should be preferred if MyType is in scope,
    otherwise use ``[`single_path::MyType`]`` or ``[`MyType`](longer::path::to::MyType)``. Only use absolute links for non-rust links.
  - Code blocks (lines surrounded by triple backticks within documentation comments) should:
    - Contain valid Rust code.
    - Ensure the total line length does not exceed 100 characters.
    - Hide lines that are not relevant to the example by starting them with `# `.
      - If functionality is hidden by this, add a non-hidden comment mentioning what was hidden, e.g. `// ...your implementation here...`.
- Use Rust's new inlined variable syntax in format strings, e.g. `format!("{variable}")` instead of `format!("{}", variable)`.
