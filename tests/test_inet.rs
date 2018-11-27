#[test]

use std::collections::HashMap;
extern crate layers;


fn test_cell() {


    ///
    let mut map: HashMap<&str, String> = HashMap::new();

     map.entry("poneyland").or_insert_with(|| {
         let s = "hoho".to_string();
         println!("9-----------------------------------------------------hihihihihihihihi");
         return s;
     });


    map.entry("poneyland").or_insert_with(|| ->String{
        let s = "hoho".to_string();
        println!("10-----------------------------------------------------hihihihihihihihi");
        return s;
    });


    assert_eq!(map["poneyland"], "hoho".to_string());
}