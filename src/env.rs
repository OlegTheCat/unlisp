use crate::object::*;
use im::HashMap;
use std::cell::RefCell;
use std::ops::Deref;
use std::ops::DerefMut;
use std::rc::Rc;

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
    pub cur_env: EnvFrame,
}

impl Env {
    pub fn new() -> Env {
        Env {
            global_env: Rc::new(RefCell::new(GlobalEnvFrame::new())),
            cur_env: EnvFrame::new(),
        }
    }

    pub fn global_env_mut<'a>(&'a self) -> impl DerefMut<Target = GlobalEnvFrame> + 'a {
        self.global_env.as_ref().borrow_mut()
    }

    pub fn global_env<'a>(&'a self) -> impl Deref<Target = GlobalEnvFrame> + 'a {
        self.global_env.as_ref().borrow()
    }
}
