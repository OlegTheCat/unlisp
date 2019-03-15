use std::rc::Rc;
use std::iter::FromIterator;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;

#[derive(Clone)]
pub struct List<T> {
    head: Link<T>,
    length: usize
}

type Link<T> = Option<Rc<Cons<T>>>;

struct Cons<T> {
    elem: T,
    tail: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None, length: 0 }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn cons(&self, x: T) -> Self {
        List {
            head: Some(Rc::new(Cons {
                elem: x,
                tail: self.head.clone(),
            })),
            length: self.length + 1
        }
    }

    pub fn first(&self) -> Option<&T> {
        self.head.as_ref().map(|cons_rc| &cons_rc.elem)
    }

    pub fn rest(&self) -> Self {
        List {
            head: self.head.as_ref().and_then(|cons_rc| cons_rc.tail.clone()),
            length: if self.is_empty() { 0 } else { self.len() - 1 }
        }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T> {
        ListIterator {
            next: self.head.as_ref().map(|cons_rc| cons_rc.as_ref())
        }
    }
}


impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut head = self.head.take();
        while let Some(cons_rc) = head {
            if let Ok(mut cons) = Rc::try_unwrap(cons_rc) {
                head = cons.tail.take();
            } else {
                break;
            }
        }
    }
}


struct ListIterator<'a, T> {
    next: Option<&'a Cons<T>>
}

impl<'a, T> Iterator for ListIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|cons| {
            self.next = cons.tail.as_ref().map(|cons_rc| cons_rc.as_ref());
            &cons.elem
        })
    }
}

impl<T> FromIterator<T> for List<T> {
    fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Self {
        let mut list = List::new();
        let buf = iter.into_iter().collect::<Vec<_>>();

        for i in buf.into_iter().rev() {
            list = list.cons(i);
        }

        list
    }
}

impl<T: fmt::Debug> fmt::Debug for List<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "( ")?;
        for elem in self.iter() {
            write!(f, "{:?} ", elem)?;
        }
        write!(f, ")")
    }
}

impl<T: Hash> Hash for List<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for elem in self.iter() {
            elem.hash(state);
        }
    }
}

impl<T: PartialEq> PartialEq for List<T> {
    fn eq(&self, rhs: &Self) -> bool {
        if self.len() != rhs.len() {
            return false;
        }

        for (l, r) in self.iter().zip(rhs.iter()) {
            if l != r {
                return false;
            }
        }

        true
    }
}

impl<T: Eq> Eq for List<T> {}
