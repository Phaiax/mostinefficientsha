//! `u::U`: A fuzzy 32bit integer

use ::term::{RTerm, Term};
use std::convert::From;
use std::fmt;
use arrayvec::ArrayVec;
use std::cmp::max;

/// This type represents a u32, an unsigned number made of 32 bits.
/// The bits are not boolean, but represented by a lazily evaluated tree of
/// `Term`s.
///
/// This type implements some arithmetic operations on `u32`s that translate
/// into bit operations under the hood. For example the + operation is translated
/// into half adders and full adders, that themselves consist of bit operations
/// like and and or.
///
/// This type implements all operations needed to calculate a sha256 hash:
/// `rotate_right`, `shift_right >>`, Addition `+` and the bitwise operations
/// `and &`, `xor ^` and `not !`. Use the member functions or the implemented
/// operators. See `sha.rs` for example use.
///
/// This U can be constructed from a u32 constant number. The bits of that
/// number will translate into `Term`s of type `Constant`.
///
/// It can be constructed by creating 32 new symbolic `Term`s.
///
/// It can also be constructed by using operations like `>>`.
///
/// After construction, the bits can be manipulated using the `bits` field
/// of this struct.
#[derive(Clone)]
pub struct U {
    /// The LSBit is bits[0], the MSBit is bits[31]. The byte order is big endian.
    /// Push LSB first.
    pub bits : ArrayVec<[RTerm ; 32]>,
}

impl U {

    /// Create a new `U` with 32 `Term`s of type `Symbol`. Do not forget to
    /// set the value of each symbol afterwards. You can set the value of each
    /// bit by using `my_u.bits[22].set(0.3f64)`. Or set 8 bits of this `U` with
    /// `my_u.set_byte('e', 2)`.
    pub fn new_symbolic() -> U {
        let mut u = U { bits : ArrayVec::new() };
        for _ in 0..32 {
            u.bits.push(Term::symbol());
        }
        u
    }

    /// Create a new `U` with 32 `Term`s of type `Constant`.
    pub fn from_const(mut c : u32) -> U {
        let mut u = U { bits : ArrayVec::new() };
        for _ in 0..32 {
            u.bits.push(Term::constant(c & 1u32 == 1u32));
            c = c >> 1; // c is little endian, but >> delivers the bits from LSBit to MSBit.
        }
        u
    }

    /// Sets the bits of one byte of this u32 to their min/max values 1.0 and 0.0 depending on `set_value`.
    /// `bytenum` must be one of 0, 1, 2, 3.
    /// Byte 0 is the most significant byte of the four byte representation of this u32.
    ///
    /// Panics if the relevant bits/`Term`s of this `U` are not of type `Symbol`.
    pub fn set_byte(&self,mut set_value : u8, bytenum : usize) {
        assert!(bytenum <= 3);
        // LSByte 3: bits[8..0]
        // MSByte 0: bits[32..24]
        for b in self.bits.iter().skip((3-bytenum)*8).take(8) {
            b.set((set_value & 1u8) as u8 as f64);
            set_value = set_value >> 1;
        }
    }

    /// Sets all bits of this `U` to a value dependend on `bytes`. See `set_byte()` for details.
    ///
    /// Panics if any bit/`Term` of this `U` is not of type `Symbol`.
    pub fn set_bytes(&self, bytes : &[u8]) {
        self.set_byte(bytes[0], 0);
        self.set_byte(bytes[1], 1);
        self.set_byte(bytes[2], 2);
        self.set_byte(bytes[3], 3);
    }

    /// Recursively resets the cache of this self's bits/`Term`s and the `Term`s self's `Term`s depend on.
    ///
    /// This need to be called after and before using `nr_of_terms()`,
    /// `nr_of_terms_flattened()` and `max_logic_depth_and_max_stack_size()`.
    ///
    /// It also needs to be called before `evaluate()` and `eval_to_u32()`, but
    /// only if a symbolic `Term` has been modified.
    pub fn reset(&self) {
        for b in self.bits.iter() {
            b.reset();
        }
    }

    /// Evaluates all bits to a f64 value. Push these values to `out`.
    /// Pushes the MSBit first.
    pub fn evaluate<'a>(&self, out : &mut Vec<f64>) {
        for b in self.bits.iter().rev() {
            out.push(b.evaluate());
        }
    }

    /// Evaluate all bits to a `f64` value, then round that value to 0 or 1
    /// and assemble a `u32` with these bits.
    pub fn eval_to_u32(&self) -> u32 {
        let mut out = 0;
        for b in self.bits.iter().rev().map(|b| b.evaluate() >= 0.5) {
            out = out << 1; // << is independend of the little endian nature of out.
            if b {
                out = out | 1u32;
            }
        }
        out
    }


    /// Max stack size needed when evaluating each bit of this `U` recursively.
    ///
    /// It also returns the maximum logic depth.
    ///
    /// Returns: (max_logic_depth, max_stacksize)
    ///
    /// Note: manually call reset() before using this function.
    pub fn max_logic_depth_and_max_stack_size(&self) -> (usize, usize) {
        self.bits.iter().rev().map(|b| b.max_logic_depth_and_max_stack_size(0))
                              .fold((0,0), |maxmax, umax| {
                                    (max(maxmax.0, umax.0),
                                     max(maxmax.1, umax.1))
                              })
    }

    /// Returns the number of `RTerms` that contribute to the evaluation of this `U`.
    ///
    /// Note: manually call reset() before using this function.
    pub fn nr_of_terms(&self) -> usize {
        self.bits.iter().map(|u| u.nr_of_terms()).sum()
    }


    /// Returns the number of `Term`s that contribute to the evaluation of this `U` if the
    /// tree would have been flattened for each bit and subtree.
    ///
    /// Note: manually call reset() before using this function.
    pub fn nr_of_terms_flattened(&self) -> usize {
        self.bits.iter().map(|u| u.nr_of_terms_flattened()).sum()
    }

    /// Returns a new `U` that evaluates to `self`s value, but bitrotated by `x`
    /// to the right. Rotation happens without any carry bit.
    /// `x` must be less or equal 32.
    pub fn rotate_right(&self, x : usize) -> U {
        let mut u = U { bits : ArrayVec::new() };
        // Old   :     1000 0000 1100 0000 1010 0000 1001 0011
        // >>3   : 011 1000 0000 1100 0000 1010 0000 1001 0
        assert!(x <= 32);
        for b in self.bits.iter().skip(x) {
            u.bits.push(b.clone());
        }
        for b in self.bits.iter().take(x) {
            u.bits.push(b.clone());
        }
        u
    }

    /// Returns a new `U` that evaluates to `self`s value, but bitshifted by `x`
    /// to the right.
    /// `x` must be less than 32.
    pub fn shift_right(&self, x : usize) -> U {
        let mut u = U { bits : ArrayVec::new() };
        // Old   :     1000 0000 1100 0000 1010 0000 1001 0011
        // >>3   : 000 1000 0000 1100 0000 1010 0000 1001 0
        assert!(x <= 31);
        for b in self.bits.iter().skip(x) {
            u.bits.push(b.clone());
        }
        for _ in 0..x {
            u.bits.push(Term::constant(false));
        }
        u
    }

    /// Returns a new `U` that evaluates to the bitwise xor with `rhs`.
    pub fn xor(&self, rhs : &U) -> U {
        let mut u = U { bits : ArrayVec::new() };
        for (b1, b2) in self.bits.iter().zip(rhs.bits.iter()) {
            u.bits.push(Term::xor(b1, b2));
        }
        u
    }

    /// Returns a new `U` that evaluates to the bitwise and with `rhs`.
    pub fn and(&self, rhs : &U) -> U {
        let mut u = U { bits : ArrayVec::new() };
        for (b1, b2) in self.bits.iter().zip(rhs.bits.iter()) {
            u.bits.push(Term::and(b1, b2));
        }
        u
    }

    /// Returns a new `U` that evaluates to the bitwise not with `rhs`.
    pub fn not(&self) -> U {
        let mut u = U { bits : ArrayVec::new() };
        for b in self.bits.iter() {
            u.bits.push(Term::not(b));
        }
        u
    }

    /// Returns a new `U` that evaluates to the arithmethic addition with `rhs`.
    pub fn add(&self, rhs : &U) -> U {
        let mut u = U { bits : ArrayVec::new() };
        let (s, mut c) = Term::half_add(&self.bits[0], &rhs.bits[0]);
        u.bits.push(s);
        for (b1, b2) in self.bits.iter().zip(rhs.bits.iter()).skip(1) {
            let (s2, c2) = Term::full_add(&b1, &b2, &c);
            u.bits.push(s2);
            c = c2;
        }
        u
    }

}

impl From<u32> for U {
    fn from(c : u32) -> Self {
        Self::from_const(c)
    }
}

impl fmt::Debug for U {
    /// Debug: MSB...LSB
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "U["));
        for b in self.bits.iter().rev() {
            try!(b.fmt(f));
        }
        write!(f, "]")
    }
}


use std::ops;


macro_rules! op {
    ($ops:ident, $opsfunc:ident, $termmethod:ident) => (

        impl ops::$ops<U> for U {
            type Output = U;
            fn $opsfunc(self, rhs: U) -> Self::Output {
                U::$termmethod(&self, &rhs)
            }
        }

        impl<'a> ops::$ops<U> for &'a U {
            type Output = U;
            fn $opsfunc(self, rhs: U) -> Self::Output {
                U::$termmethod(self, &rhs)
            }
        }

        impl<'a> ops::$ops<&'a U> for U {
            type Output = U;
            fn $opsfunc(self, rhs: &U) -> Self::Output {
                U::$termmethod(&self, rhs)
            }
        }

        impl<'a, 'b> ops::$ops<&'a U> for &'b U {
            type Output = U;
            fn $opsfunc(self, rhs: &U) -> Self::Output {
                U::$termmethod(self, rhs)
            }
        }
    )
}

macro_rules! op_usize {
    ($ops:ident, $opsfunc:ident, $termmethod:ident) => (

        impl ops::$ops<usize> for U {
            type Output = U;
            fn $opsfunc(self, rhs: usize) -> Self::Output {
                U::$termmethod(&self, rhs)
            }
        }

        impl<'a> ops::$ops<usize> for &'a U {
            type Output = U;
            fn $opsfunc(self, rhs: usize) -> Self::Output {
                U::$termmethod(self, rhs)
            }
        }
    )
}

macro_rules! op_norhs {
    ($ops:ident, $opsfunc:ident, $termmethod:ident) => (

        impl ops::$ops for U {
            type Output = U;
            fn $opsfunc(self) -> Self::Output {
                U::$termmethod(&self)
            }
        }

        impl<'a> ops::$ops for &'a U {
            type Output = U;
            fn $opsfunc(self) -> Self::Output {
                U::$termmethod(self)
            }
        }
    )
}



op_usize!(Shr, shr, shift_right);
op!(Add, add, add);
op_norhs!(Not, not, not);
op!(BitXor, bitxor, xor);
op!(BitAnd, bitand, and);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn const_back_and_forth() {
        let a : U = 1234512u32.into();
        println!("{:?}", a);
        let b = a.eval_to_u32();
        assert_eq!(1234512u32, b);
    }

    #[test]
    fn shift_right() {
        let a = 1231414u32;
        let u1 : U = a.into();
        let u_shifted = &u1 >> 2;
        let a_shifted = a >> 2;
        assert_eq!(a_shifted, u_shifted.eval_to_u32());
        let u_shifted = &u1 >> 31;
        let a_shifted = a >> 31;
        assert_eq!(a_shifted, u_shifted.eval_to_u32());
        let u_shifted = &u1 >> 0;
        let a_shifted = a >> 0;
        assert_eq!(a_shifted, u_shifted.eval_to_u32());
    }

    #[test]
    fn rotate_right() {
        let a = 1231414u32;
        let u1 : U = a.into();
        let u_shifted = u1.rotate_right(2);
        let a_shifted = a.rotate_right(2);
        assert_eq!(a_shifted, u_shifted.eval_to_u32());
        let u_shifted = u1.rotate_right(0);
        let a_shifted = a.rotate_right(0);
        assert_eq!(a_shifted, u_shifted.eval_to_u32());
        let u_shifted = u1.rotate_right(31);
        let a_shifted = a.rotate_right(31);
        assert_eq!(a_shifted, u_shifted.eval_to_u32());
        let u_shifted = u1.rotate_right(32);
        let a_shifted = a.rotate_right(32);
        assert_eq!(a_shifted, u_shifted.eval_to_u32());
    }

    #[test]
    fn xor() {
        let a1 = 11241257u32;
        let a2 = 723573424u32;
        let u1 : U = a1.into();
        let u2 : U = a2.into();
        assert_eq!(a1 ^ a2, (u1 ^ u2).eval_to_u32());
    }

    #[test]
    fn and() {
        let a1 = 11241257u32;
        let a2 = 723573424u32;
        let u1 : U = a1.into();
        let u2 : U = a2.into();
        assert_eq!(a1 & a2, (u1 & u2).eval_to_u32());
    }

    #[test]
    fn not() {
        let a1 = 11241257u32;
        let u1 : U = a1.into();
        assert_eq!(!a1, a1 ^ u32::max_value());
        assert_eq!(!a1, !u1.eval_to_u32());
    }

    #[test]
    fn add() {
        let a1 = 11241257u32;
        let a2 = 723573424u32;
        let u1 : U = a1.into();
        let u2 : U = a2.into();
        assert_eq!(a1 + a2, (u1 + u2).eval_to_u32());
    }

    #[test]
    fn endianess() {
        let h0_psc : u32 = 0x6a09e667; // 6a is msb, 67 is lsb
        let h1_psc : u32 = 0xbb67ae85;
        let k0_psc : u32 = 0x428a2f98;
        let k2_psc : u32 = 0xb5c0fbcf;


        let h0_cpp : u32 = 1779033703;
        let h1_cpp : u32 = -1150833019i32 as u32;
        let k0_cpp : i32 = 1116352408;
        let k2_cpp : i32 = -1245643825;

        assert!(h0_psc == h0_cpp);
        assert!(h1_psc == h1_cpp);
        assert!(k0_psc == k0_cpp as u32);
        assert!(k2_psc == k2_cpp as u32);

        println!("{:?}", U::from_const(h0_psc));
        println!("{:?}", U::from_const(h1_psc));
        println!("{:?}", U::from_const(k0_psc));
        println!("{:?}", U::from_const(k2_psc));

    }
}
