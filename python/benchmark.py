"""End-to-end Python comparison; list operations run in C, ours in Python."""
import csv
import statistics
import sys
import time

from permarray import PermutationArray

iterations = int(sys.argv[1]) if len(sys.argv) > 1 else 20000
if iterations <= 0:
    raise ValueError("iterations must be positive")
writer = csv.writer(sys.stdout, lineterminator="\n")
writer.writerow(["workload", "n", "container", "median_ns_per_iteration"])
for size in (8, 16, 32):
    for workload in ("edit_pair", "lookup"):
        for container in ("permarray", "list_payload", "list_offsets"):
            samples = []
            for _ in range(5):
                pa = PermutationArray()
                for value in range(size):
                    pa.insert(0 if workload == "lookup" else value, value)
                direct = pa.values()
                slab = list(range(size)) + [None]
                offsets = list(range(size))
                if workload == "lookup":
                    offsets.reverse()
                middle = size // 2
                checksum = 0
                start = time.perf_counter_ns()
                if workload == "edit_pair":
                    for value in range(iterations):
                        if container == "permarray":
                            pa.insert(middle, value)
                            checksum ^= pa.remove_newest()
                        elif container == "list_payload":
                            direct.insert(middle, value)
                            checksum ^= direct.pop(middle)
                        else:
                            slab[size] = value
                            offsets.insert(middle, size)
                            slot = offsets.pop(middle)
                            checksum ^= slab[slot]
                            slab[slot] = None
                else:
                    for value in range(iterations):
                        index = value % size
                        if container == "permarray":
                            checksum ^= pa[index]
                        elif container == "list_payload":
                            checksum ^= direct[index]
                        else:
                            checksum ^= slab[offsets[index]]
                samples.append((time.perf_counter_ns() - start) / iterations)
                assert checksum >= 0
            writer.writerow([workload, size, container, f"{statistics.median(samples):.3f}"])
