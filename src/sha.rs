//! `sha::Sha256`: Sha256 using `U`s.

use ::util::hex;
use ::u::U;
use ::term::{Term};
use arrayvec::ArrayVec;
use std::cmp::max;
use std::fmt;


/// This struct does a lazy SHA-256 hash calculation with fuzzy `f64` bits.
/// It first creates a tree
/// of thouthands of `Term`s that represent the bitwise calculations needed to
/// calculate the final hash. It then calculates that hash by evaluating the
/// `Term`s that represent the bits of the final 256 bit hash value.
///
/// It can do this evaluation to `f64` fuzzy bits or to a
/// `[8;u32]` or ascii representation of the hash by rounding the `f64`s.
///
/// The input data is made of `Symbol` type `Term`s, that have to be set before
/// evaluation.
pub struct Sha256 {
    /// The original input data. Some of the trailing bits may have been replaced. See `new()`.
    pub data : Vec<U>,
    pub input_data_len_in_bits : usize,
    /// The final tree structure is hidden here.
    pub digest : Vec<U>,
}


impl Sha256 {

    /// Create a new tree of `Term`s, that lazily calculates the SHA-256 hash of `data`.
    ///
    /// `len_message_in_last_u_in_bits` is the number of bits in `data.last()` that
    /// are part of the input data. All following bits in `data.last()` will be
    /// ignored (and replaced).
    ///
    /// You can access `data` afterwards using the `data` field.
    ///
    /// This function just does SHA-256, but with `U`s instead of `u32`s.
    pub fn new(mut data : Vec<U>, len_message_in_last_u_in_bits : usize) -> Sha256 {

        let mut h0 = U::from_const(0x6a09e667u32);
        let mut h1 = U::from_const(0xbb67ae85u32);
        let mut h2 = U::from_const(0x3c6ef372u32);
        let mut h3 = U::from_const(0xa54ff53au32);
        let mut h4 = U::from_const(0x510e527fu32);
        let mut h5 = U::from_const(0x9b05688cu32);
        let mut h6 = U::from_const(0x1f83d9abu32);
        let mut h7 = U::from_const(0x5be0cd19u32);

        let k : [U; 64] = [ 0x428a2f98.into(), 0x71374491.into(), 0xb5c0fbcf.into(), 0xe9b5dba5.into(), 0x3956c25b.into(), 0x59f111f1.into(), 0x923f82a4.into(), 0xab1c5ed5.into(), 0xd807aa98.into(), 0x12835b01.into(), 0x243185be.into(), 0x550c7dc3.into(), 0x72be5d74.into(), 0x80deb1fe.into(), 0x9bdc06a7.into(), 0xc19bf174.into(), 0xe49b69c1.into(), 0xefbe4786.into(), 0x0fc19dc6.into(), 0x240ca1cc.into(), 0x2de92c6f.into(), 0x4a7484aa.into(), 0x5cb0a9dc.into(), 0x76f988da.into(), 0x983e5152.into(), 0xa831c66d.into(), 0xb00327c8.into(), 0xbf597fc7.into(), 0xc6e00bf3.into(), 0xd5a79147.into(), 0x06ca6351.into(), 0x14292967.into(), 0x27b70a85.into(), 0x2e1b2138.into(), 0x4d2c6dfc.into(), 0x53380d13.into(), 0x650a7354.into(), 0x766a0abb.into(), 0x81c2c92e.into(), 0x92722c85.into(), 0xa2bfe8a1.into(), 0xa81a664b.into(), 0xc24b8b70.into(), 0xc76c51a3.into(), 0xd192e819.into(), 0xd6990624.into(), 0xf40e3585.into(), 0x106aa070.into(), 0x19a4c116.into(), 0x1e376c08.into(), 0x2748774c.into(), 0x34b0bcb5.into(), 0x391c0cb3.into(), 0x4ed8aa4a.into(), 0x5b9cca4f.into(), 0x682e6ff3.into(), 0x748f82ee.into(), 0x78a5636f.into(), 0x84c87814.into(), 0x8cc70208.into(), 0x90befffa.into(), 0xa4506ceb.into(), 0xbef9a3f7.into(), 0xc67178f2.into(), ];


        // WIKI: append the bit '1' to the message
        // WIKI: append k bits '0', where k is the minimum number >= 0 such that the resulting message
        // WIKI:     length (modulo 512 in bits) is 448.

        assert!(len_message_in_last_u_in_bits <= 32);
        let total_msg_len = (data.len() - 1) * 32 + len_message_in_last_u_in_bits;
        let msg_len_modulo = total_msg_len % 512;

        let mut bits_to_add = if msg_len_modulo <= 448 { 448 - msg_len_modulo }
                          else { 448 + (512 - msg_len_modulo) };

        // 448 is multiple of 32, so add 32 bit chunks aka `U`s ...

        // ... but first put `Constant` `Term`s into the inchoate one if there is one
        if len_message_in_last_u_in_bits == 32 && bits_to_add > 0 {
            assert!(bits_to_add >= 32);
            data.push(U::from_const(0x8000_0000));
            bits_to_add -= 32;
        } else {
            assert!(len_message_in_last_u_in_bits > 0);
            assert!(bits_to_add > 0);
            let mut u = data.pop().unwrap();
            u.bits[32 - len_message_in_last_u_in_bits - 1] = Term::c1();
            for i in 0..(32 - len_message_in_last_u_in_bits - 1) {
                u.bits[i] = Term::c0();
            }
            data.push(u);
            bits_to_add -= 32 - len_message_in_last_u_in_bits;
        }
        assert!(bits_to_add % 32 == 0);

        while bits_to_add > 0 {
            data.push(U::from_const(0u32));
            bits_to_add -= 32;
        }


        // WIKI: append length of message (without the '1' bit or padding), in bits, as 64-bit big-endian integer
        // WIKI:     (this will make the entire post-processed length a multiple of 512 bits)

        assert!(total_msg_len <= u32::max_value() as usize);
        data.push(U::from_const(0u32));
        data.push(U::from_const(total_msg_len as u32));

        assert!(data.len() % 16 == 0);

        for chunk in data.chunks(16) {
            // WIKI: create a 64-entry message schedule array w[0..63] of 32-bit words
            // WIKI: (The initial values in w[0..63] don't matter, so many implementations zero them here)
            // WIKI: copy chunk into first 16 words w[0..15] of the message schedule array
            let mut w : Vec<U> = vec![];
            assert!(chunk.len() == 16);
            for c in chunk.iter() { // 0..15
                w.push(c.clone());
            }

            // WIKI: Extend the first 16 words into the remaining 48 words w[16..63] of the message schedule array:
            // WIKI: for i from 16 to 63
            // WIKI:     s0 := (w[i-15] rightrotate 7) xor (w[i-15] rightrotate 18) xor (w[i-15] rightshift 3)
            // WIKI:     s1 := (w[i-2] rightrotate 17) xor (w[i-2] rightrotate 19) xor (w[i-2] rightshift 10)
            // WIKI:     w[i] := w[i-16] + s0 + w[i-7] + s1
            for i in 16..64 {
                let s0 = (w[i-15].rotate_right(7) ^ w[i-15].rotate_right(18)) ^ w[i-15].shift_right(3);
                let s1 = (w[i-2].rotate_right(17) ^ w[i-2].rotate_right(19)) ^ w[i-2].shift_right(10);
                let nextw = &w[i-16] + &s0 + &w[i-7] + &s1;
                w.push(nextw);
            }


            // WIKI: Initialize working variables to current hash value:
            let mut a = h0.clone();
            let mut b = h1.clone();
            let mut c = h2.clone();
            let mut d = h3.clone();
            let mut e = h4.clone();
            let mut f = h5.clone();
            let mut g = h6.clone();
            let mut h = h7.clone();

            for i in 0..64 {
                // WIKI: S1 := (e rightrotate 6) xor (e rightrotate 11) xor (e rightrotate 25)
                let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
                // WIKI: ch := (e and f) xor ((not e) and g)
                let ch = ( &e & &f ) ^ ( !&e & &g);
                // WIKI: temp1 := h + S1 + ch + k[i] + w[i]
                let temp1 = &h + &s1 + &ch + &k[i] + &w[i];
                // WIKI: S0 := (a rightrotate 2) xor (a rightrotate 13) xor (a rightrotate 22)
                let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
                // WIKI: maj := (a and b) xor (a and c) xor (b and c)
                let maj = (&a & &b) ^ (&a & &c) ^ (&b & &c);
                // WIKI: temp2 := S0 + maj
                let temp2 = &s0 + &maj;

                h = g.clone();
                g = f.clone();
                f = e.clone();
                e = &d + &temp1;
                d = c.clone();
                c = b.clone();
                b = a.clone();
                a = &temp1 + &temp2;
            }
            // WIKI: Add the compressed chunk to the current hash value:
            h0 = &h0 + &a;
            h1 = &h1 + &b;
            h2 = &h2 + &c;
            h3 = &h3 + &d;
            h4 = &h4 + &e;
            h5 = &h5 + &f;
            h6 = &h6 + &g;
            h7 = &h7 + &h;
        }


        Sha256 {
            data : data,
            digest : vec![h0, h1, h2, h3, h4, h5, h6, h7],
            input_data_len_in_bits : total_msg_len,
        }

    }

    /// Resets the cache of all `RTerms` within the `Term` tree.
    pub fn reset(&self) {
        for h in self.digest.iter() {
            h.reset();
        }
    }

    /// Evaluates the digest/hash result into `u32`s by rounding the `f64` bits.
    pub fn eval_to_u32(&self) -> ArrayVec<[u32; 8]> {
        self.reset();
        let mut digest = ArrayVec::new();
        for h in self.digest.iter() {
            digest.push(h.eval_to_u32());
        }
        digest
    }

    /// Evaluates the digest/hash result into its ascii hash representation
    /// by rounding the `f64` bits.
    pub fn hex(&self) -> String {
        hex(&self.eval_to_u32())
    }

    /// Evaluates the digest/hash results into 256 `f64`s.
    ///
    /// returnval[0] is the MSBit of the first byte. The first byte corresponds
    /// to the start of the ascii hash representation.
    pub fn evaluate(&self) -> Vec<f64> {
        self.reset();
        let mut out = Vec::with_capacity(256);
        for u in self.digest.iter() {
            u.evaluate(&mut out);
        }
        out
    }

    /// Returns the number of `Term`s that were created to represent this
    /// instance of the SHA-256 algorithm. The number depends heavily on the
    /// length of the input data.
    pub fn nr_of_terms(&self) -> usize {
        self.reset();
        self.digest.iter().map(|u| u.nr_of_terms()).sum()
    }

    /// Returns the number of `Term`s that would make the flattened version
    /// of the tree of each bit of the digest.
    pub fn nr_of_terms_flattened(&self) -> usize {
        self.reset();
        self.digest.iter().map(|u| u.nr_of_terms_flattened()).sum()
    }

    /// Max stack size needed when evaluating each bit of the digest recursively.
    ///
    /// It also returns the maximum logic depth.
    ///
    /// Returns: (max_logic_depth, max_stacksize)
    pub fn max_logic_depth_and_max_stack_size(&self) -> (usize, usize) {
        self.reset();
        self.digest.iter().map(|u| u.max_logic_depth_and_max_stack_size())
                              .fold((0,0), |maxmax, umax| {
                                    (max(maxmax.0, umax.0),
                                     max(maxmax.1, umax.1))
                              })
    }

    /// Returns a String describing the statistics. Same as debug print.
    pub fn statistics(&self) -> String {
        format!("{:?}", self)
    }

}


impl fmt::Debug for Sha256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let nr_of_term = self.nr_of_terms();
        let nr_of_term_flat = self.nr_of_terms_flattened();
        let (max_logic_depth, max_stacksize) = self.max_logic_depth_and_max_stack_size();
        self.reset();

        write!(f, "Input data: {} bytes and {} bits = {} bits\nTotal RTerms: {}\nMaximum depth of logic elements: {}\nNeeded recursion depth for evaluation: {}\nFlattened tree size: {}",
            self.input_data_len_in_bits / 8,
            self.input_data_len_in_bits % 8,
            self.input_data_len_in_bits,
            nr_of_term,
            max_logic_depth,
            max_stacksize,
            nr_of_term_flat)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use ::u::U;
    use test::Bencher;

    #[test]
    fn gen() {
        let data = vec![U::new_symbolic()];
        data[0].clone().set_byte(b'a', 0);
        data[0].clone().set_byte(b'\n', 1);

        let s = Sha256::new(data, 16);

        println!("{:?}", s.data[0].clone());

        // echo -n 'a' | sha256sum
        // 'a'   ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb
        // echo 'a' | sha256sum
        // 'a\n' 87428fc522803d31065e7bce3cf03fe475096631e5e07bbd7a0fde60c4cf25c7
        //assert_eq!(&s.hex().to_lowercase(), "ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb");
        assert_eq!(&s.hex(), "87428fc522803d31065e7bce3cf03fe475096631e5e07bbd7a0fde60c4cf25c7");
    }

    #[test]
    fn test_sha_long() {
        let data : Vec<U> = (0..16u8).map(|i| {[i*4%16 + b'a',
                                                i*4%16 + b'b',
                                                i*4%16 + b'c',
                                                i*4%16 + b'd'] })
                                     .map(|b4| {
                                        let u = U::new_symbolic();
                                        u.set_bytes(&b4[..]);
                                        u
                                    }).collect();
        let s = Sha256::new(data, 32);
        // echo -n abcdefghijklmnopabcdefghijklmnopabcdefghijklmnopabcdefghijklmnop | sha256sum
        // 6679a7e0f1319c73d2f2444551c2730d796fb46e6cf5b349781c730ff9644d65
        assert_eq!(&s.hex(), "6679a7e0f1319c73d2f2444551c2730d796fb46e6cf5b349781c730ff9644d65");
    }

    #[test]
    fn test_sha_long_2() {
        let data : Vec<U> = (0..16u8).map(|i| {[i*4%16 + b'a',
                                                i*4%16 + b'b',
                                                i*4%16 + b'c',
                                                i*4%16 + b'd'] })
                                     .map(|b4| {
                                        let u = U::new_symbolic();
                                        u.set_bytes(&b4[..]);
                                        u
                                    }).collect();
        let s = Sha256::new(data, 8);
        // echo -n abcdefghijklmnopabcdefghijklmnopabcdefghijklmnopabcdefghijklm | sha256sum
        assert_eq!(&s.hex(), "4ec58b2ea3a686034907a0b6634076c289bca15fdeb70acd130f804a340143be");
    }


    #[bench]
    fn bench_sha_one_byte(b: &mut Bencher) {

        let data = vec![U::new_symbolic()];
        let s = Sha256::new(data, 8);

        let data : U = s.data[0].clone();
        data.set_byte(b'a', 0);

        b.iter(|| s.eval_to_u32());
    }


    #[bench]
    fn bench_sha_onehundred_bytes(b: &mut Bencher) {

        let data : Vec<U> = (0..100u8).map(|i| [i, i, i, i] )
                                     .map(|b4| {
                                        let u = U::new_symbolic();
                                        u.set_bytes(&b4[..]);
                                        u
                                    }).collect();

        let s = Sha256::new(data, 32); // 100%4=0

        b.iter(|| s.eval_to_u32());
    }

}
