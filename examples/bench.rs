//! Lightweight benchmark harness, not Criterion: no external dependencies.
//! Five-sample medians, release mode, setup excluded, allocations reserved.
use permarray::PermutationArray;
use std::{hint::black_box, time::Instant};

fn median(mut samples: Vec<f64>) -> f64 {
    samples.sort_by(f64::total_cmp);
    samples[samples.len() / 2]
}

fn edits<const WORDS: usize>(n: usize, iterations: usize) {
    let p = n / 2;
    for container in ["permarray", "vec_payload", "vec_offsets"] {
        let mut samples = Vec::new();
        for _ in 0..5 {
            let mut pa = PermutationArray::new();
            let mut direct: Vec<[u64; WORDS]> = Vec::with_capacity(34);
            let mut offsets: Vec<usize> = Vec::with_capacity(34);
            // The offset-vector baseline has the same stable payload slab.
            let payload_slab = vec![[7u64; WORDS]; n + 1];
            let mut indirect_slab: Vec<Option<[u64; WORDS]>> =
                payload_slab.iter().take(n).copied().map(Some).collect();
            indirect_slab.push(None);
            for (i, payload) in payload_slab.iter().take(n).enumerate() {
                pa.insert(i, *payload).unwrap();
                direct.push(*payload);
                offsets.push(i);
            }
            let start = Instant::now();
            for i in 0..iterations {
                match container {
                    "permarray" => {
                        pa.insert(black_box(p), black_box([i as u64; WORDS]))
                            .unwrap();
                        black_box(&pa);
                        black_box(pa.remove_newest().unwrap());
                    }
                    "vec_payload" => {
                        direct.insert(black_box(p), black_box([i as u64; WORDS]));
                        black_box(&direct);
                        black_box(direct.remove(black_box(p)));
                    }
                    _ => {
                        indirect_slab[n] = Some(black_box([i as u64; WORDS]));
                        offsets.insert(black_box(p), black_box(n));
                        black_box((&offsets, &indirect_slab));
                        let slot = offsets.remove(black_box(p));
                        black_box(indirect_slab[slot].take().unwrap());
                    }
                }
            }
            let elapsed = start.elapsed().as_secs_f64() * 1e9 / iterations as f64;
            black_box((pa, direct, offsets, payload_slab, indirect_slab));
            samples.push(elapsed);
        }
        println!(
            "edit_pair,{n},{},{container},{:.3}",
            WORDS * 8,
            median(samples)
        );
    }
}

fn lookups(n: usize, iterations: usize) {
    for container in ["permarray", "vec_payload", "vec_offsets"] {
        let mut samples = Vec::new();
        for _ in 0..5 {
            let mut pa = PermutationArray::new();
            let mut direct = Vec::with_capacity(n);
            for i in 0..n {
                pa.insert(0, i as u64).unwrap();
                direct.insert(0, i as u64);
            }
            let slab: Vec<_> = (0..n as u64).collect();
            let offsets: Vec<_> = (0..n).rev().collect();
            let start = Instant::now();
            for i in 0..iterations {
                let index = black_box(i % n);
                let value = match container {
                    "permarray" => *pa.get(index).unwrap(),
                    "vec_payload" => direct[index],
                    _ => slab[offsets[index]],
                };
                black_box(value);
            }
            samples.push(start.elapsed().as_secs_f64() * 1e9 / iterations as f64);
        }
        println!("lookup,{n},8,{container},{:.3}", median(samples));
    }
}

fn main() {
    let iterations = std::env::args()
        .nth(1)
        .map(|s| s.parse().expect("iterations must be a positive integer"))
        .unwrap_or(50000);
    assert!(iterations > 0);
    println!("workload,n,payload_bytes,container,median_ns_per_iteration");
    for n in [8, 16, 32] {
        edits::<1>(n, iterations);
        edits::<512>(n, iterations);
        lookups(n, iterations);
    }
}
