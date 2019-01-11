use im::Vector;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Function {
    pub arglist: Vector<Symbol>,
    pub body: Vector<LispObject>
}


#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Symbol(pub String);

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum LispObject {
    Nil,
    T,
    Symbol(Symbol),
    Integer(i64),
    String(String),
    Vector(Vector<LispObject>),
    Fn(Function)
}

macro_rules! define_unwrapper {
    ($id:ident, $from:ident, $to:ty) => {
        pub fn $id(arg: LispObject) -> $to {
            match arg {
                LispObject::$from(x) => x,
                x => panic!("Cannot convert {:?} to {}", x, stringify!($to))
            }
        }
    }
}

define_unwrapper!(to_symbol, Symbol, Symbol);
define_unwrapper!(to_i64, Integer, i64);
define_unwrapper!(to_string, String, String);
define_unwrapper!(to_vector, Vector, Vector<LispObject>);
define_unwrapper!(to_function, Fn, Function);
