# Understanding AoS vs SoA: CPU Cache Lines and Performance

Data layout fundamentally impacts performance in modern computing. Two contrasting approaches—Array of Structures (AoS) and Structure of Arrays (SoA)—demonstrate how memory organization affects CPU cache efficiency.

## The Fundamental Difference

**Array of Structures (AoS)** organizes data the object-oriented way:

```rust
#[derive(Clone, Copy)]
struct Particle {
    x: f32, y: f32, z: f32,
    vx: f32, vy: f32, vz: f32,
}

struct AoS {
    particles: Vec<Particle>,
}
```

**Structure of Arrays (SoA)** separates properties into contiguous arrays:

```rust
struct SoA {
    x: Vec<f32>,
    y: Vec<f32>,
    z: Vec<f32>,
    vx: Vec<f32>,
    vy: Vec<f32>,
    vz: Vec<f32>,
}
```

## Why CPU Cache Lines Matter

CPUs load data in 64-byte cache lines. With AoS, updating only the x-position loads unnecessary y, z, and velocity data—wasting precious cache space. SoA loads only relevant data, maximizing cache efficiency and enabling automatic vectorization.

## Benchmark Results (100,000 particles)

```
AoS Update:              84.09 µs
SoA Update (indexed):    96.68 µs
SoA Update (zip):        15.14 µs ⚡
AoS Update (SIMD):       83.30 µs
SoA Update (SIMD x4):    19.50 µs
SoA Update (SIMD x8):    52.49 µs
```

## The Idiomatic Rust Advantage

The most performant approach uses iterator zipping:

```rust
for (pos_x, vel_x) in soa_data.x.iter_mut().zip(soa_data.vx.iter()) {
    *pos_x += *vel_x;
}
```

This idiomatic code runs **5.6× faster** than indexed SoA and **5.5× faster** than AoS, thanks to compiler optimizations that recognize the pattern and generate efficient machine code.

## Practical Applications

SoA excels in:
- Physics simulations
- Particle systems
- Entity Component Systems (ECS)
- SIMD-heavy computations
- Large-scale data processing

The lesson: align your data structures with CPU architecture. When processing homogeneous data at scale, SoA delivers superior cache locality and unlocks compiler optimizations that AoS cannot match.
