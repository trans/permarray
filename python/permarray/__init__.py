"""Reference prototype, not a production container. Standard library only.

SJT order starts [0, 1, ..., n-1] at rank zero. A rank counts permutations;
slot IDs denote physical storage, independent of logical positions.
"""


def insert_largest(rank, size, position):
    """Insert physical slot `size` at a zero-based permutation position.

    Constant number of arbitrary-precision arithmetic operations, not O(1)
    bit complexity. The caller must supply a physical permutation position.
    """
    if not 0 <= position <= size:
        raise IndexError(position)
    width = size + 1
    digit = size - position if rank % 2 == 0 else position
    return width * rank + digit


def rank_sjt(permutation):
    """Simple O(n^2) list-work reference ranking algorithm."""
    sequence = list(permutation)
    if sorted(sequence) != list(range(len(sequence))):
        raise ValueError("expected a permutation of range(n)")
    positions = []
    while sequence:
        largest = len(sequence) - 1
        pos = sequence.index(largest)
        positions.append((len(sequence), pos))
        sequence.pop(pos)
    rank = 0
    for width, pos in reversed(positions):
        rank = insert_largest(rank, width - 1, pos)
    return rank


def unrank_sjt(rank, size):
    """O(n) big-integer divisions plus O(n^2) worst-case list shifts."""
    if size < 0 or rank < 0:
        raise ValueError("negative size or rank")
    positions = []
    for width in range(size, 0, -1):
        quotient, digit = divmod(rank, width)
        pos = width - 1 - digit if quotient % 2 == 0 else digit
        positions.append((width - 1, pos))
        rank = quotient
    if rank:
        raise ValueError("rank outside size! permutations")
    sequence = []
    for value, pos in reversed(positions):
        sequence.insert(pos, value)
    return sequence


def offset_sjt(rank, size, index, *, with_steps=False):
    """Resolve one logical index without constructing the full permutation.

    At most n big-integer divmod operations. Valid rank assumed; size and
    index checked. Removing each largest symbol shrinks the queried index.
    """
    if not 0 <= index < size:
        raise IndexError(index)
    steps = 0
    for width in range(size, 0, -1):
        quotient, digit = divmod(rank, width)
        position = width - 1 - digit if quotient % 2 == 0 else digit
        steps += 1
        if index == position:
            return (width - 1, steps) if with_steps else width - 1
        if index > position:
            index -= 1
        rank = quotient
    raise AssertionError("unreachable")


class PermutationArray:
    """Stable payload slots; append-only allocation and tombstone deletion.

    Arbitrary insertion works but finding its permutation position after
    deletion requires decoding/filtering. Interior holes remain allocated until unused trailing slots can be reclaimed.
    There is no interior compaction or reuse.
    """

    def __init__(self):
        self.slots = []
        self.live = []
        self.rank = 0
        self.length = 0

    def __len__(self):
        return self.length

    def _slot(self, index):
        if not 0 <= index < self.length:
            raise IndexError(index)
        if self.length == len(self.slots):
            return offset_sjt(self.rank, len(self.slots), index)
        for slot in unrank_sjt(self.rank, len(self.slots)):
            if self.live[slot]:
                if index == 0:
                    return slot
                index -= 1
        raise AssertionError("unreachable")

    def __getitem__(self, index):
        return self.slots[self._slot(index)]

    def insert(self, index, value):
        if not 0 <= index <= self.length:
            raise IndexError(index)
        size = len(self.slots)
        position = index
        if self.length != size:
            position = size
            if index < self.length:
                for pos, slot in enumerate(unrank_sjt(self.rank, size)):
                    if self.live[slot]:
                        if index == 0:
                            position = pos
                            break
                        index -= 1
        self.rank = insert_largest(self.rank, size, position)
        self.slots.append(value)
        self.live.append(True)
        self.length += 1

    def delete(self, index):
        slot = self._slot(index)
        value = self.slots[slot]
        self.slots[slot] = None
        self.live[slot] = False
        self.length -= 1
        self._trim_unused_tail()
        return value

    def _trim_unused_tail(self):
        while self.live and not self.live[-1]:
            self.rank //= len(self.slots)
            self.live.pop()
            self.slots.pop()

    def remove_newest(self):
        """Remove by allocation order, not logical position; reclaim tail holes."""
        if not self.slots:
            raise IndexError("empty permutation array")
        self.rank //= len(self.slots)
        value = self.slots.pop()
        self.live.pop()
        self.length -= 1
        self._trim_unused_tail()
        return value

    def values(self):
        return [self.slots[slot] for slot in unrank_sjt(self.rank, len(self.slots))
                if self.live[slot]]
