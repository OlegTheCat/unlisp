use crate::env::StackFrameDesignator;
use crate::env::StackTrace;
use crate::object;
use crate::object::LispObject;
use std::fmt;

impl fmt::Display for object::Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self.body {
            object::FunctionBody::Native(_) => write!(f, "#<NATIVE-FN>"),
            object::FunctionBody::Interpreted(_) => write!(f, "#<INTERPRETED-FN>"),
        }
    }
}

impl fmt::Display for object::Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.name())
    }
}

impl fmt::Display for object::LispObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            LispObject::List(list) if list.is_empty() => write!(f, "nil"),
            LispObject::T => write!(f, "t"),
            LispObject::Integer(i) => write!(f, "{}", i),
            LispObject::String(s) => write!(f, "\"{}\"", s),
            LispObject::Fn(func) => write!(f, "{}", func),
            LispObject::Symbol(s) => write!(f, "{}", s),
            LispObject::List(list) => write!(f, "{}", list),
        }
    }
}

impl fmt::Display for object::FunctionSignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "lambda/{}/{}{}",
            self.name
                .as_ref()
                .map_or_else(|| "<anon>".to_string(), |s| s.name()),
            self.arglist.len(),
            self.restarg.as_ref().map_or("", |_| "+")
        )
    }
}

pub fn print_stack_trace(trace: Option<&StackTrace>) {
    println!("stack trace:");
    match trace {
        Some(trace) => {
            for designator in trace.iter() {
                match designator {
                    StackFrameDesignator::Top => println!("  <top>"),
                    StackFrameDesignator::Name(sym) => println!("  {}", sym),
                    StackFrameDesignator::Signature(sig) => println!("  {}", sig),
                }
            }
        }
        None => println!("  unavailable"),
    }
}
