use permarray::PermutationArray;

fn main() {
    let mut values = PermutationArray::new();
    values.insert(0, "A").unwrap();
    values.insert(1, "B").unwrap();
    values.insert(1, "C").unwrap();
    values.insert(1, "D").unwrap();
    println!("{:?}", values.iter().collect::<Vec<_>>()); // A, D, C, B
    assert_eq!(values.remove_newest(), Some("D"));
    assert_eq!(values.remove(1), Some("C"));
    values.insert(1, "E").unwrap();
    println!("{:?}", values.iter().collect::<Vec<_>>()); // A, E, B
}
