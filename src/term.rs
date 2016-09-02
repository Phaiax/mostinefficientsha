
use std::fmt;
use std::cell::Cell;
use std::rc::{Rc};

#[derive(Clone)]
pub struct Term {
    t : TermType,
    cached_eval : Cell<Option<f64>>,
}

#[derive(Clone)]
pub enum TermType {
    Symbol(Cell<Option<f64>>),
    Constant(bool),
    Xor(RTerm, RTerm),
    And(RTerm, RTerm),
    Or(RTerm, RTerm),
    Not(RTerm),
}

pub type RTerm = Rc<Term>;

impl Term {
    pub fn symbol() -> RTerm {
        RTerm::new(Term{
            t : TermType::Symbol(Cell::new(None)),
            cached_eval : Cell::new(None),
        })
    }

    pub fn constant(c : bool) -> RTerm {
        RTerm::new(Term{
            t : TermType::Constant(c),
            cached_eval : Cell::new(Some( if c { 1. } else { 0. })),
        })
    }

    pub fn xor (a : &RTerm, b : &RTerm) -> RTerm {
        RTerm::new(Term{
            t : TermType::Xor(a.clone(), b.clone()),
            cached_eval : Cell::new(None),
        })
    }

    pub fn or (a : &RTerm, b : &RTerm) -> RTerm {
        RTerm::new(Term{
            t : TermType::Or(a.clone(), b.clone()),
            cached_eval : Cell::new(None),
        })
    }

    /// a + b
    pub fn not (a : &RTerm) -> RTerm {
        RTerm::new(Term{
            t : TermType::Not(a.clone()),
            cached_eval : Cell::new(None),
        })
    }

    pub fn set(&self, n :f64) {
        if let TermType::Symbol(ref s) = self.t {
            s.set(Some(n));
        } else {
            panic!("Called set on non-symbol");
        }
    }

    /// reset cache (recursive)
    pub fn reset(&self) {
        if self.cached_eval.get().is_some() {
            match self.t {
                TermType::Symbol(_) => {
                    self.cached_eval.set(None);
                },
                TermType::Constant(_) => {},
                TermType::Xor(ref a, ref b) => {
                    self.cached_eval.set(None);
                    a.reset();
                    b.reset();
                },
                TermType::And(ref a, ref b) => {
                    self.cached_eval.set(None);
                    a.reset();
                    b.reset();
                },
                TermType::Or(ref b, ref a) => {
                    self.cached_eval.set(None);
                    a.reset();
                    b.reset();
                },
                TermType::Not(ref a) => {
                    self.cached_eval.set(None);
                    a.reset();
                },
            }
        }
    }

    pub fn evaluate(&self) -> f64 {
        if let Some(c) = self.cached_eval.get() {
            return c;
        } else {
            let v = match self.t {
                TermType::Symbol(ref s) => {
                    s.get().expect("Symbol not set. Eval failed.")
                },
                TermType::Constant(c) => {
                    if c { 1. } else { 0. }
                },
                TermType::Xor(ref a, ref b) => {
                    a.evaluate() * b.evaluate()
                },
                TermType::And(ref a, ref b) => {
                    println!("NOT IMPLEMENTED");
                    a.evaluate() * b.evaluate()
                },
                TermType::Or(ref b, ref a) => {
                    println!("NOT IMPLEMENTED");
                    a.evaluate() * b.evaluate()
                },
                TermType::Not(ref a) => {
                    1. - a.evaluate()
                },
            };
            self.cached_eval.set(Some(v));
            return v;
        }
    }
}



impl fmt::Debug for Term {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.t {
            TermType::Symbol(_) => {
                write!(f, ":")
            },
            TermType::Constant(c) => {
                write!(f, "{}", if c { 1 } else { 0 })
            },
            TermType::Xor(ref a, ref b) => {
                write!(f, "({:?} Xor {:?})", &a, &b)
            },
            TermType::And(ref a, ref b) => {
                write!(f, "({:?} & {:?})", &a, &b)
            },
            TermType::Or(ref a, ref b) => {
                write!(f, "{:?} | {:?}", &a, &b)
            },
            TermType::Not(ref a) => {
                write!(f, "! {:?}", &a)
            },
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {

    }

}
