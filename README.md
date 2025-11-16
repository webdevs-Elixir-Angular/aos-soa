# SoA vs AoS Performance Benchmarks

Benchmarks comparing Structure of Arrays (SoA) vs Array of Structures (AoS) performance in Rust.

## Running Benchmarks

### Run all benchmarks
```bash
cargo bench
```

### Run with native CPU optimizations (recommended)
```bash
RUSTFLAGS='-C target-cpu=native' cargo bench
```

### Run specific benchmark files
```bash
cargo bench --bench my_benchmark
cargo bench --bench auto_optimisation_benchmark
```

### Run selective benchmarks by name
```bash
# Run only AoS benchmarks
cargo bench "AoS"

# Run only SoA benchmarks
cargo bench "SoA"

# Run only SIMD benchmarks
cargo bench "SIMD"

# Run SIMD width comparison
cargo bench "SIMD x"

# Run specific benchmark
cargo bench "SoA Update (zip)"
```

## Benchmark Files

- **my_benchmark.rs**: Core SoA vs AoS comparison including SIMD variants (FIXED: proper black_box usage)
- **auto_optimisation_benchmark.rs**: Demonstrates the importance of black_box() (FIXED: shows broken vs correct usage)
- **improved_benchmark.rs**: ⭐ **RECOMMENDED** - Comprehensive benchmarks with all fixes and improvements

## What's Being Tested

### Basic Comparisons
- **AoS Update**: Traditional struct-based particle updates
- **SoA Update**: Array-based particle updates (better cache locality)
- **SoA Update (zip)**: Idiomatic Rust with iterator zipping

### SIMD Variants (improved_benchmark.rs)
- **SSE (x4)**: 128-bit SIMD (always available on x86_64)
- **AVX (x8)**: 256-bit SIMD (runtime detected)
- **AVX-512 (x16)**: 512-bit SIMD (runtime detected)

### Parameterized Tests (improved_benchmark.rs)
- **1,000 elements**: Tests L1 cache behavior (~4KB)
- **10,000 elements**: Tests L2 cache behavior (~40KB)
- **100,000 elements**: Tests L3 cache behavior (~400KB)
- **1,000,000 elements**: Tests RAM-bound performance (~4MB)

## Key Improvements ✅

### Fixed Issues:
1. ✅ **Proper `black_box()` usage** - Returns values instead of references
2. ✅ **Runtime SIMD detection** - Uses `is_x86_feature_detected!()` instead of compile-time checks
3. ✅ **Throughput measurements** - Shows elements/second processed
4. ✅ **Parameterized benchmarks** - Tests multiple data sizes for cache effects
5. ✅ **Better configuration** - Longer measurement times, proper warmup

### What Was Wrong:
- ❌ `black_box(&data)` - Only pins pointer, doesn't prevent computation elimination
- ❌ `#[cfg(target_feature = "avx512f")]` - Compile-time check that fails on most CPUs
- ❌ No throughput metrics - Hard to compare absolute performance
- ❌ Single data size - Doesn't show cache effects
