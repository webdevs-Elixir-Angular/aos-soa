// benches/my_benchmark.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::Rng;

const NUM_PARTICLES: usize = 100_000;

//================================================
// 1. Array of Structures (AoS) Definition
//================================================
// The intuitive, "object-oriented" approach.
#[derive(Clone, Copy)]
struct Particle {
    x: f32,
    y: f32,
    z: f32,
    vx: f32,
    vy: f32,
    vz: f32,
}

struct AoS {
    particles: Vec<Particle>,
}

impl AoS {
    fn new(count: usize) -> Self {
        let mut rng = rand::thread_rng();
        let particles = (0..count)
            .map(|_| Particle {
                x: rng.gen(),
                y: rng.gen(),
                z: rng.gen(),
                vx: rng.gen(),
                vy: rng.gen(),
                vz: rng.gen(),
            })
            .collect();
        Self { particles }
    }
}

//================================================
// 2. Structure of Arrays (SoA) Definition
//================================================
// The performance-oriented approach. All 'x' values are contiguous, etc.
struct SoA {
    x: Vec<f32>,
    y: Vec<f32>,
    z: Vec<f32>,
    vx: Vec<f32>,
    vy: Vec<f32>,
    vz: Vec<f32>,
}

impl SoA {
    fn new(count: usize) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            x: (0..count).map(|_| rng.gen()).collect(),
            y: (0..count).map(|_| rng.gen()).collect(),
            z: (0..count).map(|_| rng.gen()).collect(),
            vx: (0..count).map(|_| rng.gen()).collect(),
            vy: (0..count).map(|_| rng.gen()).collect(),
            vz: (0..count).map(|_| rng.gen()).collect(),
        }
    }
}

//================================================
// 3. The Benchmark Functions
//================================================

// This function simulates a simple physics update.
// We are only interested in updating the 'x' position.
// This is a common scenario in physics engines where you update one axis at a time.
fn update_positions(c: &mut Criterion) {
    // --- AoS Benchmark ---
    c.bench_function("AoS Update", |b| {
        // Setup the data for this specific benchmark run
        let mut aos_data = AoS::new(NUM_PARTICLES);

        // The black_box function prevents the compiler from optimizing away the code
        b.iter(|| {
            for p in aos_data.particles.iter_mut() {
                p.x += p.vx;
            }
            black_box(&aos_data); // Prevent the entire loop from being optimized away
        })
    });

    // --- SoA Benchmark ---
    c.bench_function("SoA Update", |b| {
        // Setup the data for this specific benchmark run
        let mut soa_data = SoA::new(NUM_PARTICLES);

        b.iter(|| {
            // Here we iterate over two separate, contiguous slices of memory.
            // This is ideal for the CPU's cache and for auto-vectorization.
            for i in 0..NUM_PARTICLES {
                soa_data.x[i] += soa_data.vx[i];
            }
            black_box(&soa_data); // Prevent the entire loop from being optimized away
        })
    });

    // SoA using zip for a more idiomatic Rust approach
    c.bench_function("SoA Update (zip)", |b| {
        let mut soa_data = SoA::new(NUM_PARTICLES);

        b.iter(|| {
            // .zip() creates an iterator that iterates over two slices at once.
            // Compilers are very good at optimizing this into the same efficient
            // machine code as the indexed loop.
            for (pos_x, vel_x) in soa_data.x.iter_mut().zip(soa_data.vx.iter()) {
                *pos_x += *vel_x;
            }
            black_box(&soa_data);
        })
    });
}

//================================================
// 4. Criterion Boilerplate
//================================================
criterion_group!(benches, update_positions);
criterion_main!(benches);
