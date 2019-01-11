use core;
use core::LispObject;
use core::Symbol;
use im::Vector;

fn prn_list(vec: &Vector<LispObject>) {
    let mut first = true;

    print!("(");

    for form in vec {
        if !first {
            print!(" ");
        }
        prn(form);
        first = false;
    }
    print!(")");
}

pub fn prn(form: &LispObject) {
    match form {
        LispObject::Nil => print!("NIL"),
        LispObject::T => print!("T"),
        LispObject::Symbol(Symbol(s)) => print!("{}", s),
        LispObject::Integer(i) => print!("{}", i),
        LispObject::String(s) => print!("\"{}\"", s),
        LispObject::Vector(vec) => prn_list(vec),
        LispObject::Fn(core::Function::NativeFunction(_)) =>
            print!("#<NATIVE-FN>"),
        LispObject::Fn(core::Function::InterpretedFunction{..}) =>
            print!("#<INTERPRETED-FN>")

    }
}
