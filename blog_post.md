# Array of Structures vs Structure of Arrays: The Performance Gap You Can't Ignore

When building high-performance systems, data layout matters more than you think. We benchmarked two common patterns: Array of Structures (AoS) and Structure of Arrays (SoA) — and the results are striking.

**AoS** is the intuitive approach: an array of objects, each containing multiple fields. Think `Vec<Particle>` where each particle has x, y, z coordinates.

**SoA** flips this: separate arrays for each field. Instead of one array of particles, you have separate `Vec<f32>` for x, y, and z coordinates.

## The Results

For 100,000 particles, **SoA is 5.3x faster** than AoS (15µs vs 80µs). But it gets better: with SIMD optimizations, SoA reaches **6.5 million elements/second**, while AoS with SIMD is often *slower* than scalar AoS due to strided memory access.

## Why SoA Wins

**Cache locality**: SoA keeps related data contiguous, filling cache lines efficiently. AoS scatters data across memory.

**Auto-vectorization**: Compilers can automatically apply SIMD to SoA. With AoS, manual SIMD often hurts performance due to gather/scatter operations.

**Conclusion**: For data-parallel workloads, SoA isn't just faster — it's dramatically faster. The numbers don't lie: structure your arrays, don't array your structures.
