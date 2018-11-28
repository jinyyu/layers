#[test]
fn test_cell() {
    use std::cell::RefCell;

    let c = RefCell::new(5);

    *c.borrow_mut() = 7;

    assert_eq!(*c.borrow(), 7);

}