use std::rc::Rc;
use std::iter::FromIterator;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;

pub struct List<T> {
    head: Link<T>,
    length: usize
}

type Link<T> = Option<Rc<Cons<T>>>;

struct Cons<T> {
    elem: Rc<T>,
    tail: Link<T>,
}

impl<T> List<T> {
    pub fn empty() -> Self {
        List { head: None, length: 0 }
    }

    pub fn from_iter<I: Iterator<Item = T>>(iter: I) -> Self {
        let mut list = List::empty();
        let buf = iter.into_iter().collect::<Vec<_>>();

        for i in buf.into_iter().rev() {
            list = list.cons(i);
        }

        list
    }

    pub fn from_rev_iter<I: Iterator<Item = T> + DoubleEndedIterator>(iter: I) -> Self {
        let mut list = List::empty();
        for i in iter.rev() {
            list = list.cons(i);
        }

        list

    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn cons_rc(&self, x: Rc<T>) -> Self {
        List {
            head: Some(Rc::new(Cons {
                elem: x,
                tail: self.head.clone(),
            })),
            length: self.length + 1
        }
    }

    pub fn cons(&self, x: T) -> Self {
        self.cons_rc(Rc::new(x))
    }

    pub fn first(&self) -> Option<&T> {
        self.first_rc().map(|rc| rc.as_ref())
    }

    fn first_rc(&self) -> Option<&Rc<T>> {
        self.head.as_ref().map(|cons_rc| &cons_rc.elem)
    }

    pub fn ufirst(&self) -> &T {
        self.first().unwrap()
    }

    pub fn tail(&self) -> Self {
        self.tailn(1)
    }

    pub fn tailn(&self, n: usize) -> Self {
        if n >= self.len() {
            Self::empty()
        } else {
            let mut i = n;
            let mut link = self.head.as_ref().unwrap();

            while i != 0 {
                link = link.tail.as_ref().unwrap();
                i -= 1;
            }

            Self {
                head: Some(link.clone()),
                length: self.len() - n
            }
        }
    }

    fn append_links(link1: Link<T>, link2: Link<T>) -> Link<T> {
        match link1 {
            Some(cons_rc) => {
                Some(Rc::new(Cons {
                    elem: cons_rc.elem.clone(),
                    tail: Self::append_links(cons_rc.tail.clone(), link2)
                }))
            }
            None => link2
        }
    }

    pub fn append(&self, other: Self) -> Self {
        Self {
            head: Self::append_links(self.head.clone(), other.head.clone()),
            length: self.len() + other.len()
        }
    }

    pub fn split_at(&self, idx: usize) -> (Self, Self) {
        let empty = Self::empty();
        if idx >= self.len() {
            (self.clone(), empty)
        } else {
            let mut lhs = vec![];
            let mut rhs = self.clone();
            let mut i = idx;

            while i != 0 {
                lhs.push(rhs.first_rc().unwrap().clone());
                rhs = rhs.tail();

                i -= 1;
            }

            let mut final_lhs = empty;

            for x in lhs.into_iter().rev() {
                final_lhs = final_lhs.cons_rc(x);
            }

            (final_lhs, rhs)
        }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T> {
        ListIterator {
            next: self.head.as_ref().map(|cons_rc| cons_rc.as_ref())
        }
    }

    pub fn rc_iter(&self) -> impl Iterator<Item = Rc<T>> {
        LinkIterator {
            next: self.head.clone()
        }
    }
}

impl<T> Clone for List<T> {
    fn clone(&self) -> Self {
        List {
            head: self.head.clone(),
            length: self.len()
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

struct LinkIterator<T> {
    next: Link<T>
}

impl<T> Iterator for LinkIterator<T> {
    type Item = Rc<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_none() {
            None
        } else {
            let cons_rc = self.next.take().unwrap();
            self.next = cons_rc.tail.clone();
            Some(cons_rc.elem.clone())
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
            cons.elem.as_ref()
        })
    }
}

impl<T> FromIterator<T> for List<T> {
    fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Self {
        Self::from_iter(iter.into_iter())
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
