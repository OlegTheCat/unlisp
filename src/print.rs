use reader::LispObject;
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
        LispObject::Symbol(s) => print!("{}", s),
        LispObject::IntegerLiteral(i) => print!("{}", i),
        LispObject::StringLiteral(s) => print!("\"{}\"", s),
        LispObject::List(vec) => prn_list(vec)
    }
}
