#![feature(test)]

extern crate arrayvec;
extern crate data_encoding;
extern crate byteorder;
extern crate test;

pub mod u;
pub mod term;
pub mod sha;

use sha::Sha256;
use u::U;

fn main() {
    let data = vec![U::new_symbolic()];
    let s = Sha256::new(data, 16);

    let data : U = s.data[0].clone();
    println!("{:?}", data);
    data.set_byte(b'a', 0);
    data.set_byte(b'\n', 1);

    println!("Sha256('a') = {:?}", &s.hex());

    s.digest[0].bits[0].reset();
    println!("{:?}", s.max_stack_size());
    println!("{:?}", s.nr_of_terms());


}