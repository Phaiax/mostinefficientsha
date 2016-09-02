
use ::term::{RTerm, Term};
use std::convert::From;
use std::fmt;
use arrayvec::ArrayVec;

pub struct U {
    /// The LSB is bits[0], the MSB = bits[31]
    pub bits : ArrayVec<[RTerm ; 32]>,
}

impl U {
    pub fn from_const(mut c : u32) -> U {
        let mut u = U { bits : ArrayVec::new() };
        for _ in 0..32 {
            // print!("{:?} ", c);
            u.bits.push(Term::constant(c & 1u32 == 1u32));
            c = c >> 1;
        }
        u
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "U[ "));
        for b in self.bits.iter() {
            try!(b.fmt(f));
        }
        write!(f, " ]")
    }
}

/*
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

op!(Add, add, add);
op!(Sub, sub, sub);
op!(Div, div, div);
op!(Mul, mul, mul);
*/
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

}
