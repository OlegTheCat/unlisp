use im::Vector;
use im::HashMap;
use std::hash::Hasher;
use std::hash::Hash;
use std::fmt;

macro_rules! define_unwrapper {
    ($id:ident ($enum:ident :: $from:ident) -> $to:ty) => {
        pub fn $id(arg: $enum) -> $to {
            match arg {
                $enum::$from(x) => x,
                x => panic!("Cannot convert {} to {}", stringify!(x), stringify!($to))
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EnvFrame {
    pub sym_env: HashMap<Symbol, LispObject>,
    pub fn_env: HashMap<Symbol, Function>
}

impl EnvFrame {
    pub fn new() -> EnvFrame {
        EnvFrame {
            sym_env: HashMap::new(),
            fn_env: HashMap::new()
        }
    }

}

#[derive(Clone)]
pub struct Env {
    pub envs: Vector<EnvFrame>
}

impl Env {
    pub fn new() -> Env {
        let frame = EnvFrame::new();
        let mut envs = Vector::new();
        envs.push_back(frame);
        Env{
            envs: envs
        }
    }

    pub fn push_frame(&self, frame: EnvFrame) -> Env {
        let mut new_env = self.clone();
        new_env.envs.push_front(frame);
        new_env
    }
}

#[derive(Clone)]
pub struct NativeFnWrapper(pub fn(&mut Env, LispObject) -> LispObject);

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct InterpretedFnWrapper {
    pub arglist: Vector<Symbol>,
    pub body: Vector<LispObject>
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Function {
    InterpretedFunction(InterpretedFnWrapper),
    NativeFunction(NativeFnWrapper)
}

define_unwrapper!(to_interpreted_function(Function::InterpretedFunction) -> InterpretedFnWrapper);
define_unwrapper!(to_native_function(Function::NativeFunction) -> NativeFnWrapper);

impl fmt::Debug for NativeFnWrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NativeFn(0x{:x})", self.0 as usize)
    }
}

impl Hash for NativeFnWrapper {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        state.write_usize(self.0 as usize);
    }
}

impl PartialEq for NativeFnWrapper {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 as usize == rhs.0 as usize
    }
}

impl Eq for NativeFnWrapper { }
impl Copy for NativeFnWrapper { }

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

define_unwrapper!(to_symbol(LispObject :: Symbol) -> Symbol);
define_unwrapper!(to_i64(LispObject :: Integer) -> i64);
define_unwrapper!(to_string(LispObject :: String) -> String);
define_unwrapper!(to_vector(LispObject :: Vector) -> Vector<LispObject>);
define_unwrapper!(to_function(LispObject :: Fn) -> Function);

pub fn identity<T>(v: T) -> T { v }
