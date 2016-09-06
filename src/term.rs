//! `term::Term`: A fuzzy bit.

use std::fmt;
use std::cell::Cell;
use std::rc::Rc;
use std::cmp::max;

#[derive(Clone)]
/// A `Term` is either [constant, symbolic or the result of a logical operation
/// of other terms](enum.TermType.html).
///
/// A `Term` can be evaluated to a f64 fuzzy bit that is between 0 and 1.
///
/// During later use, many `Term`s will span some large trees of logic operations.
/// The root `term` of each tree should then be lazily evaluated if needed. The f64
/// evaluation result depends on the type of operation the tree consists of and
/// the leaves, that are either symbolic (0.0 to 1.0) or constant (0.0 or 1.0).
///
/// * The `Term` can be `Constant`. Its value is either `false=0.0` or `true=1.0`. Constants are shortcutted during Term creation. For example the `and`-operation between a `false` constant and another arbitrary `Term` will result in a constant `false` because `false & a == false`.
/// * The `Term` can be `Symbolic`. Its value is a number between 0.0 and 1.0. In contrast to constants, symbols can not be short cut. During later use, you will set all symbols to choosen values and then evaluate the root term.
/// * The `Term` can be a logical operation. Currently implemented are: `Xor ^`, `And &`, `Or |` and `Not !`. Except for `Not`, all Terms take two operands.
///
/// The implementation uses reference counted terms `RTerm = Rc<Term>` to prevent copying large parts of trees and allow caching during evaluations. This makes the `Term` network structure not a tree, but an acyclic directed graph with multiple roots, I guess.
///
/// During those lazy evaluations, every Term caches its value. This caching is not done for successive calls to the root term. It is instead important since some sub trees may be used by thousands of other terms. In other words, the flattened out structure may be very large. (Gigabytes if the logic structure of SHA256 is printed into an ascii file.)
///
/// Use `reset()` to clear the cache.
pub struct Term {
    t : TermType,
    cached_eval : Cell<Option<f64>>,
}

/// Types of Terms.
#[derive(Clone)]
pub enum TermType {
    Symbol(Cell<Option<f64>>),
    Constant(bool),
    Xor(RTerm, RTerm),
    And(RTerm, RTerm),
    Or(RTerm, RTerm),
    Not(RTerm),
}

/// The always used Handle for any `Term`.
pub type RTerm = Rc<Term>;

impl Term {
    /// Create a `Symbol` type term with unassigned value. Assign value with `set()`.
    pub fn symbol() -> RTerm {
        RTerm::new(Term{
            t : TermType::Symbol(Cell::new(None)),
            cached_eval : Cell::new(None),
        })
    }

    /// Shortcut for creating a `true` constant.
    pub fn c1() -> RTerm {
        Term::constant(true)
    }

    /// Shortcut for creating a `false` constant.
    pub fn c0() -> RTerm {
        Term::constant(false)
    }

    /// Create a `Constant` type term with given value.
    pub fn constant(c : bool) -> RTerm {
        RTerm::new(Term{
            t : TermType::Constant(c),
            cached_eval : Cell::new(Some( if c { 1. } else { 0. })),
        })
    }

    /// Checks if this term is of type constant.
    pub fn is_const(&self) -> bool {
        if let TermType::Constant(_) = self.t {
            true
        } else {
            false
        }
    }

    /// Returns the constant value of this `Term`.
    /// Panics if type is not `Constant`.
    pub fn const_val(&self) -> bool {
        if let TermType::Constant(b) = self.t {
            b
        } else {
            panic!("Term is not const.");
        }
    }

    /// Creates a new `RTerm` that lazily evaluates to the xor operation `a ^ b`
    /// of the two input terms.
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

    /// Creates a new `RTerm` that lazily evaluates to the or operation `a | b`
    /// of the two input terms.
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

    /// Creates a new `RTerm` that lazily evaluates to the and operation `a & b`
    /// of the two input terms.
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

    /// Creates a new `RTerm` that lazily evaluates to the not operation `!a`
    /// of the input term.
    pub fn not (a : &RTerm) -> RTerm {
        if a.is_const() {
            return Term::constant(!a.const_val());
        }
        RTerm::new(Term{
            t : TermType::Not(a.clone()),
            cached_eval : Cell::new(None),
        })
    }

    /// Shortcut for (  a^b  ,  a&b  ). The first term is the sum bit of a logic
    /// half adder, the second term is the carry bit.
    pub fn half_add(a : &RTerm, b : &RTerm) -> (RTerm, RTerm) {
        (Term::xor(&a, &b),
         Term::and(&a, &b))
    }

    /// Shortcut for (  a^b^c  ,  (a&b)^(c&(a^b))  ).
    /// The first term is the sum bit of a logic full adder,
    /// the second term is the carry bit.
    pub fn full_add(a : &RTerm, b : &RTerm, carry : &RTerm) -> (RTerm, RTerm) {
        let a_xor_b = &Term::xor(&a, &b);
        // Sum: a xor b xor c
        (Term::xor(&carry, a_xor_b),
        // Carry: (a & b) xor ( c and (a xor b) )
         Term::xor( &Term::and(&a, &b), &Term::and(&carry, &a_xor_b)) )
    }

    /// Sets the value of a `Symbol` type term.
    /// Panics if the type is not `Symbol`.
    pub fn set(&self, n :f64) {
        if let TermType::Symbol(ref s) = self.t {
            s.set(Some(n));
        } else {
            panic!("Called set on non-symbol");
        }
    }

    /// Reset the cached value of this term and all terms this term depends on.
    /// Does not reset anything if this term has already been reset.
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

    /// Evaluate this term by recursively evaluating all sub terms.
    /// Cache the evaluated value.
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

    /// Stack size needed when evaluating this `Term` recursively.
    /// The stack_cnt is the current stack size. (Use 0 at the root term).
    /// It also returns the maximum logic depth. This value is created by
    /// utilizing the eval cache. So you must call `reset()` before and after
    /// using this function.
    ///
    /// Returns: (max_logic_depth, max_stacksize)
    pub fn max_logic_depth_and_max_stack_size(&self, stack_cnt : usize) -> (usize, usize) {
        // Misuse eval cache to save max logic depth.
        // This results in the same cache set behaviour as if
        // evaluate() has been called -> same stack behaviour.
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
                    let (a_max_depth, a_max_stacksize) = a.max_logic_depth_and_max_stack_size(stack_cnt + 1);
                    let (b_max_depth, b_max_stacksize) = b.max_logic_depth_and_max_stack_size(stack_cnt + 1);
                    (max(a_max_depth, b_max_depth), max(a_max_stacksize, b_max_stacksize))
                },
                TermType::And(ref a, ref b) => {
                    let (a_max_depth, a_max_stacksize) = a.max_logic_depth_and_max_stack_size(stack_cnt + 1);
                    let (b_max_depth, b_max_stacksize) = b.max_logic_depth_and_max_stack_size(stack_cnt + 1);
                    (max(a_max_depth, b_max_depth), max(a_max_stacksize, b_max_stacksize))
                },
                TermType::Or(ref b, ref a) => {
                    let (a_max_depth, a_max_stacksize) = a.max_logic_depth_and_max_stack_size(stack_cnt + 1);
                    let (b_max_depth, b_max_stacksize) = b.max_logic_depth_and_max_stack_size(stack_cnt + 1);
                    (max(a_max_depth, b_max_depth), max(a_max_stacksize, b_max_stacksize))
                },
                TermType::Not(ref a) => {
                    a.max_logic_depth_and_max_stack_size(stack_cnt + 1)
                },
            };
            let max_depth = max_depth + 1;
            self.cached_eval.set(Some(max_depth as f64));
            return (max_depth, max_stacksize);
        }
    }

    /// Returns the number of RTerms that contribute to the evaluation of this RTerm.
    /// This function uses the eval cache, so you must call `reset()` before and
    /// after using this function.
    /// To prevent double counting, this function returns the actual number on
    /// the first call and zero on all following calls.
    ///
    /// Returns: number_of_uncounted_rterms_including_this_rterm
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

    /// Returns the number of Terms that contribute to the evaluation if the
    /// tree would have been flattened.
    pub fn nr_of_terms_flattened(&self) -> usize {
        // misuse eval cache to prevent double counting
        if self.cached_eval.get().is_some() {
            return self.cached_eval.get().unwrap() as usize;
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
