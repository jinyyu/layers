extern crate layers;

use layers::layer::StreamID;
use layers::layer::TCPStream;
use std::collections::HashMap;
use std::boxed::Box;

#[test]
fn test_id() {
    let mut map = HashMap::new();
    let id = StreamID::new(1, 2, 4, 5);

    assert!(!map.contains_key(&id));

    {
        map.entry(id).or_insert_with(|| -> Box<TCPStream> {
            assert!(true);
            println!("---------------------------------------insert new");
            let stream = TCPStream {};
            return Box::new(stream);
        });
    }

    assert!(map.contains_key(&id));

    map.entry(id).or_insert_with(|| -> Box<TCPStream>  {
        assert!(false);

        println!("---------------------------------------insert new");
        let stream = TCPStream {};
        return Box::new(stream);
    });


}