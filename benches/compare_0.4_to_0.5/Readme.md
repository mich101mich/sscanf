# Benchmark 0.4.4 -> 0.5.0

Raw data source: PowerShell `Measure-Command` runs from
[build_benches.ps1](./build_benches.ps1). The tables show average elapsed time per process run
(`TotalMilliseconds / Runs`). Single cases use 1,000 measured runs and multi cases use 100 measured
runs, after 50 and 5 warmup runs respectively.

## Benchmark setup

- "Simple" is a `sscanf` call that just parses two numbers from a short string.
- "Complex" is a `sscanf` call with a decently large type hierarchy, parsing 30 fields across 7 structs/enums.
- "Complex no numbers" uses the same type hierarchy and field count as "Complex", but all numeric-looking
  values are parsed as strings.
- The separate "Complex no numbers" case helps isolate general parsing costs from the number-parsing
  improvements in version 0.5.0.

See [src/main.rs](src/main.rs) for the implementation

**Note that the actual numbers in this benchmark are highly fluctuating, and the actual results depend on the exact types and inputs that are used.**

## Measurements

### Single Parse (`sscanf!` once)

| Benchmark | 0.4.4 | 0.5.0 | Relative result |
| --------- | ----: | ----: | --------------- |
| simple | 11.56 ms | 8.75 ms | `0.5.0` is `1.32x` faster (`-24.4%` time) |
| complex | 17.30 ms | 10.46 ms | `0.5.0` is `1.65x` faster (`-39.5%` time) |
| complex no numbers | 10.04 ms | 10.18 ms | `0.5.0` is `1.4%` slower (`+1.4%` time) |

### Multiple parse calls

Each multi binary is measured 100 times. The averages below are per process run and distinguish direct
repeated `sscanf` calls from explicit `Parser` reuse.

| Benchmark | 0.4.4 | 0.5.0 (without parser reuse) | 0.5.0 (with parser reuse) |
| --------- | ----: | ----: | ------------------------: |
| simple | 11.45 ms | 59.64 ms | 9.02 ms |
| complex | 158.28 ms | 876.89 ms | 21.23 ms |
| complex no numbers | 19.55 ms | 908.18 ms | 19.85 ms |

### Parser reuse comparison

| Benchmark | 0.5.0 with parser vs 0.4.4 | 0.5.0 without parser vs 0.4.4 | With parser vs without parser |
| --------- | --------------------------: | -----------------------------: | ---------------------------: |
| simple | `1.27x` faster (`-21.2%`) | `5.21x` slower (`+420.7%`) | `6.61x` faster (`-84.9%`) |
| complex | `7.46x` faster (`-86.6%`) | `5.54x` slower (`+454.0%`) | `41.30x` faster (`-97.6%`) |
| complex no numbers | `1.5%` slower (`+1.5%`) | `46.44x` slower (`+4544.7%`) | `45.76x` faster (`-97.8%`) |

### Number-parsing isolation

In this run, removing numeric conversions from the complex single case reduced 0.4.4 from 17.30 ms
to 10.04 ms, while 0.5.0 changed from 10.46 ms to 10.18 ms. This supports the purpose of the
additional case: the number-parsing improvements in 0.5.0 remove most of the numeric-conversion
overhead visible in 0.4.4.

## Results

- For single calls, 0.5.0 is 24-40% faster for the simple and complex number-parsing cases.
- The complex no-numbers single case is effectively unchanged, which isolates the number-parsing gain.
- Without parser reuse, 0.5.0 is 5.21x to 46.44x slower than 0.4.4 in the multi-run cases.
- With explicit parser reuse, 0.5.0 is 1.27x to 7.46x faster than 0.4.4 for the simple and complex
  cases, while the complex no-numbers result is essentially the same.
- Parser reuse makes 0.5.0 6.61x to 45.76x faster than repeated parsing without reuse.
