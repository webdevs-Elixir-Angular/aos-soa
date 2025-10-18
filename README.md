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

- **my_benchmark.rs**: Core SoA vs AoS comparison including SIMD variants
- **auto_optimisation_benchmark.rs**: Tests compiler optimization behavior

## What's Being Tested

- **AoS Update**: Traditional struct-based particle updates
- **SoA Update**: Array-based particle updates (better cache locality)
- **SoA Update (zip)**: Idiomatic Rust with iterator zipping
- **SIMD variants**: Manual SIMD implementations at different widths (x4, x8, x16, x32)
