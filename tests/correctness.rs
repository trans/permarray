use permarray::{Error, Permutation, PermutationArray, MAX_SLOTS};

fn next_sjt(values: &mut [usize], directions: &mut [isize]) -> bool {
    let mut selected = None;
    for (i, &value) in values.iter().enumerate() {
        let target = i as isize + directions[value];
        if target >= 0
            && (target as usize) < values.len()
            && value > values[target as usize]
            && selected.is_none_or(|(v, _, _)| value > v)
        {
            selected = Some((value, i, target as usize));
        }
    }
    let Some((value, i, target)) = selected else {
        return false;
    };
    values.swap(i, target);
    for direction in directions.iter_mut().skip(value + 1) {
        *direction *= -1;
    }
    true
}

#[test]
fn exhaustive_against_independent_generator() {
    for size in 0..=8 {
        let mut values: Vec<_> = (0..size).collect();
        let mut directions = vec![-1; size];
        let mut rank = 0;
        loop {
            let p = Permutation::new(size, rank).unwrap();
            assert_eq!(p.order(), values);
            assert_eq!(Permutation::from_order(&values).unwrap(), p);
            for (index, &slot) in values.iter().enumerate() {
                assert_eq!(p.offset(index), Some(slot));
            }
            assert_eq!(p.offset(size), None);
            if size > 0 {
                let mut reduced = p;
                assert_eq!(reduced.remove_newest(), Some(size - 1));
                assert_eq!(
                    reduced.order(),
                    values
                        .iter()
                        .copied()
                        .filter(|&x| x != size - 1)
                        .collect::<Vec<_>>()
                );
            }
            if size <= 6 {
                for position in 0..=size {
                    let mut inserted = p;
                    inserted.insert_newest(position).unwrap();
                    let mut expected = values.clone();
                    expected.insert(position, size);
                    assert_eq!(inserted.order(), expected);
                }
            }
            rank += 1;
            if !next_sjt(&mut values, &mut directions) {
                break;
            }
        }
        assert_eq!(rank, (1..=size).fold(1u128, |f, n| f * n as u128));
    }
}

#[test]
fn boundaries_and_atomic_errors() {
    assert_eq!(Permutation::new(0, 1), Err(Error::InvalidRank));
    assert_eq!(Permutation::new(35, 0), Err(Error::Capacity));
    assert_eq!(
        Permutation::from_order(&[0, 0]),
        Err(Error::InvalidPermutation)
    );
    assert_eq!(
        Permutation::from_order(&[0, 2]),
        Err(Error::InvalidPermutation)
    );
    let factorial = (1..=MAX_SLOTS).fold(1u128, |f, n| f * n as u128);
    let last = Permutation::new(MAX_SLOTS, factorial - 1).unwrap();
    assert_eq!(Permutation::from_order(&last.order()).unwrap(), last);
    assert_eq!(
        Permutation::new(MAX_SLOTS, factorial),
        Err(Error::InvalidRank)
    );
    let mut array = PermutationArray::new();
    assert_eq!(array.insert(1, 42), Err((Error::Index, 42)));
    assert_eq!(array.remove(0), None);
    assert_eq!(array.remove_newest(), None);
    for i in 0..MAX_SLOTS {
        array.insert(0, i).unwrap();
    }
    let snapshot: Vec<_> = array.iter().copied().collect();
    assert_eq!(array.insert(0, 99), Err((Error::Capacity, 99)));
    assert_eq!(snapshot, array.iter().copied().collect::<Vec<_>>());
    // Interior deletion does not reclaim an allocated slot.
    array.remove(MAX_SLOTS - 1);
    assert_eq!(array.len(), MAX_SLOTS - 1);
    assert_eq!(array.insert(0, 99), Err((Error::Capacity, 99)));
    array.remove_newest();
    array.insert(0, 99).unwrap();
    array.clear();
    assert!(array.is_empty());
    assert_eq!(array.allocated_slots(), 0);
}

fn random(state: &mut u64) -> usize {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state as usize
}

#[test]
fn randomized_container_matches_vec() {
    let mut seed = 20261102;
    for _ in 0..100 {
        let mut array = PermutationArray::new();
        let mut expected = Vec::new();
        for value in 0..1000 {
            if array.allocated_slots() == MAX_SLOTS {
                // Exercise complete draining, including reclamation of holes.
                while !expected.is_empty() {
                    let i = random(&mut seed) % expected.len();
                    assert_eq!(array.remove(i), Some(expected.remove(i)));
                }
                assert_eq!(array.allocated_slots(), 0);
            }
            if expected.is_empty() || random(&mut seed) % 100 < 58 {
                let i = random(&mut seed) % (expected.len() + 1);
                array.insert(i, value).unwrap();
                expected.insert(i, value);
            } else {
                let i = random(&mut seed) % expected.len();
                assert_eq!(array.remove(i), Some(expected.remove(i)));
            }
            assert_eq!(array.len(), expected.len());
            assert_eq!(array.iter().copied().collect::<Vec<_>>(), expected);
            for (i, value) in expected.iter().enumerate() {
                assert_eq!(array.get(i), Some(value));
            }
        }
    }
}

#[test]
fn python_cross_language_fixtures() {
    for line in include_str!("python-fixtures.tsv").lines().skip(1) {
        let fields: Vec<_> = line.split('\t').collect();
        let size: usize = fields[0].parse().unwrap();
        let rank: u128 = fields[1].parse().unwrap();
        let expected: Vec<usize> = fields[2]
            .split(',')
            .filter(|s| !s.is_empty() && *s != "-")
            .map(|s| s.parse().unwrap())
            .collect();
        assert_eq!(Permutation::new(size, rank).unwrap().order(), expected);
    }
}

#[test]
fn mutation_and_drop_ownership() {
    use std::{cell::Cell, rc::Rc};
    #[derive(Debug)]
    struct Tracked(Rc<Cell<usize>>);
    impl Drop for Tracked {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let drops = Rc::new(Cell::new(0));
    {
        let mut array = PermutationArray::new();
        for i in 0..8 {
            array.insert(i, Tracked(drops.clone())).unwrap();
        }
        drop(array.remove(2));
        drop(array.remove_newest());
        assert_eq!(drops.get(), 2);
        array.clear();
        assert_eq!(drops.get(), 8);
    }
    assert_eq!(drops.get(), 8);
    let mut array = PermutationArray::new();
    array.insert(0, 1).unwrap();
    *array.get_mut(0).unwrap() = 7;
    assert_eq!(array.get(0), Some(&7));
}
