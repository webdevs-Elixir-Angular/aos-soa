// benches/auto_optimisation_benchmark.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
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
// The Benchmark Functions - Demonstrating black_box() importance
//================================================

fn check_compiler_optimizations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Compiler Optimizations");

    // --- AoS Benchmark WITHOUT black_box (will be optimized away!) ---
    group.bench_function("AoS WITHOUT black_box (BROKEN)", |b| {
        let mut aos_data = AoS::new(NUM_PARTICLES);
        b.iter(|| {
            // The update loop is performed
            for p in aos_data.particles.iter_mut() {
                p.x += p.vx;
            }
            // ❌ NO black_box - compiler can eliminate entire loop!
            // This will show extremely fast times (incorrectly)
        })
    });

    // --- AoS Benchmark WITH black_box (reference - still unreliable!) ---
    group.bench_function("AoS WITH black_box(&ref) (UNRELIABLE)", |b| {
        let mut aos_data = AoS::new(NUM_PARTICLES);
        b.iter(|| {
            for p in aos_data.particles.iter_mut() {
                p.x += p.vx;
            }
            // ⚠️ Passing reference - may or may not work
            black_box(&aos_data);
        })
    });

    // --- AoS Benchmark WITH black_box (correct - single value) ---
    group.bench_function("AoS WITH black_box(value) (CORRECT)", |b| {
        let mut aos_data = AoS::new(NUM_PARTICLES);
        b.iter(|| {
            for p in aos_data.particles.iter_mut() {
                p.x += p.vx;
            }
            // ✅ Return a value that depends on computation
            black_box(aos_data.particles[0].x)
        })
    });

    // --- SoA Benchmark WITHOUT black_box (will be optimized away!) ---
    group.bench_function("SoA WITHOUT black_box (BROKEN)", |b| {
        let mut soa_data = SoA::new(NUM_PARTICLES);
        b.iter(|| {
            for (pos_x, vel_x) in soa_data.x.iter_mut().zip(soa_data.vx.iter()) {
                *pos_x += *vel_x;
            }
            // ❌ NO black_box - entire loop gets eliminated
        })
    });

    // --- SoA Benchmark WITH black_box (reference - still unreliable!) ---
    group.bench_function("SoA WITH black_box(&ref) (UNRELIABLE)", |b| {
        let mut soa_data = SoA::new(NUM_PARTICLES);
        b.iter(|| {
            for (pos_x, vel_x) in soa_data.x.iter_mut().zip(soa_data.vx.iter()) {
                *pos_x += *vel_x;
            }
            // ⚠️ Passing reference - may or may not work
            black_box(&soa_data);
        })
    });

    // --- SoA Benchmark WITH black_box (correct - single value) ---
    group.bench_function("SoA WITH black_box(value) (CORRECT)", |b| {
        let mut soa_data = SoA::new(NUM_PARTICLES);
        b.iter(|| {
            for (pos_x, vel_x) in soa_data.x.iter_mut().zip(soa_data.vx.iter()) {
                *pos_x += *vel_x;
            }
            // ✅ Return first element to force computation
            black_box(soa_data.x[0])
        })
    });

    // --- SoA Benchmark WITH black_box (most robust - sum all values) ---
    group.bench_function("SoA WITH black_box(sum) (MOST ROBUST)", |b| {
        let mut soa_data = SoA::new(NUM_PARTICLES);
        b.iter(|| {
            for (pos_x, vel_x) in soa_data.x.iter_mut().zip(soa_data.vx.iter()) {
                *pos_x += *vel_x;
            }
            // ✅ Sum requires all elements to be computed
            black_box(soa_data.x.iter().sum::<f32>())
        })
    });

    group.finish();
}

//================================================
// Criterion Boilerplate
//================================================
criterion_group!(benches, check_compiler_optimizations);
criterion_main!(benches);
