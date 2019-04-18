use crate::cons::List;
use crate::object::*;
use crate::error::ErrorWithStackTrace;
use im::HashMap;
use std::cell::RefCell;
use std::ops::Deref;
use std::ops::DerefMut;
use std::rc::Rc;
use std::error::Error;

#[derive(Debug, Clone)]
pub enum StackFrameDesignator {
    Signature(FunctionSignature),
    Name(Symbol),
    Top,
}

pub type StackTrace = List<StackFrameDesignator>;

#[derive(Debug, Clone)]
struct LocalEnv {
    sym_env: HashMap<Symbol, LispObject>,
    fn_env: HashMap<Symbol, Function>,
    macro_env: HashMap<Symbol, Function>,
    stack_trace: StackTrace,
}

impl LocalEnv {
    fn new() -> LocalEnv {
        LocalEnv {
            sym_env: HashMap::new(),
            fn_env: HashMap::new(),
            macro_env: HashMap::new(),
            stack_trace: List::empty().cons(StackFrameDesignator::Top),
        }
    }
}

#[derive(Debug, Clone)]
struct GlobalEnv {
    sym_env: HashMap<Symbol, LispObject>,
    fn_env: HashMap<Symbol, Function>,
    macro_env: HashMap<Symbol, Function>,
    special_env: HashMap<Symbol, NativeFnWrapper>,
}

impl GlobalEnv {
    fn new() -> GlobalEnv {
        GlobalEnv {
            sym_env: HashMap::new(),
            fn_env: HashMap::new(),
            special_env: HashMap::new(),
            macro_env: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Env {
    global_env: Rc<RefCell<GlobalEnv>>,
    local_env: LocalEnv,
}

macro_rules! lookup_symbol {
    ($env:ident, $lookup_env:ident, $sym:expr) => {{
        let global = $env.global_env();
        $env.local_env
            .$lookup_env
            .get($sym)
            .or_else(|| global.$lookup_env.get($sym))
            .cloned()
    }};
}

impl Env {
    pub fn new() -> Self {
        Self {
            global_env: Rc::new(RefCell::new(GlobalEnv::new())),
            local_env: LocalEnv::new(),
        }
    }

    pub fn clone_with_global(&self) -> Self {
        Self {
            global_env: Rc::new(RefCell::new(self.global_env().clone())),
            local_env: self.local_env.clone(),
        }
    }

    fn global_env_mut<'a>(&'a self) -> impl DerefMut<Target = GlobalEnv> + 'a {
        self.global_env.as_ref().borrow_mut()
    }

    fn global_env<'a>(&'a self) -> impl Deref<Target = GlobalEnv> + 'a {
        self.global_env.as_ref().borrow()
    }

    pub fn lookup_symbol_special(&self, s: &Symbol) -> Option<NativeFnWrapper> {
        self.global_env().special_env.get(s).cloned()
    }

    pub fn lookup_symbol_value(&self, s: &Symbol) -> Option<LispObject> {
        lookup_symbol!(self, sym_env, s)
    }

    pub fn lookup_symbol_function(&self, s: &Symbol) -> Option<Function> {
        lookup_symbol!(self, fn_env, s)
    }

    pub fn lookup_symbol_macro(&self, s: &Symbol) -> Option<Function> {
        lookup_symbol!(self, macro_env, s)
    }

    pub fn set_local_value(&mut self, s: Symbol, val: LispObject) {
        self.local_env.sym_env.insert(s, val);
    }

    pub fn set_global_function(&mut self, s: Symbol, val: Function) {
        self.global_env_mut().fn_env.insert(s, val);
    }

    pub fn set_global_macro(&mut self, s: Symbol, val: Function) {
        self.global_env_mut().macro_env.insert(s, val);
    }

    pub fn set_global_special(&mut self, s: Symbol, val: NativeFnWrapper) {
        self.global_env_mut().special_env.insert(s, val);
    }

    pub fn push_stack_frame_name(&mut self, name: Symbol) {
        let cur_stack_trace = &self.local_env.stack_trace;
        self.local_env.stack_trace = cur_stack_trace.cons(StackFrameDesignator::Name(name));
    }

    pub fn push_stack_frame_sig(&mut self, sig: FunctionSignature) {
        let cur_stack_trace = &self.local_env.stack_trace;
        self.local_env.stack_trace = cur_stack_trace.cons(StackFrameDesignator::Signature(sig));
    }

    pub fn get_stack_trace(&self) -> StackTrace {
        self.local_env.stack_trace.clone()
    }

    pub fn attach_st_box<T>(&self, res: Result<T, Box<Error>>) -> Result<T, ErrorWithStackTrace> {
        res.map_err(|e| self.st_box_err(e))
    }

    pub fn attach_st<T, E: Error + 'static>(&self, res: Result<T, E>) -> Result<T, ErrorWithStackTrace> {
        res.map_err(|e| self.st_err(e))
    }

    pub fn st_err<T: Error + 'static>(&self, err: T) -> ErrorWithStackTrace {
        self.st_box_err(Box::new(err))
    }

    pub fn st_box_err(&self, err: Box<Error>) -> ErrorWithStackTrace {
        ErrorWithStackTrace::new(err, self.get_stack_trace())
    }
}
