use std::collections::HashMap;

#[test]
fn test() {
    let mut map: HashMap<u32, &str> = HashMap::new();
    map.insert(1, "one");
    map.insert(2, "two");
    map.insert(3, "three");

    map.retain(|k, v| k % 2 != 0);

    for item in map.iter() {
        println!("{}-{}", item.0, item.1);
    }
}
