// benches/improved_benchmark.rs
// Improved benchmarks with proper black_box usage, parameterization, and throughput metrics

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rand::Rng;
use std::hint::black_box;
use std::time::Duration;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

//================================================
// Data Structure Definitions
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
// Parameterized Benchmarks with Throughput Metrics
//================================================

fn aos_vs_soa_parameterized(c: &mut Criterion) {
    // Test different sizes to observe cache effects
    let sizes = vec![
        1_000,      // L1 cache (4KB data)
        10_000,     // L2 cache (40KB data)
        100_000,    // L3 cache (400KB data)
        1_000_000,  // RAM-bound (4MB data)
    ];

    let mut group = c.benchmark_group("AoS vs SoA (Parameterized)");

    // Configure for more accurate measurements
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);
    group.warm_up_time(Duration::from_secs(3));

    for size in sizes.iter() {
        // Set throughput to measure elements/second
        group.throughput(Throughput::Elements(*size as u64));

        // AoS Update
        group.bench_with_input(BenchmarkId::new("AoS", size), size, |b, &size| {
            let mut aos_data = AoS::new(size);
            b.iter(|| {
                for p in aos_data.particles.iter_mut() {
                    p.x += p.vx;
                }
                // ✅ Proper black_box usage
                black_box(aos_data.particles[0].x)
            });
        });

        // SoA Update (indexed)
        group.bench_with_input(BenchmarkId::new("SoA (indexed)", size), size, |b, &size| {
            let mut soa_data = SoA::new(size);
            b.iter(|| {
                for i in 0..size {
                    soa_data.x[i] += soa_data.vx[i];
                }
                black_box(soa_data.x[0])
            });
        });

        // SoA Update (zip)
        group.bench_with_input(BenchmarkId::new("SoA (zip)", size), size, |b, &size| {
            let mut soa_data = SoA::new(size);
            b.iter(|| {
                for (pos_x, vel_x) in soa_data.x.iter_mut().zip(soa_data.vx.iter()) {
                    *pos_x += *vel_x;
                }
                black_box(soa_data.x[0])
            });
        });
    }

    group.finish();
}

//================================================
// SIMD Benchmarks with Runtime Detection
//================================================

#[cfg(target_arch = "x86_64")]
fn simd_benchmarks(c: &mut Criterion) {
    const SIZE: usize = 100_000;

    let mut group = c.benchmark_group("SIMD Comparisons");
    group.throughput(Throughput::Elements(SIZE as u64));
    group.measurement_time(Duration::from_secs(10));

    // Baseline: Scalar
    group.bench_function("Scalar", |b| {
        let mut soa_data = SoA::new(SIZE);
        b.iter(|| {
            for i in 0..SIZE {
                soa_data.x[i] += soa_data.vx[i];
            }
            black_box(soa_data.x[0])
        });
    });

    // SSE (128-bit, 4 floats) - Always available on x86_64
    group.bench_function("SIMD SSE (x4)", |b| {
        let mut soa_data = SoA::new(SIZE);
        b.iter(|| {
            unsafe {
                let chunks = SIZE / 4;
                let x_ptr = black_box(soa_data.x.as_mut_ptr());
                let vx_ptr = black_box(soa_data.vx.as_ptr());

                for i in 0..chunks {
                    let idx = i * 4;
                    let x = _mm_loadu_ps(x_ptr.add(idx));
                    let vx = _mm_loadu_ps(vx_ptr.add(idx));
                    let result = _mm_add_ps(x, vx);
                    _mm_storeu_ps(x_ptr.add(idx), result);
                }

                // Handle remainder
                for i in (chunks * 4)..SIZE {
                    soa_data.x[i] += soa_data.vx[i];
                }
            }
            black_box(soa_data.x[0])
        });
    });

    // AVX (256-bit, 8 floats) - Check at runtime
    if is_x86_feature_detected!("avx") {
        group.bench_function("SIMD AVX (x8)", |b| {
            let mut soa_data = SoA::new(SIZE);
            b.iter(|| {
                unsafe {
                    let chunks = SIZE / 8;
                    let x_ptr = black_box(soa_data.x.as_mut_ptr());
                    let vx_ptr = black_box(soa_data.vx.as_ptr());

                    for i in 0..chunks {
                        let idx = i * 8;
                        let x = _mm256_loadu_ps(x_ptr.add(idx));
                        let vx = _mm256_loadu_ps(vx_ptr.add(idx));
                        let result = _mm256_add_ps(x, vx);
                        _mm256_storeu_ps(x_ptr.add(idx), result);
                    }

                    // Handle remainder
                    for i in (chunks * 8)..SIZE {
                        soa_data.x[i] += soa_data.vx[i];
                    }
                }
                black_box(soa_data.x[0])
            });
        });
    } else {
        eprintln!("⚠️  AVX not detected - skipping AVX benchmarks");
    }

    // AVX-512 (512-bit, 16 floats) - Check at runtime
    if is_x86_feature_detected!("avx512f") {
        group.bench_function("SIMD AVX-512 (x16)", |b| {
            let mut soa_data = SoA::new(SIZE);
            b.iter(|| {
                unsafe {
                    let chunks = SIZE / 16;
                    let x_ptr = black_box(soa_data.x.as_mut_ptr());
                    let vx_ptr = black_box(soa_data.vx.as_ptr());

                    for i in 0..chunks {
                        let idx = i * 16;
                        let x = _mm512_loadu_ps(x_ptr.add(idx));
                        let vx = _mm512_loadu_ps(vx_ptr.add(idx));
                        let result = _mm512_add_ps(x, vx);
                        _mm512_storeu_ps(x_ptr.add(idx), result);
                    }

                    // Handle remainder
                    for i in (chunks * 16)..SIZE {
                        soa_data.x[i] += soa_data.vx[i];
                    }
                }
                black_box(soa_data.x[0])
            });
        });
    } else {
        eprintln!("⚠️  AVX-512 not detected - skipping AVX-512 benchmarks");
    }

    group.finish();
}

//================================================
// AoS SIMD Benchmark (showing why it's difficult)
//================================================

#[cfg(target_arch = "x86_64")]
fn aos_simd_comparison(c: &mut Criterion) {
    const SIZE: usize = 100_000;

    let mut group = c.benchmark_group("AoS SIMD (vs Scalar)");
    group.throughput(Throughput::Elements(SIZE as u64));

    // AoS Scalar
    group.bench_function("AoS Scalar", |b| {
        let mut aos_data = AoS::new(SIZE);
        b.iter(|| {
            for p in aos_data.particles.iter_mut() {
                p.x += p.vx;
            }
            black_box(aos_data.particles[0].x)
        });
    });

    // AoS SIMD (strided loads/stores - often slower!)
    group.bench_function("AoS SIMD (strided)", |b| {
        let mut aos_data = AoS::new(SIZE);
        b.iter(|| {
            unsafe {
                for chunk in aos_data.particles.chunks_exact_mut(4) {
                    // Strided load (inefficient)
                    let x = _mm_set_ps(chunk[3].x, chunk[2].x, chunk[1].x, chunk[0].x);
                    let vx = _mm_set_ps(chunk[3].vx, chunk[2].vx, chunk[1].vx, chunk[0].vx);
                    let result = _mm_add_ps(x, vx);

                    // Strided store (inefficient)
                    let mut temp = [0f32; 4];
                    _mm_storeu_ps(temp.as_mut_ptr(), result);
                    chunk[0].x = temp[0];
                    chunk[1].x = temp[1];
                    chunk[2].x = temp[2];
                    chunk[3].x = temp[3];
                }
            }
            black_box(aos_data.particles[0].x)
        });
    });

    group.finish();
}

//================================================
// Criterion Configuration
//================================================

#[cfg(target_arch = "x86_64")]
criterion_group!(
    benches,
    aos_vs_soa_parameterized,
    simd_benchmarks,
    aos_simd_comparison
);

#[cfg(not(target_arch = "x86_64"))]
criterion_group!(benches, aos_vs_soa_parameterized);

criterion_main!(benches);
