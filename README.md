# permarray

An experimental collection that stores values in stable slots and encodes
their logical order as a Steinhaus–Johnson–Trotter permutation rank.

The starting question: can a small integer replace an array of offsets,
letting us rearrange a collection without shifting its existing values?
New-slot insertion has a short arithmetic formula. Random access and holes
from deletion are the interesting costs that remain.

This repository contains a dependency-free **Python reference** and an
experimental **Rust library**. Neither is published to a package registry.
The package name is provisional; registry availability has not been checked.

## Rust

Requires Rust 1.82 or newer. No external crates, unsafe code, or build script.

```rust
use permarray::PermutationArray;

let mut a = PermutationArray::new();
a.insert(0, "A").unwrap();
a.insert(1, "B").unwrap();
a.insert(1, "C").unwrap();
assert_eq!(a.iter().copied().collect::<Vec<_>>(), ["A", "C", "B"]);
assert_eq!(a.get(1), Some(&"C"));
assert_eq!(a.remove_newest(), Some("C"));
```

The Rust version uses a `u128`: **at most 34 allocated slots**, since 34!
fits and 35! does not. It reserves the storage capacity at construction.
New insertion leaves existing slot IDs and their backing allocation intact;
this is not a `Pin` API or a promise of permanent addresses after removal.

`remove(index)` deletes by logical position. `remove_newest()` deletes the
most recently allocated surviving slot, wherever it occurs in the logical
sequence. These are different operations. Both reclaim trailing unused
slots. Interior holes remain, so insertion may hit the allocated-slot limit
even when fewer than 34 values are live. `clear()` resets the collection.
Failed insertion returns `(Error, original_value)` without modifying it.

`Permutation` exposes the lower-level rank representation independently of
payload storage: validated construction, ranking, decoding, direct indexed
lookup, new-largest-slot insertion, and largest-slot removal.

## Python

Requires Python 3.10 or newer, with no runtime dependencies. Use directly
from the checkout with `PYTHONPATH=python`, or install the local package
from `python/` in your own environment (building uses setuptools).

```python
from permarray import PermutationArray

a = PermutationArray()
a.insert(0, "A")
a.insert(1, "B")
a.insert(1, "C")
assert a.values() == ["A", "C", "B"]
assert a.delete(1) == "C"
```

The reference uses arbitrary-precision integers and has no fixed slot limit.
It rejects negative indices. Its public rank helpers assume valid ranks
except where expressly validated; it is research code, not a hardened input
parser. `None` is a valid payload: a separate live-slot list tracks deletion.
Python lists may reallocate, so stable slot IDs do not promise stable memory
addresses. Like Rust, it reclaims only unused trailing slots.

## Performance model

Let `m` be the allocated slot count, including interior holes.

| Operation | Current implementation |
| --- | --- |
| Insert into a dense collection | Constant number of rank arithmetic operations, plus payload storage |
| Resolve an index in a dense collection | Up to `m` divisions |
| Decode the complete order | `m` divisions plus O(m²) worst-case list/vector shifts |
| Insert or resolve an index with holes | May decode/filter the full order |
| Remove by logical index | Resolve index, mark slot unused, trim trailing holes |
| Remove newest allocated value | One rank division plus any trailing-hole reclamation |

For Rust, arithmetic is fixed-width within the 34-slot bound. Python's
integer operations get more expensive as the rank grows. These are costs of
the implemented algorithms, not lower-bound proofs about all possible
representations. The prototype does not establish O(1) arbitrary deletion or
O(log m) random access.

See [benchmark methodology and results](benchmarks/README.md). The comparison
includes a plain value vector/list and an offset vector/list into stable
payload storage, rather than comparing only against shifting large objects.

## Run the checks

From this directory:

```sh
cargo test --offline
cargo clippy --offline --all-targets -- -D warnings
cargo run --offline --example basic
PYTHONPATH=python python -m unittest discover -s python/tests -v
PYTHONPATH=python python python/tests/verify_exhaustive.py
```

The independent SJT enumerators cover every permutation through eight slots:
46,234 permutations and 362,879 indexed lookups. They also check new-slot
insertion and largest-slot removal. Rust checks 68 Python-generated fixtures
through 34 slots, including maximum ranks, plus randomized container behavior,
error atomicity, and ownership/drop behavior. Python checks 10,000 mixed
container operations; Rust performs 100,000 generated operations plus drain
cycles against `Vec`.

## Next experiments

- Benchmark actual small-stack workloads rather than only synthetic edits.
- Compare a `u64`/20-slot implementation with the current `u128` version.
- Explore table-assisted lookup for very small bounds.
- Explore slot reuse or explicit compaction, with clear slot-ID semantics.
- Decide whether a demonstrated use case merits a reusable API and releases.

No general-purpose performance advantage is claimed. The pure-Python code
is mainly an executable specification; Rust is where fixed-width experiments
can tell us more.

## Background

- [Johnson–Trotter, NIST](https://xlinux.nist.gov/dads/HTML/johnsonTrotter.html)
- [Myrvold and Ruskey: Ranking and Unranking Permutations in Linear Time](https://webhome.cs.uvic.ca/~ruskey/Publications/RankPerm/RankPerm.html)
- [SymPy permutation ranking APIs](https://docs.sympy.org/latest/modules/combinatorics/permutations.html)

Permutation ranking is established work. This project's focus is the use of
that representation for an editable stable-slot collection. No novelty claim
is made for the SJT recurrence itself.
