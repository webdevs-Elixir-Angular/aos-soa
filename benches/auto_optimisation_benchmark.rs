// benches/optimization_benchmark.rs

use criterion::{criterion_group, criterion_main, Criterion};
use rand::Rng;

const NUM_PARTICLES: usize = 100_000;

//================================================
// Data Structure Definitions (same as before)
//================================================

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
// The Benchmark Function (NO black_box)
//================================================

fn check_compiler_optimizations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Compiler Optimizations");

    // --- AoS Benchmark (Optimized Away?) ---
    group.bench_function("AoS Optimized Away?", |b| {
        // Data is created inside the loop
        let mut aos_data = AoS::new(NUM_PARTICLES);
        b.iter(|| {
            // The update loop is performed
            for p in aos_data.particles.iter_mut() {
                p.x += p.vx;
            }
            // BUT, aos_data is never read from or returned. Its final state is unused.
        })
    });

    // --- SoA Benchmark (Optimized Away?) ---
    group.bench_function("SoA Optimized Away?", |b| {
        // Data is created inside the loop
        let mut soa_data = SoA::new(NUM_PARTICLES);
        b.iter(|| {
            // The update loop is performed
            for (pos_x, vel_x) in soa_data.x.iter_mut().zip(soa_data.vx.iter()) {
                *pos_x += *vel_x;
            }
            // AGAIN, soa_data is never read from. Its final state has no observable effect.
        })
    });

    group.finish();
}

//================================================
// Criterion Boilerplate
//================================================
criterion_group!(benches, check_compiler_optimizations);
criterion_main!(benches);
