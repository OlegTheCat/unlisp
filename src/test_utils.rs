#![macro_use]

use crate::common;
use crate::env::Env;
use crate::error;
use crate::native;
use crate::object::*;
use crate::reader::Reader;
use crate::special;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Context {
    env: Env,
}

impl Context {
    pub fn new(load_specials: bool, load_natives: bool, load_stdlib: bool) -> Self {
        let mut env = Env::new();

        if load_specials {
            special::prepare_specials(&mut env);
        }

        if load_natives {
            native::prepare_natives(&mut env);
        }

        if load_stdlib {
            if !(load_specials && load_natives) {
                panic!("cannot load stdlib without specials or natives");
            }

            common::eval_stdlib(&env);
        }

        Self { env: env }
    }

    fn env(&self) -> Env {
        Env {
            global_env: Rc::new(RefCell::new(self.env.global_env().clone())),
            cur_env: self.env.cur_env.clone(),
        }
    }

    pub fn eval(&self, s: impl Into<String>) -> LispObjectResult {
        let env = self.env();
        let s = s.into();
        let mut bytes = s.as_bytes();
        let mut reader = Reader::create(&mut bytes);
        let mut res = Ok(LispObject::nil());
        loop {
            match reader.read_form() {
                Ok(form) => {
                    res = common::macroexpand_and_eval(env.clone(), &form);
                }
                ref err @ Err(_) if common::is_gen_eof(err) => break,
                Err(e) => return Err(e),
            }
        }
        res
    }

    pub fn ok_eval(&self, s: impl Into<String>) -> LispObject {
        self.eval(s).unwrap()
    }

    pub fn err_eval(&self, s: impl Into<String>) -> error::GenError {
        self.eval(s).unwrap_err()
    }
}

pub fn read(s: impl Into<String>) -> LispObject {
    let s = s.into();
    let mut bytes = s.as_bytes();
    let mut reader = Reader::create(&mut bytes);
    reader.read_form().unwrap()
}

macro_rules! assert_ok {
    ($ctx:ident, $actual:expr, $expected:expr) => {
        assert_eq!($ctx.ok_eval($actual), read($expected));
    };
}

macro_rules! assert_err {
    ($ctx:ident, $actual:expr, $downcast_to:ty) => {
        assert!($ctx.err_eval($actual).downcast::<$downcast_to>().is_ok());
    };
}
