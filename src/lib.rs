#![feature(test)]

extern crate arrayvec;
extern crate data_encoding;
extern crate byteorder;
extern crate test;

pub mod u;
pub mod term;
pub mod sha;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
