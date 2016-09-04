
use ::u::U;
use ::term::{Term};
use arrayvec::ArrayVec;
use data_encoding::hex;
use byteorder::{BigEndian, WriteBytesExt};
use std::cmp::max;

pub struct Sha256 {
    pub data : Vec<U>,
    pub digest : Vec<U>,
}


impl Sha256 {

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


        // append the bit '1' to the message
        // append k bits '0', where k is the minimum number >= 0 such that the resulting message
        //     length (modulo 512 in bits) is 448.

        assert!(len_message_in_last_u_in_bits <= 32);
        let total_msg_len = (data.len() - 1) * 32 + len_message_in_last_u_in_bits;
        let msg_len_modulo = total_msg_len % 512;

        let mut bits_to_add = if msg_len_modulo <= 448 { 448 - msg_len_modulo }
                          else { 448 + (512 - msg_len_modulo) };

        //println!("bits to add {:?}", bits_to_add);

        // 448 is multiple of 32, so fill every U32
        // Fill the started one
        if len_message_in_last_u_in_bits == 32 && bits_to_add > 0 {
            //println!("frst");
            assert!(bits_to_add >= 32);
            data.push(U::from_const(0x8000000));
            bits_to_add -= 32;
        } else {
            //println!("sec");
            assert!(len_message_in_last_u_in_bits > 0);
            assert!(bits_to_add > 0);
            let mut u = data.pop().unwrap();
            u.bits[32 - len_message_in_last_u_in_bits - 1] = Term::c1();
            for i in 0..(32 - len_message_in_last_u_in_bits - 1) {
                u.bits[i] = Term::c0();
            }
            //println!("{:?}", u);
            data.push(u);
            bits_to_add -= 32 - len_message_in_last_u_in_bits;
        }
        assert!(bits_to_add % 32 == 0);
        //println!("bits to add {:?}", bits_to_add);

        while bits_to_add > 0 {
            data.push(U::from_const(0u32));
            bits_to_add -= 32;
        }


        // append length of message (without the '1' bit or padding), in bits, as 64-bit big-endian integer
        //     (this will make the entire post-processed length a multiple of 512 bits)

        assert!(total_msg_len <= u32::max_value() as usize);
        data.push(U::from_const(0u32));
        data.push(U::from_const(total_msg_len as u32));

        assert!(data.len() % 16 == 0);

        for chunk in data.windows(16) {
            // create a 64-entry message schedule array w[0..63] of 32-bit words
            // (The initial values in w[0..63] don't matter, so many implementations zero them here)
            // copy chunk into first 16 words w[0..15] of the message schedule array
            let mut w : Vec<U> = vec![];
            assert!(chunk.len() == 16);
            for c in chunk.iter() { // 0..15
                w.push(c.clone());
            }

            // Extend the first 16 words into the remaining 48 words w[16..63] of the message schedule array:
            // for i from 16 to 63
            //     s0 := (w[i-15] rightrotate 7) xor (w[i-15] rightrotate 18) xor (w[i-15] rightshift 3)
            //     s1 := (w[i-2] rightrotate 17) xor (w[i-2] rightrotate 19) xor (w[i-2] rightshift 10)
            //     w[i] := w[i-16] + s0 + w[i-7] + s1
            for i in 16..64 {
                let s0 = (w[i-15].rotate_right(7) ^ w[i-15].rotate_right(18)) ^ w[i-15].shift_right(3);
                let s1 = (w[i-2].rotate_right(17) ^ w[i-2].rotate_right(19)) ^ w[i-2].shift_right(10);
                let nextw = &w[i-16] + &s0 + &w[i-7] + &s1;
                w.push(nextw);
            }


            // Initialize working variables to current hash value:
            let mut a = h0.clone();
            let mut b = h1.clone();
            let mut c = h2.clone();
            let mut d = h3.clone();
            let mut e = h4.clone();
            let mut f = h5.clone();
            let mut g = h6.clone();
            let mut h = h7.clone();

            for i in 0..64 {
                // S1 := (e rightrotate 6) xor (e rightrotate 11) xor (e rightrotate 25)
                let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
                // ch := (e and f) xor ((not e) and g)
                let ch = ( &e & &f ) ^ ( !&e & &g);
                // temp1 := h + S1 + ch + k[i] + w[i]
                let temp1 = &h + &s1 + &ch + &k[i] + &w[i];
                // S0 := (a rightrotate 2) xor (a rightrotate 13) xor (a rightrotate 22)
                let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
                // maj := (a and b) xor (a and c) xor (b and c)
                let maj = (&a & &b) ^ (&a & &c) ^ (&b & &c);
                // temp2 := S0 + maj
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
            // Add the compressed chunk to the current hash value:
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
        }

    }

    pub fn reset(&self) {
        for h in self.digest.iter() {
            h.reset();
        }
    }

    pub fn eval_to_u32(&self) -> ArrayVec<[u32; 8]> {
        self.reset();
        let mut digest = ArrayVec::new();
        for h in self.digest.iter() {
            digest.push(h.eval_to_u32());
        }
        digest
    }

    pub fn nr_of_terms(&self) -> usize {
        self.reset();
        self.digest.iter().map(|u| u.nr_of_terms()).sum()
    }

    pub fn max_stack_size(&self) -> (usize, usize) {
        self.reset();
        self.digest.iter().map(|u| u.max_stack_size())
                              .fold((0,0), |maxmax, umax| {
                                    (max(maxmax.0, umax.0),
                                     max(maxmax.1, umax.1))
                              })
    }

    pub fn hex(&self) -> String {
        let intvec = self.eval_to_u32();
        let mut wtr = vec![];
        for i in intvec {
            wtr.write_u32::<BigEndian>(i).unwrap();
        }
        hex::encode(&wtr)
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

        // 'a'   ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb
        // 'a\n' 87428fc522803d31065e7bce3cf03fe475096631e5e07bbd7a0fde60c4cf25c7
        //assert_eq!(&s.hex().to_lowercase(), "ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb");
        assert_eq!(&s.hex().to_lowercase(), "87428fc522803d31065e7bce3cf03fe475096631e5e07bbd7a0fde60c4cf25c7");
    }

    #[bench]
    fn bench_add_two(b: &mut Bencher) {

        let data = vec![U::new_symbolic()];
        let s = Sha256::new(data, 8);

        let data : U = s.data[0].clone();
        data.set_byte(b'a', 0);

        b.iter(|| s.eval_to_u32());
    }
}
