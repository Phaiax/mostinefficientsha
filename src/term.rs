
use std::fmt;
use std::cell::Cell;
use std::rc::{Rc};
use std::cmp::max;

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

    pub fn c1() -> RTerm {
        Term::constant(true)
    }

    pub fn c0() -> RTerm {
        Term::constant(false)
    }

    pub fn constant(c : bool) -> RTerm {
        RTerm::new(Term{
            t : TermType::Constant(c),
            cached_eval : Cell::new(Some( if c { 1. } else { 0. })),
        })
    }

    pub fn is_const(&self) -> bool {
        if let TermType::Constant(_) = self.t {
            true
        } else {
            false
        }
    }

    pub fn const_val(&self) -> bool {
        if let TermType::Constant(b) = self.t {
            b
        } else {
            panic!("Term is not const.");
        }
    }

    pub fn xor (a : &RTerm, b : &RTerm) -> RTerm {
        if a.is_const() && b.is_const() {
            return Term::constant(a.const_val() ^ b.const_val());
        } else if a.is_const() && a.const_val() == true {
            return Term::not(&b);
        } else if a.is_const() && a.const_val() == false {
            return b.clone();
        } else if b.is_const() && b.const_val() == true {
            return Term::not(&a);
        } else if b.is_const() && b.const_val() == false {
            return a.clone();
        }
        // no consts
        RTerm::new(Term{
            t : TermType::Xor(a.clone(), b.clone()),
            cached_eval : Cell::new(None),
        })
    }

    pub fn or (a : &RTerm, b : &RTerm) -> RTerm {
        if a.is_const() && b.is_const() {
            return Term::constant(a.const_val() | b.const_val());
        } else if a.is_const() && a.const_val() == true {
            return Term::c1(); // 1 | b => 1
        } else if a.is_const() && a.const_val() == false {
            return b.clone();  // 0 | b => b
        } else if b.is_const() && b.const_val() == true {
            return Term::c1(); // a | 1 => 1
        } else if b.is_const() && b.const_val() == false {
            return a.clone();  // a | 0 => a
        }
        // no consts
        RTerm::new(Term{
            t : TermType::Or(a.clone(), b.clone()),
            cached_eval : Cell::new(None),
        })
    }

    pub fn and (a : &RTerm, b : &RTerm) -> RTerm {
        if a.is_const() && b.is_const() {
            return Term::constant(a.const_val() & b.const_val());
        } else if a.is_const() && a.const_val() == true {
            return b.clone(); // 1 & b => b
        } else if a.is_const() && a.const_val() == false {
            return Term::c0();  // 0 & b => 0
        } else if b.is_const() && b.const_val() == true {
            return a.clone(); // a & 1 => a
        } else if b.is_const() && b.const_val() == false {
            return Term::c0();  // a & 0 => 0
        }
        RTerm::new(Term{
            t : TermType::And(a.clone(), b.clone()),
            cached_eval : Cell::new(None),
        })
    }

    pub fn not (a : &RTerm) -> RTerm {
        if a.is_const() {
            return Term::constant(!a.const_val());
        }
        RTerm::new(Term{
            t : TermType::Not(a.clone()),
            cached_eval : Cell::new(None),
        })
    }

    pub fn half_add(a : &RTerm, b : &RTerm) -> (RTerm, RTerm) {
        (Term::xor(&a, &b),
         Term::and(&a, &b))
    }

    pub fn full_add(a : &RTerm, b : &RTerm, carry : &RTerm) -> (RTerm, RTerm) {
        let a_xor_b = &Term::xor(&a, &b);
        // Sum: a xor b xor c
        (Term::xor(&carry, a_xor_b),
        // Carry: (a & b) xor ( c and (a xor b) )
         Term::xor( &Term::and(&a, &b), &Term::and(&carry, &a_xor_b)) )
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
                    (a.evaluate() - b.evaluate()).abs()
                },
                TermType::And(ref a, ref b) => {
                    a.evaluate() * b.evaluate()
                },
                TermType::Or(ref b, ref a) => {
                     (a.evaluate().sqrt() + b.evaluate().sqrt()).min(1.)
                },
                TermType::Not(ref a) => {
                    1. - a.evaluate()
                },
            };
            self.cached_eval.set(Some(v));
            return v;
        }
    }

    /// Stack size needed to fit the recursive evaluate calls
    /// Returns: (max_logic_depth, max_stacksize)
    pub fn max_stack_size(&self, stack_cnt : usize) -> (usize, usize) {
        // misuse eval cache to save max logic depth
        // this results in the same cache set behaviour like if
        // evaluate() has been called -> same stack behaviour
        if let Some(d) = self.cached_eval.get() {
            return (d as usize, stack_cnt);
        } else {
            let (max_depth, max_stacksize) = match self.t {
                TermType::Symbol(_) => {
                    (0, stack_cnt)
                },
                TermType::Constant(_) => {
                    (0, stack_cnt)
                },
                TermType::Xor(ref a, ref b) => {
                    let (a_max_depth, a_max_stacksize) = a.max_stack_size(stack_cnt + 1);
                    let (b_max_depth, b_max_stacksize) = b.max_stack_size(stack_cnt + 1);
                    (max(a_max_depth, b_max_depth), max(a_max_stacksize, b_max_stacksize))
                },
                TermType::And(ref a, ref b) => {
                    let (a_max_depth, a_max_stacksize) = a.max_stack_size(stack_cnt + 1);
                    let (b_max_depth, b_max_stacksize) = b.max_stack_size(stack_cnt + 1);
                    (max(a_max_depth, b_max_depth), max(a_max_stacksize, b_max_stacksize))
                },
                TermType::Or(ref b, ref a) => {
                    let (a_max_depth, a_max_stacksize) = a.max_stack_size(stack_cnt + 1);
                    let (b_max_depth, b_max_stacksize) = b.max_stack_size(stack_cnt + 1);
                    (max(a_max_depth, b_max_depth), max(a_max_stacksize, b_max_stacksize))
                },
                TermType::Not(ref a) => {
                    a.max_stack_size(stack_cnt + 1)
                },
            };
            let max_depth = max_depth + 1;
            self.cached_eval.set(Some(max_depth as f64));
            return (max_depth, max_stacksize);
        }
    }

    /// Number of RTerms
    /// Returns: (number_of_uncounted_rterms_including_)
    pub fn nr_of_terms(&self) -> usize {
        // misuse eval cache to prevent double counting
        if self.cached_eval.get().is_some() {
            return 0;
        } else {
            let nr = 1 + match self.t {
                TermType::Symbol(_) => {
                    0
                },
                TermType::Constant(_) => {
                    0
                },
                TermType::Xor(ref a, ref b) => {
                    a.nr_of_terms() + b.nr_of_terms()
                },
                TermType::And(ref a, ref b) => {
                    a.nr_of_terms() + b.nr_of_terms()
                },
                TermType::Or(ref b, ref a) => {
                    a.nr_of_terms() + b.nr_of_terms()
                },
                TermType::Not(ref a) => {
                    a.nr_of_terms()
                },
            };
            self.cached_eval.set(Some(nr as f64));
            return nr;
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

    //xor or and not half_add full_add

    #[test]
    fn xor() {
        assert_eq!(Term::xor(&Term::c0(), &Term::c0()).evaluate(), 0.);
        assert_eq!(Term::xor(&Term::c1(), &Term::c0()).evaluate(), 1.);
        assert_eq!(Term::xor(&Term::c0(), &Term::c1()).evaluate(), 1.);
        assert_eq!(Term::xor(&Term::c1(), &Term::c1()).evaluate(), 0.);
    }

    #[test]
    fn or() {
        assert_eq!(Term::or(&Term::c0(), &Term::c0()).evaluate(), 0.);
        assert_eq!(Term::or(&Term::c1(), &Term::c0()).evaluate(), 1.);
        assert_eq!(Term::or(&Term::c0(), &Term::c1()).evaluate(), 1.);
        assert_eq!(Term::or(&Term::c1(), &Term::c1()).evaluate(), 1.);
    }

    #[test]
    fn and() {
        assert_eq!(Term::and(&Term::c0(), &Term::c0()).evaluate(), 0.);
        assert_eq!(Term::and(&Term::c1(), &Term::c0()).evaluate(), 0.);
        assert_eq!(Term::and(&Term::c0(), &Term::c1()).evaluate(), 0.);
        assert_eq!(Term::and(&Term::c1(), &Term::c1()).evaluate(), 1.);
    }

    #[test]
    fn not() {
        assert_eq!(Term::not(&Term::c1()).evaluate(), 0.);
        assert_eq!(Term::not(&Term::c0()).evaluate(), 1.);
    }


    #[test]
    fn half_add() {
        let (sum, carry) = Term::half_add(&Term::c0(), &Term::c0());
        assert_eq!(sum.evaluate(), 0.);
        assert_eq!(carry.evaluate(), 0.);
        let (sum, carry) = Term::half_add(&Term::c1(), &Term::c0());
        assert_eq!(sum.evaluate(), 1.);
        assert_eq!(carry.evaluate(), 0.);
        let (sum, carry) = Term::half_add(&Term::c0(), &Term::c1());
        assert_eq!(sum.evaluate(), 1.);
        assert_eq!(carry.evaluate(), 0.);
        let (sum, carry) = Term::half_add(&Term::c1(), &Term::c1());
        assert_eq!(sum.evaluate(), 0.);
        assert_eq!(carry.evaluate(), 1.);
    }

    #[test]
    fn full_add() {
        let (sum, carry) = Term::full_add(&Term::c0(), &Term::c0(), &Term::c0());
        assert_eq!(sum.evaluate(), 0.);
        assert_eq!(carry.evaluate(), 0.);
        let (sum, carry) = Term::full_add(&Term::c1(), &Term::c0(), &Term::c0());
        assert_eq!(sum.evaluate(), 1.);
        assert_eq!(carry.evaluate(), 0.);
        let (sum, carry) = Term::full_add(&Term::c0(), &Term::c1(), &Term::c0());
        assert_eq!(sum.evaluate(), 1.);
        assert_eq!(carry.evaluate(), 0.);
        let (sum, carry) = Term::full_add(&Term::c1(), &Term::c1(), &Term::c0());
        assert_eq!(sum.evaluate(), 0.);
        assert_eq!(carry.evaluate(), 1.);

        // carry in = 1
        let (sum, carry) = Term::full_add(&Term::c0(), &Term::c0(), &Term::c1());
        assert_eq!(sum.evaluate(), 1.);
        assert_eq!(carry.evaluate(), 0.);
        let (sum, carry) = Term::full_add(&Term::c1(), &Term::c0(), &Term::c1());
        assert_eq!(sum.evaluate(), 0.);
        assert_eq!(carry.evaluate(), 1.);
        let (sum, carry) = Term::full_add(&Term::c0(), &Term::c1(), &Term::c1());
        assert_eq!(sum.evaluate(), 0.);
        assert_eq!(carry.evaluate(), 1.);
        let (sum, carry) = Term::full_add(&Term::c1(), &Term::c1(), &Term::c1());
        assert_eq!(sum.evaluate(), 1.);
        assert_eq!(carry.evaluate(), 1.);
    }

}
