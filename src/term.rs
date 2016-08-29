
use ::symbol::{Symbol, RSymbol};
use std::fmt;
use std::convert::From;

#[derive(Clone)]
pub enum Term {
    Symbol(RSymbol),
    ConstantInt(i64),
    ConstantDouble(f64),
    Sum(BTerm, BTerm),
    Difference(BTerm, BTerm),
    Product(BTerm, BTerm),
    Quotient(BTerm, BTerm),
}

pub type BTerm = Box<Term>;

impl Term {
    pub fn from_symbol(symbol : &RSymbol) -> Term {
        Term::Symbol(symbol.clone())
    }


    pub fn natural(c : i64) -> Term {
        Term::ConstantInt(c)
    }

    pub fn real(c : f64) -> Term {
        Term::ConstantDouble(c)
    }

    #[inline]
    pub fn boxed(self) -> Box<Self> {
        BTerm::new(self)
    }

    /// a + b
    pub fn add<'a,'b,A,B> (a : &'a A, b : &'b B) -> Term
        where Term: From<&'a A>, Term: From<&'b B> {
        Term::Sum(Term::from(a).boxed(), Term::from(b).boxed())
    }

    /// a - b
    pub fn sub<'a,'b,A,B> (a : &'a A, b : &'b B) -> Term
        where Term: From<&'a A>, Term: From<&'b B> {
        Term::Difference(Term::from(a).boxed(), Term::from(b).boxed())
    }

    /// a * b
    pub fn mul<'a,'b,A,B> (a : &'a A, b : &'b B) -> Term
        where Term: From<&'a A>, Term: From<&'b B> {
        Term::Product(Term::from(a).boxed(), Term::from(b).boxed())
    }

    /// a / b
    pub fn div<'a,'b,A,B> (a : &'a A, b : &'b B) -> Term
        where Term: From<&'a A>, Term: From<&'b B> {
        Term::Quotient(Term::from(a).boxed(), Term::from(b).boxed())
    }

    pub fn set(&self, n :f64) {
        if let &Term::Symbol(ref s) = self {
            s.set(n);
        } else {
            panic!("Called set on non-symbol");
        }
    }

    pub fn evaluate(&self) -> f64 {
        match *self {
            Term::Symbol(ref s) => {
                s.evaluate()
            },
            Term::ConstantInt(c) => {
                c as f64
            },
            Term::ConstantDouble(c) => {
                c
            },
            Term::Sum(ref a, ref b) => {
                a.evaluate() + b.evaluate()
            },
            Term::Difference(ref a, ref b) => {
                a.evaluate() - b.evaluate()
            },
            Term::Product(ref a, ref b) => {
                a.evaluate() * b.evaluate()
            },
            Term::Quotient(ref a, ref b) => {
                a.evaluate() / b.evaluate()
            },
        }
    }
}

impl<'a> From<&'a str> for Term {
    fn from(s : &str) -> Self {
        Self::from_symbol(&Symbol::new(&s))
    }
}

impl From<RSymbol> for Term {
    fn from(s : RSymbol) -> Self {
        Self::from_symbol(&s)
    }
}

impl<'a> From<&'a RSymbol> for Term {
    fn from(s : &RSymbol) -> Self {
        Self::from_symbol(&s)
    }
}

impl From<i64> for Term {
    fn from(c : i64) -> Self {
        Self::natural(c)
    }
}

impl From<f64> for Term {
    fn from(c : f64) -> Self {
        Self::real(c)
    }
}


impl<'a> From<&'a Term> for Term {
    fn from(c : &Term) -> Self {
        c.clone()
    }
}



use std::ops;


macro_rules! op {
    ($ops:ident, $opsfunc:ident, $termmethod:ident) => (

        impl ops::$ops<Term> for Term {
            type Output = Term;
            fn $opsfunc(self, rhs: Term) -> Self::Output {
                Term::$termmethod(&self, &rhs)
            }
        }

        impl<'a> ops::$ops<Term> for &'a Term {
            type Output = Term;
            fn $opsfunc(self, rhs: Term) -> Self::Output {
                Term::$termmethod(self, &rhs)
            }
        }

        impl<'a> ops::$ops<&'a Term> for Term {
            type Output = Term;
            fn $opsfunc(self, rhs: &Term) -> Self::Output {
                Term::$termmethod(&self, rhs)
            }
        }

        impl<'a, 'b> ops::$ops<&'a Term> for &'b Term {
            type Output = Term;
            fn $opsfunc(self, rhs: &Term) -> Self::Output {
                Term::$termmethod(self, rhs)
            }
        }
    )
}

op!(Add, add, add);
op!(Sub, sub, sub);
op!(Div, div, div);
op!(Mul, mul, mul);


impl fmt::Debug for Term {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Term::Symbol(ref s) => {
                write!(f, "{:?}", s)
            },
            Term::ConstantInt(c) => {
                write!(f, "{}", c)
            },
            Term::ConstantDouble(c) => {
                write!(f, "{}", c)
            },
            Term::Sum(ref a, ref b) => {
                write!(f, "({:?} + {:?})", &a, &b)
            },
            Term::Difference(ref a, ref b) => {
                write!(f, "({:?} - {:?})", &a, &b)
            },
            Term::Product(ref a, ref b) => {
                write!(f, "{:?} * {:?}", &a, &b)
            },
            Term::Quotient(ref a, ref b) => {
                write!(f, "{:?} / {:?}", &a, &b)
            },
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let a : Term = "x".into();
        let b : Term = "y".into();
        let p2 = &a + &b * &a / &b - &a;
        //let p2 = &a + &b + &a;
        let p = Term::add(&a, &b);
        println!("{:?}", p);
        println!("{:?}", p2);
    }

    #[test]
    fn eval() {
        let a : Term = "x".into();
        let b : Term = "y".into();
        let p2 = &a + &b * &a / &b - &a;
        a.set(3.);
        b.set(5.);
        println!("{:?} = {}", p2, p2.evaluate());
    }


}
