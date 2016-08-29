
use std::cell::RefCell;
use std::rc::{Weak, Rc};
use std::fmt;

pub struct Symbol {
    pub name : String,
    selfref : RefCell<Option<Weak<Symbol>>>,
}

pub type RSymbol = Rc<Symbol>;

impl Symbol {
    pub fn new(name : &str) -> RSymbol {
        let s = RSymbol::new(Symbol {
            name : name.into(),
            selfref : RefCell::default(),
        });
        *s.selfref.borrow_mut() = Some(Rc::downgrade(&s));
        s
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let _s = Symbol::new("x");

        println!("{:?}", _s);
    }
}
