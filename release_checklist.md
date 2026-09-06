# Release Checklist

- Run `test.sh`
  - Coverage is acceptable
- Resolve any TODOs and FIXMEs
- Check Cargo.toml
  - Version updated in a semver-correct way.
  - No outdated dependencies.
- Check documentation
  - Everything up-to-date.
  - Sufficient examples.
    - Any `compile_fail` examples are copied verbatim to `tests/fail/{stable,nightly}/compile_fail_doctests.rs`
      and produce the error message that is advertised.
  - Links point to the correct target.
  - No spelling/grammar mistakes (AI is really helpful here).
  - No overly long paragraphs.
- Check Changelog.md
  - Includes the to-be-released version
  - Includes all changes that might affect the user
  - Is organized in the <https://keepachangelog.com/en/1.1.0/> style
  - Contains version-specific links
  - Contains the current date behind the version header (only done on the day of release)
