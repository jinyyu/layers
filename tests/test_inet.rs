#[test]
fn test_cell() {
    use std::num::Wrapping;

    let a = Wrapping(std::u32::MAX);
    let b = a + Wrapping(1);

    println!("============================================{}", b.0);
}