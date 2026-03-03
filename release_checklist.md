# Release Checklist

- Run `test.sh`
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
