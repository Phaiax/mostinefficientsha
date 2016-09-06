//! `linopt::Linopt`: Linear optimization

use ::u::U;
use ::term::RTerm;
use ::sha::Sha256;
use ::util::{dehex, hex, f64bits_to_u32, u32_to_f64bits};
use std::cmp::{min};
use arrayvec::ArrayVec;
use std::fmt;


/// Simple optimizer that tries to use the fuzzy Sha256 implementation to
/// try to optimize to a target hash using a very simple linerarization algorithm.
pub struct Linopt {
    /// The lazy fuzzy hasl algorith,
    sha : Sha256,
    /// The target value
    target_hash : Vec<f64>,
    /// References to all the symbolic input `Term`s.
    input_bits : Vec<RTerm>,
}

impl Linopt {

    /// Uses `len_input_bytes` as the length of the input to the sha256 hash algorithm.
    /// `target_hash` is the hash it tries to optimize to.
    pub fn new(len_input_bytes : usize, target_hash : &str) -> Linopt {

        assert!(len_input_bytes >= 1);
        let mut len_input_bits : usize = len_input_bytes * 8;
        let mut input_data = Vec::with_capacity(len_input_bytes % 4 + 1);
        let mut input_bits = Vec::<RTerm>::with_capacity(len_input_bits);
        let mut len_message_in_last_u_in_bits = 0;

        // Create `U` s for input to sha256.
        while len_input_bits > 0 {
            let u = U::new_symbolic();
            // Keep references to all symbolic `Term`s.
            len_message_in_last_u_in_bits = min(len_input_bits, 32);
            for b in u.bits.iter().rev().take(len_message_in_last_u_in_bits) {
                input_bits.push(b.clone());
            }
            len_input_bits = len_input_bits.saturating_sub(32);
            input_data.push(u.clone());
        }

        Linopt {
            sha : Sha256::new(input_data, len_message_in_last_u_in_bits),
            target_hash : u32_to_f64bits(dehex(&target_hash).as_ref()),
            input_bits : input_bits,
        }

    }

    /// Distance measure to the `target_hash`. Return value between 0 and 1.
    fn distance(&self, b : &[f64]) -> f64 {
        assert!(256 == b.len());
        let mut dist = 0.0;
        for (a,b) in self.target_hash.iter().zip(b.iter()) {
            dist += (a - b).abs();
        }
        dist / 256.0
    }

    /// Inits all input bits to 0.5.
    pub fn init(&self) {
        for b in self.input_bits.iter() {
            b.set(0.5);
        }
    }

    /// Run optimization for `rounds` rounds.
    pub fn optimize(&self, rounds : usize) {
        for _ in 0..rounds {

            for (i, b) in self.input_bits.iter().enumerate() {

                // optimize input bit b

                let unchanged = self.sha.evaluate();
                let epsilon = 0.01;
                let sign = if b.evaluate() <= epsilon { 1.0 } else {-1.0 };
                let epsilon = epsilon * sign;

                b.set(b.evaluate() + epsilon);
                let changed = self.sha.evaluate();
                let unchanged_dist = self.distance(&unchanged[..]);
                println!("Dist: {}", unchanged_dist);
                println!("{}", hex(&f64bits_to_u32(&changed[..])[..]));
                let derivative = (self.distance(&changed[..]) - unchanged_dist) / epsilon;

                b.set( (b.evaluate() + derivative * epsilon * 0.1).min(0.0).max(1.0) );
                println!("b[{}] = {}", i, b.evaluate());

            }
        }
    }

    /// Evaluate sha algorithm to u32s.
    pub fn eval_to_u32(&self) -> ArrayVec<[u32; 8]> {
        self.sha.eval_to_u32()
    }

    /// Evaluate sha algorithm to ascii representation.
    pub fn hex(&self) -> String {
        self.sha.hex()
    }

}

impl fmt::Debug for Linopt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.sha.fmt(f)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newlinopt() {
        let l = Linopt::new(64, "87428fc522803d31065e7bce3cf03fe475096631e5e07bbd7a0fde60c4cf25c7");
        println!("{:?}", l);
        l.init();
    }

}