use std::collections::HashMap;

#[test]
fn test() {
    let mut map: HashMap<u32, Vec<u8>> = HashMap::new();
    map.insert(1, Vec::new());
    map.insert(2, Vec::new());
    map.insert(3, Vec::new());

    {
        let mut iter = map.iter_mut();

        loop {
            match iter.next() {
                None => {
                    break;
                }
                Some(item) => {
                    let abc = item.0;
                    let def = item.1;
                    def.insert(0, 1);
                }
            }
        }
    }

    for item in map.iter() {
        println!(
            "<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<{}-{}",
            item.0,
            item.1.len()
        );
    }
}

#[test]
fn test_borrow() {
    let mut abc = 1;
    let def = &abc;

    let &kkk = &abc;
}
