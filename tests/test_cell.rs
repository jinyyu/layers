use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::collections::HashMap;


struct Abc {
    a : u8
}

#[test]
fn test_cell() {

    let b = Rc::new(Abc{ a : 10});
    let c = b.clone();
    assert_eq!(10, c.a);
    print!("{}", c.a);
}