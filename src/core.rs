use error;
use im::HashMap;
use cons::List;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::cell::RefCell;
use std::rc::Rc;

macro_rules! define_unwrapper {
    ($id:ident ($enum:ident :: $from:ident) -> $to:ty) => {
        #[allow(unused)]
        pub fn $id(arg: &$enum) -> Result<&$to, error::CastError> {
            match arg {
                $enum::$from(x) => Ok(x),
                x => Err(error::CastError::new(
                    format!("{}", x).to_string(),
                    stringify!($to).to_string(),
                )),
            }
        }
    };
}

macro_rules! define_unwrapper_owned {
    ($id:ident ($enum:ident :: $from:ident) -> $to:ty) => {
        #[allow(unused)]
        pub fn $id(arg: $enum) -> Result<$to, error::CastError> {
            match arg {
                $enum::$from(x) => Ok(x),
                x => Err(error::CastError::new(
                    format!("{}", x).to_string(),
                    stringify!($to).to_string(),
                )),
            }
        }
    };
}


#[derive(Debug, Clone)]
pub struct EnvFrame {
    pub sym_env: HashMap<Symbol, LispObject>,
    pub fn_env: HashMap<Symbol, Function>,
    pub macro_env: HashMap<Symbol, Function>,
}

impl EnvFrame {
    pub fn new() -> EnvFrame {
        EnvFrame {
            sym_env: HashMap::new(),
            fn_env: HashMap::new(),
            macro_env: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GlobalEnvFrame {
    pub sym_env: HashMap<Symbol, LispObject>,
    pub fn_env: HashMap<Symbol, Function>,
    pub macro_env: HashMap<Symbol, Function>,
    pub special_env: HashMap<Symbol, NativeFnWrapper>,
}

impl GlobalEnvFrame {
    pub fn new() -> GlobalEnvFrame {
        GlobalEnvFrame {
            sym_env: HashMap::new(),
            fn_env: HashMap::new(),
            special_env: HashMap::new(),
            macro_env: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Env {
    pub global_env: Rc<RefCell<GlobalEnvFrame>>,
    pub cur_env: EnvFrame
}

impl Env {
    pub fn new() -> Env {
        Env {
            global_env: Rc::new(RefCell::new(GlobalEnvFrame::new())),
            cur_env: EnvFrame::new()
        }
    }
}

#[derive(Clone)]
pub struct NativeFnWrapper(pub fn(Env, List<LispObject>) -> error::GenResult<LispObject>);

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct InterpretedFn {
    pub restarg: Option<Symbol>,
    pub arglist: List<Symbol>,
    pub body: List<LispObject>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Function {
    InterpretedFunction(InterpretedFn),
    NativeFunction(NativeFnWrapper),
}

define_unwrapper!(to_interpreted_function(Function::InterpretedFunction) -> InterpretedFn);
define_unwrapper!(to_native_function(Function::NativeFunction) -> NativeFnWrapper);

impl fmt::Debug for NativeFnWrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NativeFn(0x{:x})", self.0 as usize)
    }
}

impl Hash for NativeFnWrapper {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write_usize(self.0 as usize);
    }
}

impl PartialEq for NativeFnWrapper {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 as usize == rhs.0 as usize
    }
}

impl Eq for NativeFnWrapper {}
impl Copy for NativeFnWrapper {}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Symbol(Rc<String>);

impl Symbol {
    pub fn new(s: impl Into<String>) -> Self {
        Symbol(Rc::new(s.into()))
    }

    pub fn name(&self) -> String {
        self.0.as_ref().clone()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum LispObject {
    T,
    Symbol(Symbol),
    Integer(i64),
    String(String),
    List(List<LispObject>),
    Fn(Function)
}

impl LispObject {
    pub fn nil() -> Self {
        LispObject::List(List::empty())
    }
}

define_unwrapper!(to_symbol(LispObject :: Symbol) -> Symbol);
define_unwrapper!(to_i64(LispObject :: Integer) -> i64);
define_unwrapper!(to_string(LispObject :: String) -> String);
define_unwrapper!(to_list(LispObject :: List) -> List<LispObject>);
define_unwrapper!(to_function(LispObject :: Fn) -> Function);

define_unwrapper_owned!(to_symbol_owned(LispObject :: Symbol) -> Symbol);
define_unwrapper_owned!(to_i64_owned(LispObject :: Integer) -> i64);
define_unwrapper_owned!(to_string_owned(LispObject :: String) -> String);
define_unwrapper_owned!(to_list_owned(LispObject :: List) -> List<LispObject>);
define_unwrapper_owned!(to_function_owned(LispObject :: Fn) -> Function);
