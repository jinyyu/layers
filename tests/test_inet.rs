use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::collections::HashMap;


struct Abc {
    a : u8
}

#[test]
fn test_cell() {
    for i in 0..10 {
        println!("hi number {} from the spawned thread!", i);
    }
}