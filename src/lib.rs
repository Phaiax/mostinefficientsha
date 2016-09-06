#![feature(test)]

//! This crate tries to break SHA256. (But fails unfortunately).
//!
//! It implements SHA-256 in the most inefficient fashion ever. Usually, SHA-256
//! operates on integers with 32 bits (u32). This implementation uses one double (f64)
//! for each bit of each integer, thereby allowing to use 8 fuzzy input bits for each input byte.
//!
//! [Term](term/struct.Term.html) represents a fuzzy bit either as a settable bit (constant or symbolic)
//! or as a bitwise combination of other `Term`s, thereby creating a treelike graph of `Term`s.
//! These terms are evaluated lazily.
//!
//! [U](u/struct.U.html) combines 32 `Term`s and represents a fuzzy integer.
//! U also implements _high level_ operations like shifting, rotating, adding.
//! These operations will be transformed into fuzzy lazily evaluated bit operations.
//!
//! [Sha256](sha/struct.Sha256.html) uses these `U`s to calculate the SHA-256
//! algorithm.
//!
//! [Linopt](linopt/struct.Linopt.html) uses the fuzzy `Sha256` to try to break it.
//! No chance.




extern crate arrayvec;
extern crate data_encoding;
extern crate byteorder;
extern crate test;

pub mod util;
pub mod term;
pub mod u;
pub mod sha;
pub mod linopt;
