extern crate layers;


struct Stream {
    val: u32,
}

impl Stream {
    fn update(&mut self, val:u32) {
        self.val = val
    }
}


struct Contest<'s> {
    stream: &'s Stream
}


impl<'s> Contest<'s> {
    fn new(stream: &Stream) -> Contest {
        Contest {
            stream,
        }
    }

    fn get(&self) ->u32 {
        self.stream.val
    }
}


#[test]
fn test_id() {
    let mut stream = Stream {
        val: 1,
    };

    let refstream = &stream;

    stream.val = 32;


    assert_eq!(stream.val, refstream.val)
}