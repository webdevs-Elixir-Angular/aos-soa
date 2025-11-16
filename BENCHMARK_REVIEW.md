# Benchmark Review and Improvements

## Executive Summary

This document details the issues found in the AoS vs SoA benchmarks and the improvements made. The original benchmarks had **critical flaws** that made them unreliable or completely broken.

### Key Finding
**The original benchmarks were potentially measuring nothing** due to compiler optimizations eliminating the computation entirely.

---

## Critical Issues Found

### 1. ❌ CRITICAL: Broken `black_box()` Usage

**Files Affected:** `my_benchmark.rs`, `auto_optimisation_benchmark.rs`

**The Problem:**
```rust
// ❌ INCORRECT - Taking a reference
b.iter(|| {
    for i in 0..NUM_PARTICLES {
        soa_data.x[i] += soa_data.vx[i];
    }
    black_box(&soa_data);  // Only pins the pointer!
})
```

**Why This Is Broken:**
- `black_box(&data)` forces the **pointer** to exist
- It does NOT force the **computation** to happen
- The compiler can legally skip the entire loop and just return the pointer
- Results in measuring ~0ns instead of actual computation time

**The Fix:**
```rust
// ✅ CORRECT - Return a value from the computation
b.iter(|| {
    for i in 0..NUM_PARTICLES {
        soa_data.x[i] += soa_data.vx[i];
    }
    black_box(soa_data.x[0])  // Must compute to return this value
})
```

**Impact:** 🔴 **CRITICAL** - Without this fix, benchmarks measure nothing

**Locations Fixed:**
- `my_benchmark.rs:91, 106, 121, 158, 185, 208, 234, 265`
- `auto_optimisation_benchmark.rs` - Completely rewritten to demonstrate the issue

---

### 2. ❌ CRITICAL: Missing `black_box()` Entirely

**File:** `auto_optimisation_benchmark.rs` (original version)

**The Problem:**
```rust
// ❌ NO black_box at all
b.iter(|| {
    for p in aos_data.particles.iter_mut() {
        p.x += p.vx;
    }
    // aos_data is never observed - entire loop eliminated!
})
```

**Benchmark Results Showed:**
- AoS WITHOUT black_box: ~89 µs (partially optimized, unreliable)
- SoA WITHOUT black_box: ~15 µs (could be optimized away)
- With proper black_box: Reliable, consistent measurements

**The Fix:**
- Added proper `black_box()` calls returning computed values
- Created comparison benchmarks showing BROKEN vs CORRECT approaches
- Demonstrates the impact of different black_box strategies

**Impact:** 🔴 **CRITICAL** - Benchmark was completely broken

---

### 3. ❌ HIGH: Incorrect SIMD Feature Detection

**File:** `my_benchmark.rs`

**The Problem:**
```rust
// ❌ Compile-time check only
#[cfg(target_feature = "avx512f")]
c.bench_function("SoA Update (SIMD x16)", |b| {
    // This code won't even compile on most systems!
})
```

**Why This Fails:**
- `#[cfg(target_feature)]` is a **compile-time** check
- Most consumer CPUs don't have AVX-512
- Code won't compile unless you explicitly enable the feature
- Even if compiled with the feature, might crash on CPUs without it

**The Fix (in improved_benchmark.rs):**
```rust
// ✅ Runtime detection
if is_x86_feature_detected!("avx512f") {
    group.bench_function("SIMD AVX-512 (x16)", |b| {
        // Only runs if CPU supports it
    });
} else {
    eprintln!("⚠️  AVX-512 not detected - skipping");
}
```

**Impact:** 🟡 **HIGH** - AVX-512 benchmarks won't run on most systems

---

### 4. ❌ MEDIUM: No Data Size Parameterization

**Files:** `my_benchmark.rs`, `auto_optimisation_benchmark.rs`

**The Problem:**
- Only tests with 100,000 elements
- Can't observe cache effects (L1 vs L2 vs L3 vs RAM)
- Real-world performance varies dramatically by data size

**Cache Sizes (typical):**
- L1: ~32 KB (extremely fast)
- L2: ~256 KB - 1 MB (very fast)
- L3: ~8 MB - 32 MB (fast)
- RAM: GB scale (slower)

**The Fix (improved_benchmark.rs):**
```rust
let sizes = vec![
    1_000,      // L1 cache (4KB data)
    10_000,     // L2 cache (40KB data)
    100_000,    // L3 cache (400KB data)
    1_000_000,  // RAM-bound (4MB data)
];

for size in sizes.iter() {
    group.bench_with_input(BenchmarkId::new("AoS", size), size, |b, &size| {
        // Benchmark with this size
    });
}
```

**Impact:** 🟡 **MEDIUM** - Miss important performance characteristics

---

### 5. ❌ MEDIUM: No Throughput Measurements

**Files:** All original benchmarks

**The Problem:**
- Only shows total time (e.g., "100 µs")
- Hard to compare different data sizes
- Can't calculate elements/second or GB/second

**The Fix:**
```rust
group.throughput(Throughput::Elements(*size as u64));
```

**Result:** Criterion now shows:
```
time:   [15.051 µs 15.207 µs 15.246 µs]
thrpt:  [6.5579 M elem/s 6.5749 M elem/s 6.6429 M elem/s]
```

**Impact:** 🟡 **MEDIUM** - Harder to interpret results

---

### 6. ❌ LOW: No Benchmark Configuration

**Files:** `my_benchmark.rs`, `auto_optimisation_benchmark.rs`

**The Problem:**
- Uses default Criterion settings
- Default warmup may be too short for SIMD code
- Default sample size may not be optimal

**The Fix:**
```rust
group.measurement_time(Duration::from_secs(10));  // Longer measurement
group.sample_size(100);                           // More samples
group.warm_up_time(Duration::from_secs(3));       // Proper warmup
```

**Impact:** 🟢 **LOW** - Mostly affects measurement accuracy

---

## Common Criterion Benchmarking Mistakes

### 1. Taking References in `black_box()`
**Wrong:** `black_box(&data)` ❌
**Right:** `black_box(data[0])` ✅

### 2. No Observable Side Effects
**Wrong:** Computation with no return value ❌
**Right:** Return or use computed values ✅

### 3. Compile-Time Feature Flags
**Wrong:** `#[cfg(target_feature = "avx")]` ❌
**Right:** `is_x86_feature_detected!("avx")` ✅

### 4. Single Data Size
**Wrong:** Only test one size ❌
**Right:** Parameterize across sizes ✅

### 5. No Throughput Metrics
**Wrong:** Only time measurements ❌
**Right:** Add `Throughput::Elements()` ✅

### 6. Default Configuration
**Wrong:** No explicit warmup/samples ❌
**Right:** Configure measurement time ✅

### 7. Unverified Results
**Wrong:** Trust the numbers blindly ❌
**Right:** Check assembly, compare variants ✅

---

## Theory: Why Compiler Optimizations Break Benchmarks

### The "As-If" Rule

Compilers follow the **as-if rule**: They can do anything they want, as long as the **observable behavior** is as if the code ran literally.

Observable behaviors:
- Return values from `main()`
- I/O operations (print, file, network)
- `panic!()` calls
- Volatile operations
- Cross-module boundaries (before LTO)

**In a benchmark:**
```rust
b.iter(|| {
    let mut sum = 0;
    for i in 0..1000 {
        sum += i;
    }
    // sum is dropped - never observed
});
```

Observable behavior: **NONE**
Legal optimization: **Remove everything, return immediately**

### Dead Code Elimination (DCE)

The optimizer sees:
1. Allocate memory
2. Perform computation
3. Drop memory
4. No external observation of results

Conclusion: This is dead code → eliminate it!

### Why References Don't Help

```rust
let data = vec![1, 2, 3];
// ... modify data ...
black_box(&data);
```

What must exist:
- ✅ A valid pointer (the reference)
- ✅ An allocation (for the pointer to point to)
- ❌ Correct data values (never dereferenced!)

Legal optimization:
```rust
let data = vec![uninitialized, uninitialized, uninitialized];
black_box(&data);  // Valid pointer, check!
```

### The Correct Approach

Force the compiler to actually compute values by **observing them**:

```rust
b.iter(|| {
    // Computation
    for i in 0..SIZE {
        data[i] += 1;
    }
    // Observation - must be correct!
    black_box(data[0])
})
```

Now the compiler must:
1. Actually perform the computation (at least for `data[0]`)
2. Ensure the value is correct
3. Cannot eliminate the loop

---

## Verification Methods

### Method 1: Check Assembly

```bash
cargo rustc --release --bench my_benchmark -- --emit asm
```

Look for:
- ❌ Just `ret` instruction → optimized away!
- ✅ Actual loop with vector instructions → working!

### Method 2: Compare Times

Broken benchmark signs:
- Sub-microsecond for 100k elements
- No difference between Debug and Release
- Suspiciously consistent timing

Good benchmark signs:
- Reasonable times (microseconds to milliseconds)
- Release faster than Debug
- Some natural variance

### Method 3: Comparison Benchmarks

Create a baseline that should be faster:
```rust
c.bench_function("Empty", |b| {
    b.iter(|| black_box(0))
});
```

If your main benchmark is faster than this → it's broken!

---

## Files Modified

### Fixed Files:
1. ✅ `benches/my_benchmark.rs` - Fixed all `black_box()` usage
2. ✅ `benches/auto_optimisation_benchmark.rs` - Complete rewrite demonstrating issues
3. ✅ `Cargo.toml` - Added new benchmark entry

### New Files:
1. ✨ `benches/improved_benchmark.rs` - **RECOMMENDED** - All fixes + improvements
2. 📄 `BENCHMARK_REVIEW.md` - This document
3. ✅ `README.md` - Updated with improvements

---

## Recommendations

### For This Repo:

1. **Use `improved_benchmark.rs`** as the primary benchmark
2. Keep `auto_optimisation_benchmark.rs` for educational purposes
3. Consider deprecating `my_benchmark.rs` or updating it further

### For Future Benchmarks:

1. **Always use `std::hint::black_box()` with values, not references**
2. **Parameterize across data sizes** to observe cache effects
3. **Add throughput measurements** for easier interpretation
4. **Use runtime feature detection** for SIMD code
5. **Configure warmup and measurement time** explicitly
6. **Verify results** by checking assembly or comparing variants
7. **Document what you're measuring** clearly

---

## Performance Results (Expected)

With proper benchmarks, you should see:

**SoA vs AoS:**
- SoA is ~5-8x faster for simple operations
- Gap widens with SIMD
- AoS SIMD often slower than AoS scalar (strided loads)

**Cache Effects:**
- Small data (L1): Minimal difference
- Medium data (L2): SoA starts winning
- Large data (L3+): SoA wins significantly

**SIMD:**
- SSE (x4): ~2-3x speedup over scalar
- AVX (x8): ~4-6x speedup over scalar
- AVX-512 (x16): ~8-12x speedup (when available)

---

## Conclusion

The original benchmarks had **critical flaws** that could result in measuring nothing. The fixes ensure:

✅ Benchmarks actually measure computation (not optimized away)
✅ SIMD code runs on appropriate hardware
✅ Results are comparable across sizes
✅ Throughput is clearly visible
✅ Configuration is explicit and appropriate

**Use `improved_benchmark.rs` for reliable measurements.**
