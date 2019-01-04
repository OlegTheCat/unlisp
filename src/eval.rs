use im::Vector;

use reader::LispObject;

pub fn quote(form: LispObject) -> Option<LispObject> {
    if let LispObject::List(vec) = form {
        if vec.head().is_some() && *vec.head().unwrap() == LispObject::Symbol("quote".to_string()) {
            return Some(vec.skip(1).into_iter().next().unwrap());
        }
    }

    None
}

pub fn eval(form: LispObject) -> LispObject {
    LispObject::Nil
}
