use std::collections::BTreeMap;
use std::vec::Vec;

#[test]
fn test() {
    let mut map: BTreeMap<u32, &str> = BTreeMap::new();
    map.insert(1, "one");
    map.insert(2, "two");
    map.insert(3, "three");


    let mut keys: Vec<u32> = Vec::new();
    {
        let iter = map.iter().filter(|item| {
            *item.0 % 2 == 0
        });

        for item in iter {
            keys.push(*item.0)
        }
    }


    for key in keys.iter() {
        map.remove(key).unwrap();
    }


    for item in map.iter() {
        println!("{}-{}", item.0, item.1);
    }
}