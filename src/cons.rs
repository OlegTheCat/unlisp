use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::iter::FromIterator;
use std::rc::Rc;

pub struct List<T> {
    head: Link<T>,
    length: usize,
}

type Link<T> = Option<Rc<Cons<T>>>;

struct Cons<T> {
    elem: Rc<T>,
    tail: Link<T>,
}

impl<T> List<T> {
    pub fn empty() -> Self {
        List {
            head: None,
            length: 0,
        }
    }

    pub fn from_iter<I, II>(iter: II) -> Self
    where I: Iterator<Item = T>,
          II: IntoIterator<Item = T, IntoIter = I>
    {
        let buf: Vec<_> = iter.into_iter().collect();

        Self::from_rev_iter(buf)
    }

    pub fn from_rev_iter<I, II>(iter: II) -> Self
    where
        I: Iterator<Item = T> + DoubleEndedIterator,
        II: IntoIterator<Item = T, IntoIter = I>,
    {
        let mut list = List::empty();
        for i in iter.into_iter().rev() {
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
            length: self.length + 1,
        }
    }

    pub fn cons(&self, x: T) -> Self {
        self.cons_rc(Rc::new(x))
    }

    pub fn first(&self) -> Option<&T> {
        self.first_rc().map(|rc| rc.as_ref())
    }

    pub fn first_rc(&self) -> Option<&Rc<T>> {
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
                length: self.len() - n,
            }
        }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T> {
        ListIterator {
            next: self.head.as_ref().map(|cons_rc| cons_rc.as_ref()),
        }
    }

    pub fn rc_iter(&self) -> impl Iterator<Item = Rc<T>> {
        LinkIterator {
            next: self.head.clone(),
        }
    }
}

impl<T> Clone for List<T> {
    fn clone(&self) -> Self {
        List {
            head: self.head.clone(),
            length: self.len(),
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
    next: Link<T>,
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
    next: Option<&'a Cons<T>>,
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
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
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
