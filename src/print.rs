use crate::cons::List;
use crate::core;
use crate::core::LispObject;
use std::fmt;

fn write_list(f: &mut fmt::Formatter, list: &List<LispObject>) -> Result<(), fmt::Error> {
    let mut first = true;

    write!(f, "(")?;

    for form in list.iter() {
        if !first {
            write!(f, " ")?;
        }
        write!(f, "{}", form)?;
        first = false;
    }
    write!(f, ")")
}

impl fmt::Display for core::Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self.body {
            core::FunctionBody::Native(_) => write!(f, "#<NATIVE-FN>"),
            core::FunctionBody::Interpreted(_) => write!(f, "#<INTERPRETED-FN>"),
        }
    }
}

impl fmt::Display for core::Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.name())
    }
}

impl fmt::Display for core::LispObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            LispObject::List(list) if list.is_empty() => write!(f, "nil"),
            LispObject::T => write!(f, "t"),
            LispObject::Integer(i) => write!(f, "{}", i),
            LispObject::String(s) => write!(f, "\"{}\"", s),
            LispObject::Fn(func) => write!(f, "{}", func),
            LispObject::Symbol(s) => write!(f, "{}", s),
            LispObject::List(list) => write_list(f, list),
        }
    }
}
