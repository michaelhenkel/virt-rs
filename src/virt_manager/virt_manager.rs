use std::env;
use virt::connect::Connect;

pub struct VirtManager{
    pub conn: Connect,
}

impl VirtManager{
    pub fn new() -> VirtManager{
        let uri = env::args().nth(1);
        println!("Attempting to connect to hypervisor: '{:?}'", uri);

        let conn = match Connect::open(uri.as_deref().unwrap()) {
            Ok(c) => c,
            Err(e) => panic!("No connection to hypervisor: {} ", e),
        };

        VirtManager{
            conn,
        }
    }
}