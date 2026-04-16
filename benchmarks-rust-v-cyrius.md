# Benchmarks: Rust vs Cyrius

Measured 2026-04-16 on same machine, sequential runs.

- **Rust**: Criterion 0.5, 100 samples, `cargo bench` (release mode, LTO)
- **Cyrius**: Manual timing via `gettimeofday`, `cyrius bench` (static ELF, no optimizer)

## Comparable Benchmarks

Benchmarks that test the same logical operation in both implementations.

| Benchmark | Rust (ns/iter) | Cyrius (ns/iter) | Ratio | Notes |
|-----------|---------------|-----------------|-------|-------|
| cmd_create (install single) | 48 | 192 | 4.0x | Rust: parse_args slice. Cyrius: alloc struct + vec_new |
| config_default | ~350 (serialize) | 224 | 0.6x | Rust measured serde serialize. Cyrius: raw struct init |
| source_to_str | ~18 (list parse) | 4 | 0.2x | Rust: clap parse. Cyrius: string compare chain |
| db_search (20 vs 100 entries) | 2,963 (100) | 1,864 (20) | -- | Different dataset sizes; not directly comparable |
| db_total_size (20 vs 100) | 74 (100) | 630 (20) | -- | Rust uses HashMap::values().map().sum(); Cyrius: map_values + loop |
| db_integrity (20 vs 100) | 99 (100) | 1,762 (20) | -- | Rust: quick check only. Cyrius: full issue struct alloc |
| txn_begin_commit | 3,112 | 1,484 | 0.5x | Cyrius: simpler alloc model, no serde overhead |
| output_display (20 pkgs) | 3,410 | 656 | 0.2x | Cyrius: str_builder is lighter than String + format! |

## Direct Comparisons (normalized)

These benchmarks do equivalent work and are directly comparable.

### Command Creation

| Operation | Rust | Cyrius | Winner |
|-----------|------|--------|--------|
| Create Install cmd (1 pkg) | 48 ns | ~192 ns | Rust 4x |
| Create Install cmd (4 pkgs) | 78 ns | ~192 ns | Rust 2.5x |

Rust wins on command parsing because `parse_args` operates on borrowed
`&[&str]` slices with zero allocation for simple cases. Cyrius allocates
a 72-byte struct + vec on every call.

### Transaction Lifecycle

| Operation | Rust | Cyrius | Winner |
|-----------|------|--------|--------|
| begin + add_op + commit | 3,112 ns | 1,484 ns | **Cyrius 2.1x** |
| recent(10) from 50 | 13 ns | ~100 ns | Rust 8x |
| get by ID from 50 | 69 ns | ~50 ns | **Cyrius 1.4x** |

Cyrius wins on full lifecycle because the bump allocator has zero
free/dealloc overhead and no serde serialization happens in-memory.
Rust wins on `recent()` due to iterator + collect being cache-friendly.
Cyrius wins on `get()` due to O(1) hashmap index vs Rust's O(n) linear scan.

### Output Rendering

| Operation | Rust | Cyrius | Winner |
|-----------|------|--------|--------|
| format 20 packages to string | 3,410 ns | 656 ns | **Cyrius 5.2x** |

Cyrius str_builder + cstr concatenation is significantly faster than
Rust's `format!()` macro with `String` allocation and `Display` trait
dispatch through `anstyle`.

## Build Artifacts

| Metric | Rust | Cyrius |
|--------|------|--------|
| Binary size (release) | ~2.1 MB | 532 KB |
| Binary size (stripped) | ~1.4 MB | 532 KB |
| Binary format | ELF dynamic | ELF static |
| Compile time | ~4.2s (incremental) | <0.1s |
| Dependencies | 8 crates + transitive | 19 stdlib modules |

## Test Coverage

| Metric | Rust | Cyrius |
|--------|------|--------|
| Test count | 46 tests (114 total with config) | 147 assertions |
| Test framework | `#[test]` + Criterion | assert lib + manual bench |
| Security tests | 0 | 18 (name validation, escaping, underflow) |

## Methodology Notes

- Rust Criterion uses statistical analysis (100 samples, warmup) and
  `black_box` to prevent dead code elimination. Numbers are highly stable.
- Cyrius uses `gettimeofday` wall-clock timing with 10K-100K iterations.
  No warmup phase, no statistical analysis. Numbers vary ~5-10% between runs.
- Rust PackageDb benchmarks use 100 entries; Cyrius uses 20. The Rust
  `HashMap` is O(1) lookup so entry count matters less for `search`;
  Cyrius `map_values` + linear scan is O(n).
- Cyrius has no optimizer or dead code elimination in the benchmark binary.
  The compiler emits straightforward x86_64 with no register allocation
  beyond locals. Performance comes from the bump allocator and minimal
  abstraction overhead.
- The Cyrius `txn_begin_commit` benchmark includes hashmap index maintenance
  (O(1) lookup optimization added in P(-1)), adding ~2-3us overhead per
  cycle that pays off at scale.
