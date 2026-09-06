# Benchmark 0.4.4 -> 0.5.0

Raw data source: hyperfine runs from [build_benches.ps1](./build_benches.ps1)

## Benchmark setup

- "Simple" is a `sscanf` call that just parses two numbers from a short string.
- "Complex" is a `sscanf` call with a decently large type hierarchy, parsing 30 fields across 7 structs/enums.

See [src/main.rs](src/main.rs) for the implementation

**Note that the actual numbers in this benchmark are highly fluctuating, and the actual results depend on the exact types and inputs that are used.**

## Measurements

### Single Parse (`sscanf!` once)

| Benchmark | 0.4.4 | 0.5.0 | Relative result |
| --------- | ----: | ----: | --------------- |
| simple | 12.2 ms | 9.5 ms | `0.5.0` is `1.29x` faster (`-22.1%` time) |
| complex | 20.0 ms | 13.0 ms | `0.5.0` is `1.53x` faster (`-35.0%` time) |

### Multiple parse calls

This setup calls `sscanf` / `Parser::parse` 1000 times in a loop.

| Benchmark | 0.4.4 | 0.5.0 | 0.5.0 (with parser reuse) |
| --------- | ----: | ----: | ------------------------: |
| simple | 14.1 ms | 60.6 ms | 11.4 ms |
| complex | 168.0 ms | 944.0 ms | 23.5 ms |

## Results

- For single `sscanf` calls, version 0.5.0 is 20-35% faster
- For multiple calls, 0.4.4 contains built-in caching of the Parser, so it can be many times faster than just
  calling `sscanf` in a loop in 0.5.0 (up to 6x slower)
  - However, in return, 0.5.0 allows explicit `Parser` caching, which ends up being even more efficient
    (7x faster than 0.4.4 and incomparably faster than repeated-call 0.5.0)
