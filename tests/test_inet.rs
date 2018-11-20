use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::collections::HashMap;


struct Abc {
    a : u8
}

#[test]
fn test_cell() {
    use std::thread;
    use std::sync::mpsc;
    use std::time::Duration;

    fn main() {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            for received in rx {
                println!("Got: {}", received);
            }
        });

        let vals = vec![
            String::from("hi"),
            String::from("from"),
            String::from("the"),
            String::from("thread"),
        ];

        for val in vals {
            tx.send(val).unwrap();
            thread::sleep(Duration::from_secs(1));
        }


    }
}