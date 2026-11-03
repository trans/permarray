"""Independent enumeration and state-machine checks for article claims."""
import json
import math
from pathlib import Path
from random import Random

from permarray import (PermutationArray, insert_largest, offset_sjt,
                               rank_sjt, unrank_sjt)


def enumerate_sjt(size):
    """Independent mobile-element generator, not the rank recurrence."""
    values = list(range(size))
    directions = [-1] * size
    yield values.copy()
    while True:
        mobile = []
        for i, value in enumerate(values):
            target = i + directions[value]
            if 0 <= target < size and value > values[target]:
                mobile.append((value, i, target))
        if not mobile:
            return
        value, i, target = max(mobile)
        values[i], values[target] = values[target], values[i]
        for larger in range(value + 1, size):
            directions[larger] *= -1
        yield values.copy()


counts = {"permutations": 0, "lookups": 0, "insertions": 0,
          "largest_deletions": 0, "container_operations": 0}
for size in range(9):
    enumerated = 0
    for rank, permutation in enumerate(enumerate_sjt(size)):
        assert rank_sjt(permutation) == rank
        assert unrank_sjt(rank, size) == permutation
        counts["permutations"] += 1
        enumerated += 1
        for index, slot in enumerate(permutation):
            assert offset_sjt(rank, size, index) == slot
            counts["lookups"] += 1
        if size:
            reduced = [x for x in permutation if x != size-1]
            assert rank // size == rank_sjt(reduced)
            counts["largest_deletions"] += 1
        if size <= 6:
            for index in range(size + 1):
                expected = permutation.copy()
                expected.insert(index, size)
                assert rank_sjt(expected) == insert_largest(rank, size, index)
                counts["insertions"] += 1

    assert enumerated == math.factorial(size)

rng = Random(20261102)
for trial in range(100):
    reference, array = [], PermutationArray()
    for operation in range(100):
        if not reference or rng.random() < .58:
            index = rng.randrange(len(reference) + 1)
            value = (trial, operation)
            reference.insert(index, value)
            array.insert(index, value)
        else:
            index = rng.randrange(len(reference))
            assert array.delete(index) == reference.pop(index)
        assert array.values() == reference
        assert len(array) == len(reference)
        if reference:
            index = rng.randrange(len(reference))
            assert array[index] == reference[index]
        counts["container_operations"] += 1

# A neighboring SJT rank is an adjacent swap, but the converse is false.
counterexample = {"before": [0, 1, 2], "after": [1, 0, 2]}
counterexample["ranks"] = [rank_sjt(counterexample[k]) for k in ("before", "after")]
assert counterexample["ranks"] == [0, 5]

growth = []
for size in (16, 50, 64, 256, 1024):
    slot, steps = offset_sjt(0, size, 0, with_steps=True)
    assert slot == 0 and steps == size
    growth.append({"size": size, "max_rank_bits": (math.factorial(size)-1).bit_length(),
                   "identity_first_lookup_divmods": steps})

report = {"checks": counts, "adjacent_swap_counterexample": counterexample,
          "growth": growth,
          "limitations": "Reference algorithms and exact operation counts, not a wall-clock benchmark or lower-bound proof."}
(Path(__file__).resolve().parents[2] / "benchmarks" / "python-correctness.json").write_text(json.dumps(report, indent=2)+"\n")
print(json.dumps(report, indent=2))
