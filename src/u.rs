
use ::term::{RTerm, Term};
use std::convert::From;
use std::fmt;
use arrayvec::ArrayVec;
use std::cmp::max;

#[derive(Clone)]
pub struct U {
    /// The LSB is bits[0], the MSB = bits[31]
    /// Push LSB first
    pub bits : ArrayVec<[RTerm ; 32]>,
}

impl U {

    pub fn new_symbolic() -> U {
        let mut u = U { bits : ArrayVec::new() };
        for _ in 0..32 {
            u.bits.push(Term::symbol());
        }
        u
    }

    pub fn from_const(mut c : u32) -> U {
        let mut u = U { bits : ArrayVec::new() };
        for _ in 0..32 {
            // print!("{:?} ", c);
            u.bits.push(Term::constant(c & 1u32 == 1u32));
            c = c >> 1;
        }
        u
    }

    /// first byte is that on the MSB side
    pub fn set_byte(&self,mut content : u8, bytenum : usize) {
        assert!(bytenum <= 3);
        for b in self.bits.iter().skip((3-bytenum)*8).take(8) {
            b.set((content & 1u8) as u8 as f64);
            content = content >> 1;
        }
    }

    pub fn eval_to_u32(&self) -> u32 {
        let mut out = 0;
        for b in self.bits.iter().rev() {
            let v = b.evaluate();
            out = out << 1;
            if v >= 0.5 {
                out = out | 1u32;
            }
            // print!("{:?} ", out);
        }
        out
    }

    /// note: manually call reset() before
    pub fn max_stack_size(&self) -> (usize, usize) {
        self.bits.iter().rev().map(|u| u.max_stack_size(0))
                              .fold((0,0), |maxmax, umax| {
                                    (max(maxmax.0, umax.0),
                                     max(maxmax.1, umax.1))
                              })
    }

    /// note: manually call reset() before
    pub fn nr_of_terms(&self) -> usize {
        self.bits.iter().map(|u| u.nr_of_terms()).sum()
    }

    pub fn reset(&self) {
        for b in self.bits.iter() {
            b.reset();
        }
    }

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

    pub fn xor(&self, rhs : &U) -> U {
        let mut u = U { bits : ArrayVec::new() };
        for (b1, b2) in self.bits.iter().zip(rhs.bits.iter()) {
            u.bits.push(Term::xor(b1, b2));
        }
        u
    }
    pub fn and(&self, rhs : &U) -> U {
        let mut u = U { bits : ArrayVec::new() };
        for (b1, b2) in self.bits.iter().zip(rhs.bits.iter()) {
            u.bits.push(Term::and(b1, b2));
        }
        u
    }
    pub fn not(&self) -> U {
        let mut u = U { bits : ArrayVec::new() };
        for b in self.bits.iter() {
            u.bits.push(Term::not(b));
        }
        u
    }
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

//impl<'a> From<&'a U> for U {
//    fn from(c : &U) -> Self {
//        c.clone()
//    }
//}

impl fmt::Debug for U {
    /// Debug: MSB...LSB
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "U[ "));
        for b in self.bits.iter().rev() {
            try!(b.fmt(f));
        }
        write!(f, " ]")
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
}
