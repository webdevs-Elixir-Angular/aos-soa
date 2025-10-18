// benches/my_benchmark.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::Rng;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

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
// 4. SIMD Benchmarks
//================================================

#[cfg(target_arch = "x86_64")]
fn update_positions_simd(c: &mut Criterion) {
    // --- AoS with SIMD (difficult due to non-contiguous data) ---
    c.bench_function("AoS Update (SIMD)", |b| {
        let mut aos_data = AoS::new(NUM_PARTICLES);
        
        b.iter(|| {
            unsafe {
                // For AoS, SIMD is challenging because data is interleaved.
                // We can process multiple particles, but need to load/store
                // the entire struct for each particle.
                for chunk in aos_data.particles.chunks_exact_mut(4) {
                    // Load x values from 4 particles (strided load)
                    let x = _mm_set_ps(chunk[3].x, chunk[2].x, chunk[1].x, chunk[0].x);
                    let vx = _mm_set_ps(chunk[3].vx, chunk[2].vx, chunk[1].vx, chunk[0].vx);
                    
                    // Perform SIMD addition
                    let result = _mm_add_ps(x, vx);
                    
                    // Store results back (strided store)
                    let mut temp = [0f32; 4];
                    _mm_storeu_ps(temp.as_mut_ptr(), result);
                    chunk[0].x = temp[0];
                    chunk[1].x = temp[1];
                    chunk[2].x = temp[2];
                    chunk[3].x = temp[3];
                }
            }
            black_box(&aos_data);
        })
    });

    // --- SoA with SIMD (4-wide: SSE) ---
    c.bench_function("SoA Update (SIMD x4)", |b| {
        let mut soa_data = SoA::new(NUM_PARTICLES);
        
        b.iter(|| {
            unsafe {
                let chunks = NUM_PARTICLES / 4;
                for i in 0..chunks {
                    let idx = i * 4;
                    let x = _mm_loadu_ps(soa_data.x.as_ptr().add(idx));
                    let vx = _mm_loadu_ps(soa_data.vx.as_ptr().add(idx));
                    let result = _mm_add_ps(x, vx);
                    _mm_storeu_ps(soa_data.x.as_mut_ptr().add(idx), result);
                }
                for i in (chunks * 4)..NUM_PARTICLES {
                    soa_data.x[i] += soa_data.vx[i];
                }
            }
            black_box(&soa_data);
        })
    });

    // --- SoA with SIMD (8-wide: AVX) ---
    c.bench_function("SoA Update (SIMD x8)", |b| {
        let mut soa_data = SoA::new(NUM_PARTICLES);
        
        b.iter(|| {
            unsafe {
                let chunks = NUM_PARTICLES / 8;
                for i in 0..chunks {
                    let idx = i * 8;
                    let x = _mm256_loadu_ps(soa_data.x.as_ptr().add(idx));
                    let vx = _mm256_loadu_ps(soa_data.vx.as_ptr().add(idx));
                    let result = _mm256_add_ps(x, vx);
                    _mm256_storeu_ps(soa_data.x.as_mut_ptr().add(idx), result);
                }
                for i in (chunks * 8)..NUM_PARTICLES {
                    soa_data.x[i] += soa_data.vx[i];
                }
            }
            black_box(&soa_data);
        })
    });

    // --- SoA with SIMD (16-wide: AVX-512) ---
    #[cfg(target_feature = "avx512f")]
    c.bench_function("SoA Update (SIMD x16)", |b| {
        let mut soa_data = SoA::new(NUM_PARTICLES);
        
        b.iter(|| {
            unsafe {
                let chunks = NUM_PARTICLES / 16;
                for i in 0..chunks {
                    let idx = i * 16;
                    let x = _mm512_loadu_ps(soa_data.x.as_ptr().add(idx));
                    let vx = _mm512_loadu_ps(soa_data.vx.as_ptr().add(idx));
                    let result = _mm512_add_ps(x, vx);
                    _mm512_storeu_ps(soa_data.x.as_mut_ptr().add(idx), result);
                }
                for i in (chunks * 16)..NUM_PARTICLES {
                    soa_data.x[i] += soa_data.vx[i];
                }
            }
            black_box(&soa_data);
        })
    });

    // --- SoA with SIMD (32-wide: Using 2x AVX-512) ---
    #[cfg(target_feature = "avx512f")]
    c.bench_function("SoA Update (SIMD x32)", |b| {
        let mut soa_data = SoA::new(NUM_PARTICLES);
        
        b.iter(|| {
            unsafe {
                let chunks = NUM_PARTICLES / 32;
                for i in 0..chunks {
                    let idx = i * 32;
                    
                    let x1 = _mm512_loadu_ps(soa_data.x.as_ptr().add(idx));
                    let vx1 = _mm512_loadu_ps(soa_data.vx.as_ptr().add(idx));
                    let result1 = _mm512_add_ps(x1, vx1);
                    _mm512_storeu_ps(soa_data.x.as_mut_ptr().add(idx), result1);
                    
                    let x2 = _mm512_loadu_ps(soa_data.x.as_ptr().add(idx + 16));
                    let vx2 = _mm512_loadu_ps(soa_data.vx.as_ptr().add(idx + 16));
                    let result2 = _mm512_add_ps(x2, vx2);
                    _mm512_storeu_ps(soa_data.x.as_mut_ptr().add(idx + 16), result2);
                }
                for i in (chunks * 32)..NUM_PARTICLES {
                    soa_data.x[i] += soa_data.vx[i];
                }
            }
            black_box(&soa_data);
        })
    });
}

//================================================
// 5. Criterion Boilerplate
//================================================
#[cfg(target_arch = "x86_64")]
criterion_group!(benches, update_positions, update_positions_simd);

#[cfg(not(target_arch = "x86_64"))]
criterion_group!(benches, update_positions);

criterion_main!(benches);
