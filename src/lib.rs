//! Experimental stable-slot collections with SJT-encoded logical order.
//!
//! `u128` holds every permutation through 34 slots. Allocated holes count
//! toward that limit; only unused trailing slots are automatically reclaimed.
//! This is not a claim of constant-time random access or general deletion.

use std::fmt;

/// Largest n for which n! fits in a u128.
pub const MAX_SLOTS: usize = 34;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Capacity,
    Index,
    InvalidRank,
    InvalidPermutation,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Capacity => "permutation exceeds 34 allocated slots",
            Self::Index => "logical index out of bounds",
            Self::InvalidRank => "rank must be less than size factorial",
            Self::InvalidPermutation => "expected a permutation of 0..size",
        })
    }
}
impl std::error::Error for Error {}

/// Dense permutation of physical slot IDs 0..len, encoded in SJT order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Permutation {
    rank: u128,
    len: usize,
}

impl Permutation {
    pub fn new(len: usize, rank: u128) -> Result<Self, Error> {
        if len > MAX_SLOTS {
            return Err(Error::Capacity);
        }
        let factorial = (1..=len).fold(1u128, |f, n| f * n as u128);
        if rank >= factorial {
            return Err(Error::InvalidRank);
        }
        Ok(Self { rank, len })
    }

    /// Reference ranking with O(n^2) small-vector work.
    pub fn from_order(order: &[usize]) -> Result<Self, Error> {
        if order.len() > MAX_SLOTS {
            return Err(Error::Capacity);
        }
        let mut seen = [false; MAX_SLOTS];
        for &slot in order {
            if slot >= order.len() || seen[slot] {
                return Err(Error::InvalidPermutation);
            }
            seen[slot] = true;
        }
        let mut reduced = order.to_vec();
        let mut positions = Vec::with_capacity(order.len());
        while !reduced.is_empty() {
            let slot = reduced.len() - 1;
            let position = reduced.iter().position(|&x| x == slot).unwrap();
            positions.push(position);
            reduced.remove(position);
        }
        let mut result = Self::default();
        for position in positions.into_iter().rev() {
            result.insert_newest(position)?;
        }
        Ok(result)
    }

    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn rank(&self) -> u128 {
        self.rank
    }

    /// Insert a newly allocated largest slot. Constant fixed-width arithmetic.
    pub fn insert_newest(&mut self, position: usize) -> Result<usize, Error> {
        if position > self.len {
            return Err(Error::Index);
        }
        if self.len == MAX_SLOTS {
            return Err(Error::Capacity);
        }
        let slot = self.len;
        let digit = if self.rank & 1 == 0 {
            slot - position
        } else {
            position
        };
        self.rank = (slot as u128 + 1) * self.rank + digit as u128;
        self.len += 1;
        Ok(slot)
    }

    /// Remove the highest-numbered physical slot, wherever it appears.
    pub fn remove_newest(&mut self) -> Option<usize> {
        if self.is_empty() {
            return None;
        }
        self.rank /= self.len as u128;
        self.len -= 1;
        Some(self.len)
    }

    /// Resolve a logical position in O(n) fixed-width arithmetic operations.
    pub fn offset(&self, mut index: usize) -> Option<usize> {
        if index >= self.len {
            return None;
        }
        let mut rank = self.rank;
        for width in (1..=self.len).rev() {
            let quotient = rank / width as u128;
            let digit = (rank % width as u128) as usize;
            let position = if quotient & 1 == 0 {
                width - 1 - digit
            } else {
                digit
            };
            if index == position {
                return Some(width - 1);
            }
            if index > position {
                index -= 1;
            }
            rank = quotient;
        }
        unreachable!()
    }

    /// Decode with O(n) divisions and O(n^2) worst-case vector shifts.
    pub fn order(&self) -> Vec<usize> {
        let mut rank = self.rank;
        let mut positions = [0; MAX_SLOTS];
        for width in (1..=self.len).rev() {
            let quotient = rank / width as u128;
            let digit = (rank % width as u128) as usize;
            positions[width - 1] = if quotient & 1 == 0 {
                width - 1 - digit
            } else {
                digit
            };
            rank = quotient;
        }
        let mut order = Vec::with_capacity(self.len);
        for (slot, &position) in positions[..self.len].iter().enumerate() {
            order.insert(position, slot);
        }
        order
    }
}

/// Values retain physical slot IDs until removed. No heap reallocation on insert
/// (all 34 slots are reserved at construction), but moving the container itself
/// is permitted. This is not a pinning API. Failed insertions return the value.
#[derive(Debug)]
pub struct PermutationArray<T> {
    order: Permutation,
    slots: Vec<Option<T>>,
    len: usize,
}

impl<T> Default for PermutationArray<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> PermutationArray<T> {
    pub fn new() -> Self {
        Self {
            order: Permutation::default(),
            slots: Vec::with_capacity(MAX_SLOTS),
            len: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn allocated_slots(&self) -> usize {
        self.slots.len()
    }
    pub fn rank(&self) -> u128 {
        self.order.rank()
    }

    fn slot_at(&self, index: usize) -> Option<usize> {
        if index >= self.len {
            return None;
        }
        if self.len == self.slots.len() {
            return self.order.offset(index);
        }
        self.order
            .order()
            .into_iter()
            .filter(|&s| self.slots[s].is_some())
            .nth(index)
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.slots[self.slot_at(index)?].as_ref()
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        let slot = self.slot_at(index)?;
        self.slots[slot].as_mut()
    }

    /// General insertion; dense fast path avoids full decoding. Interior holes
    /// are retained, so capacity can be exhausted before live length reaches 34.
    pub fn insert(&mut self, index: usize, value: T) -> Result<(), (Error, T)> {
        if index > self.len {
            return Err((Error::Index, value));
        }
        if self.slots.len() == MAX_SLOTS {
            return Err((Error::Capacity, value));
        }
        let position = if self.len == self.slots.len() {
            index
        } else if index == self.len {
            self.slots.len()
        } else {
            self.order
                .order()
                .into_iter()
                .enumerate()
                .filter(|&(_, slot)| self.slots[slot].is_some())
                .nth(index)
                .unwrap()
                .0
        };
        self.order.insert_newest(position).unwrap();
        self.slots.push(Some(value));
        self.len += 1;
        Ok(())
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        let slot = self.slot_at(index)?;
        let result = self.slots[slot].take();
        self.len -= 1;
        self.trim_unused_tail();
        result
    }

    /// Fast removal by allocation order, not logical position. Also reclaims
    /// trailing holes left by earlier removals; worst-case O(allocated slots).
    pub fn remove_newest(&mut self) -> Option<T> {
        let result = self.slots.pop()?.unwrap();
        self.order.remove_newest();
        self.len -= 1;
        self.trim_unused_tail();
        Some(result)
    }

    fn trim_unused_tail(&mut self) {
        while self.slots.last().is_some_and(Option::is_none) {
            self.slots.pop();
            self.order.remove_newest();
        }
    }

    /// Full decoding allocates an offset vector. Returned iterator skips holes.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.order
            .order()
            .into_iter()
            .filter_map(|s| self.slots[s].as_ref())
    }

    pub fn clear(&mut self) {
        self.slots.clear();
        self.order = Permutation::default();
        self.len = 0;
    }
}
