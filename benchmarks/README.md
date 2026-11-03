# Benchmark snapshot

Measured 2026-11-02. Values are medians of five samples in nanoseconds per iteration. Raw results: [Rust](rust.csv), [Python](python.csv); [environment](environment.json).

## Rust: 4 KiB values, middle insertion followed by removal

| Existing values | permarray | Vec of values | Vec of offsets + stable payload slab |
| ---: | ---: | ---: | ---: |
| 8 | 162.0 | 295.3 | 206.2 |
| 16 | 159.1 | 432.7 | 218.4 |
| 32 | 158.0 | 1371.0 | 214.2 |

For 32 existing values in this workload, permarray is about 8.7× faster than moving the values directly, and about 1.36× faster than the offset-vector alternative. These are observations about this harness, not general speedup guarantees.

## Rust: indexed lookup of 8-byte values

| Existing values | permarray | Vec of values | Vec of offsets + slab |
| ---: | ---: | ---: | ---: |
| 8 | 11.6 | 1.3 | 1.3 |
| 16 | 22.4 | 1.3 | 1.3 |
| 32 | 63.8 | 1.3 | 1.3 |

The lookup workload cycles through logical positions in a reversed permutation. It demonstrates the cost of decoding; it does not establish an unavoidable lower bound.

## Python: 32 existing values

| Workload | permarray | Python list | List of offsets + slab |
| --- | ---: | ---: | ---: |
| edit_pair | 304.4 | 108.0 | 168.3 |
| lookup | 1822.5 | 86.0 | 93.7 |

The pure-Python implementation loses to built-in lists here. Its methods execute Python bytecode, whereas list operations use optimized native code. Treat it as a reference implementation, not evidence that every implementation of the representation must lose.

## Methodology and limits

- Rust was built in release mode; setup is outside the timed loop. Each sample uses 500,000 iterations and `std::hint::black_box` to expose operations/results to the optimizer. Python uses 20,000 iterations and a consumed checksum.
- An edit iteration inserts at the fixed middle position and removes exactly that new value. Permarray uses its allocation-order removal shortcut; the vector baselines remove at the already-known insertion position. No arbitrary-deletion lookup is charged to either side. This deliberately favorable dense workload does not represent mixed long-lived edits.
- The offset-vector baseline also writes and takes the new payload in a stable slab. It is not merely shuffling offsets while omitting payload work. Storage is reserved before timing; no per-edit heap allocation is needed.
- Rust tests 8-byte values and inline 4 KiB arrays. Large payload movement is the intended distinction. Python lists store references; its benchmark has no equivalent inline-large-value case.
- Capacity is held fixed, no holes accumulate, all data is hot, and there is no cache-cold/random application workload. Rust iteration and hole-handling costs are not benchmarked here.
- The process was not CPU-pinned, clocks were not controlled, and the shared machine may have other workloads. No confidence intervals are claimed. Compiler specialization and memory-copy implementations can change results.
- Rust/Python figures are separate end-to-end experiments, not a language-neutral algorithm comparison.

Reproduce from the repository root:

```sh
cargo run --release --offline --example bench -- 500000 > benchmarks/rust.csv
PYTHONPATH=python python python/benchmark.py 20000 > benchmarks/python.csv
```

These commands replace the raw snapshots. Update this narrative if results change.
