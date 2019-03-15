use core;
use core::LispObject;
use cons::List;
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
        match self {
            core::Function::NativeFunction(_) => write!(f, "#<NATIVE-FN"),
            core::Function::InterpretedFunction(_) => write!(f, "#<INTERPRETED-FN>"),
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
            LispObject::List(list) if list.is_empty() => write!(f, "NIL"),
            LispObject::T => write!(f, "T"),
            LispObject::Integer(i) => write!(f, "{}", i),
            LispObject::String(s) => write!(f, "\"{}\"", s),
            LispObject::Fn(func) => write!(f, "{}", func),
            LispObject::Macro(func) => write!(f, "{}+MACRO", func),
            LispObject::Special(_) => Err(fmt::Error),
            LispObject::Symbol(s) => write!(f, "{}", s),
            LispObject::List(list) => write_list(f, list),
        }
    }
}
