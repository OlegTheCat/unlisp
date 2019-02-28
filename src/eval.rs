use core;
use core::Env;
use core::EnvFrame;
use core::LispObject;
use core::Symbol;
use error;
use im::Vector;
use scopeguard::guard;
use std::ops::DerefMut;

define_native_fn! {
    native_add(_env, ... args: core::to_i64) -> LispObject::Integer {
        let mut res = 0;
        for arg in args {
            res += arg
        }
        res
    }
}

define_native_fn! {
    native_sub(_env, from: core::to_i64, ... args: core::to_i64) -> LispObject::Integer {
        let mut res = from;
        for arg in args {
            res -= arg
        }
        res
    }
}

define_native_fn! {
    native_mul(_env, from: core::to_i64, ... args: core::to_i64) -> LispObject::Integer {
        let mut res = from;
        for arg in args {
            res *= arg
        }
        res
    }
}

define_native_fn! {
    native_equal(_env, x: core::identity_converter, y: core::identity_converter) -> core::identity {
        if x == y {
            LispObject::T
        } else {
            LispObject::Nil
        }
    }
}

define_native_fn! {
    native_cons(_env, item: core::identity_converter, list: core::to_vector) -> LispObject::Vector {
        list.push_front(item);
        list
    }
}

define_native_fn! {
    native_first(_env, list: core::to_vector) -> core::identity {
        let first = list.into_iter().next()
            .ok_or(
                Box::new(
                    error::GenericError::new(
                        "cannot do first on empty list".to_string())))?;
        first
    }
}

define_native_fn! {
    native_rest(_env, list: core::to_vector) -> LispObject::Vector {
        list.slice(1..)
    }
}

define_native_fn! {
    native_listp(_env, arg: core::identity_converter) -> core::identity {
        let converted = core::to_vector(arg);
        if converted.is_ok() {
            LispObject::T
        } else {
            LispObject::Nil
        }
    }
}

define_native_fn! {
    native_emptyp(_env, arg: core::to_vector) -> core::identity {
        if arg.is_empty() {
            LispObject::T
        } else {
            LispObject::Nil
        }
    }
}

define_native_fn! {
    native_symbolp(_env, arg: core::identity_converter) -> core::identity {
        let converted = core::to_symbol(arg);
        if converted.is_ok() {
            LispObject::T
        } else {
            LispObject::Nil
        }
    }
}

fn fill_stdlib(frame: &mut core::GlobalEnvFrame) {
    let mut set = |name: &str, f| {
        frame.fn_env.insert(
            Symbol(name.to_string()),
            core::Function::NativeFunction(core::NativeFnWrapper(f)),
        );
    };

    set("add", native_add);
    set("sub", native_sub);
    set("mul", native_mul);
    set("equal", native_equal);
    set("cons", native_cons);
    set("first", native_first);
    set("rest", native_rest);
    set("listp", native_listp);
    set("emptyp", native_emptyp);
    set("symbolp", native_symbolp);
}

pub fn prepare_native_stdlib(env: &mut Env) {
    fill_stdlib(&mut env.global_env);
}

fn nth(vec: Vector<LispObject>, i: usize) -> Option<LispObject> {
    vec.into_iter().nth(i)
}

fn syntax_err(message: &str) -> error::SyntaxError {
    error::SyntaxError::new(message.to_string())
}

fn lookup_symbol_value(env: &mut Env, s: &Symbol) -> Option<LispObject> {
    for frame in &env.envs {
        if let Some(val) = frame.sym_env.get(s) {
            return Some(val.clone());
        }
    }

    env.global_env.sym_env.get(s).map(|v| v.clone())
}

pub fn lookup_symbol_fn(env: &mut Env, s: &Symbol) -> Option<core::Function> {
    for frame in &env.envs {
        if let Some(val) = frame.fn_env.get(s) {
            return Some(val.clone());
        }
    }

    env.global_env.fn_env.get(s).map(|v| v.clone())
}

pub fn lookup_symbol_macro(env: &mut Env, s: &Symbol) -> Option<core::Function> {
    for frame in &env.envs {
        if let Some(val) = frame.macro_env.get(s) {
            return Some(val.clone());
        }
    }

    env.global_env.macro_env.get(s).map(|v| v.clone())
}

pub fn call_function_object(env: &mut Env, f: core::Function, args: Vector<LispObject>, eval_args: bool) -> error::GenResult<LispObject> {
    match f {
        core::Function::NativeFunction(native_fn) => {
            let mut args: Vector<LispObject> = args
                .into_iter()
                .map(|lo| if eval_args { eval(env, lo) } else { Ok(lo) })
                .collect::<Result<Vector<_>, _>>()?;

            args.push_front(core::LispObject::Fn(core::Function::NativeFunction(native_fn.clone())));

            native_fn.0(env, LispObject::Vector(args))
        },
        core::Function::InterpretedFunction(interpreted_fn) => {
            let has_restarg = interpreted_fn.restarg.is_some();

            if (args.len() < interpreted_fn.arglist.len()) || (!has_restarg && interpreted_fn.arglist.len() != args.len()) {
                let expected = interpreted_fn.arglist.len();
                let actual = args.len();
                let mut arglist_as_vec = interpreted_fn
                    .arglist
                    .into_iter()
                    .map(|s| LispObject::Symbol(s))
                    .collect::<Vector<_>>();

                if let Some(restarg) = interpreted_fn.restarg {
                    arglist_as_vec.push_back(LispObject::Symbol(Symbol("&".to_string())));
                    arglist_as_vec.push_back(LispObject::Symbol(restarg));
                }

                let arglist_as_vec = LispObject::Vector(arglist_as_vec);

                return Err(Box::new(error::ArityError::new(
                    expected,
                    actual,
                    format!("(lambda {} ...)", arglist_as_vec),
                )));
            }

            let mut args_iter = args
                .into_iter()
                .map(|lo| if eval_args { eval(env, lo) } else { Ok(lo) })
                .collect::<Result<Vector<_>, _>>()?
                .into_iter();

            let mut frame = EnvFrame::new();
            for (sym, val) in interpreted_fn.arglist.into_iter().zip(args_iter.by_ref()) {
                frame.sym_env.insert(sym, val);
            }

            if has_restarg {
                let restarg = args_iter.collect();
                frame
                    .sym_env
                    .insert(interpreted_fn.restarg.unwrap(), LispObject::Vector(restarg));
            }

            env.push_frame(frame);

            let mut env = guard(env, |env| env.pop_frame());

            let mut result = LispObject::Nil;
            for form in interpreted_fn.body {
                result = eval(env.deref_mut(), form)?;
            }

            Ok(result)
        }
    }
}

fn call_fn(env: &mut Env, form: LispObject) -> error::GenResult<LispObject> {
    let mut form = core::to_vector(form)?;
    let func = core::to_function(nth(form.clone(), 0).unwrap())?;
    call_function_object(env, func, form.slice(1..), true)
}

fn call_macro(env: &mut Env, form: LispObject) -> error::GenResult<LispObject> {
    let mut form = core::to_vector(form)?;
    let func = core::to_macro(nth(form.clone(), 0).unwrap())?;
    let expanded = call_function_object(env, func, form.slice(1..), false)?;
    eval(env, expanded)
}

fn call_symbol(env: &mut Env, form: LispObject) -> error::GenResult<LispObject> {
    let mut form = core::to_vector(form)?;
    let sym = core::to_symbol(form[0].clone())?;

    let obj = if let Some(&f) = env.global_env.special_env.get(&sym) {
        LispObject::Special(f)
    } else if let Some(f) = lookup_symbol_fn(env, &sym) {
        LispObject::Fn(f)
    } else if let Some(f) = lookup_symbol_macro(env, &sym) {
        LispObject::Macro(f)
    } else {
        return Err(Box::new(error::UndefinedSymbol::new(
            sym.0.to_string(),
            true,
        )));
    };

    form.pop_front();
    form.push_front(obj);
    eval(env, LispObject::Vector(form))
}

pub fn eval(env: &mut Env, form: LispObject) -> error::GenResult<LispObject> {
    match form {
        self_eval @ LispObject::Nil => Ok(self_eval),
        self_eval @ LispObject::T => Ok(self_eval),
        self_eval @ LispObject::Integer(_) => Ok(self_eval),
        self_eval @ LispObject::String(_) => Ok(self_eval),
        self_eval @ LispObject::Fn(_) => Ok(self_eval),

        LispObject::Special(_) => Err(Box::new(syntax_err("standalone special"))),
        LispObject::Macro(_) => Err(Box::new(syntax_err("standalone macro"))),

        LispObject::Vector(ref vec) if vec.len() == 0 => Ok(LispObject::Vector(vec.clone())),
        LispObject::Symbol(s) => {
            lookup_symbol_value(env, &s).ok_or(Box::new(error::UndefinedSymbol::new(s.0, false)))
        }
        LispObject::Vector(ref vec) => match nth(vec.clone(), 0).unwrap() {
            LispObject::Symbol(_) => call_symbol(env, form.clone()),

            LispObject::Fn(_) => call_fn(env, form.clone()),
            LispObject::Macro(_) => call_macro(env, form.clone()),
            LispObject::Special(core::NativeFnWrapper(f)) => f(env, form.clone()),

            _=> Err(Box::new(syntax_err("illegal function call")))
        },
    }
}
