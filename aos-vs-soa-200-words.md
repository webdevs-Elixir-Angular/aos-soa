# AoS vs SoA: A Performance Tale

When designing high-performance systems, data layout matters tremendously. Two fundamental approaches exist: Array of Structures (AoS) and Structure of Arrays (SoA).

**Array of Structures (AoS)** is the intuitive approach where each object bundles all its properties together. Think of a particle with position and velocity stored as a single struct in an array. This mirrors object-oriented thinking but creates scattered memory access patterns.

**Structure of Arrays (SoA)** flips this design, storing each property in separate contiguous arrays. All x-positions together, all velocities together. This layout aligns perfectly with CPU cache lines and SIMD operations.

Our benchmarks on 100,000 particles reveal dramatic differences:
- AoS Update: 84 µs
- SoA Update (indexed): 97 µs
- SoA Update (zip): 15 µs (5.6× faster!)
- SoA SIMD x4: 19.5 µs
- SoA SIMD x8: 52 µs

The surprising winner is idiomatic Rust code using iterator zipping, which enables aggressive compiler optimizations. The SoA layout combined with modern Rust idioms delivers exceptional performance without manual SIMD intrinsics.

For data-intensive applications—physics engines, particle systems, ECS architectures—choosing SoA can transform performance. The CPU cache loves predictable, contiguous memory access patterns.
