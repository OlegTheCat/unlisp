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
    native_int_eq(_env, x: core::to_i64, y: core::to_i64) -> core::identity {
        if x == y {
            LispObject::T
        } else {
            LispObject::Nil
        }
    }
}

define_native_fn! {
    native_list(_env, ... args: core::identity_converter) -> LispObject::Vector {
        args
    }
}

define_native_fn! {
    native_cons(_env, item: core::identity_converter, list: core::to_vector) -> LispObject::Vector {
        list.push_front(item);
        list
    }
}

define_native_fn! {
    native_list_star(_env, ... args: core::identity_converter) -> LispObject::Vector {
        let len = args.len();
        let mut list = core::to_vector(args.remove(len - 1))?;
        for arg in args.into_iter().rev() {
            list.push_front(arg);
        }

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

define_native_fn! {
    native_sym_eq(_env, x: core::identity_converter, y: core::identity_converter) -> core::identity {
        let x = core::to_symbol(x);
        let y = core::to_symbol(y);
        if x.is_ok() && y.is_ok() {
            if x.unwrap() == y.unwrap() {
                return Ok(LispObject::T)
            }
        }

        LispObject::Nil
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
    set("list", native_list);
    set("cons", native_cons);
    set("sub", native_sub);
    set("mul", native_mul);
    set("int-eq", native_int_eq);
    set("list*", native_list_star);
    set("first", native_first);
    set("rest", native_rest);
    set("listp", native_listp);
    set("emptyp", native_emptyp);
    set("sym-eq", native_sym_eq);
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

fn lookup_symbol_fn(env: &mut Env, s: &Symbol) -> Option<core::Function> {
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

pub fn call_interpreted_fn(
    env: &mut Env,
    form: LispObject,
    call_macro: bool,
) -> error::GenResult<LispObject> {
    let form = core::to_vector(form)?;
    let func = nth(form.clone(), 0).unwrap();
    let func = core::to_interpreted_function(if call_macro {
        core::to_macro(func)?
    } else {
        core::to_function(func)?
    })?;

    let args = form.clone().slice(1..);
    let has_restarg = func.restarg.is_some();

    if (args.len() < func.arglist.len()) || (!has_restarg && func.arglist.len() != args.len()) {
        let expected = func.arglist.len();
        let actual = args.len();
        let mut arglist_as_vec = func
            .arglist
            .into_iter()
            .map(|s| LispObject::Symbol(s))
            .collect::<Vector<_>>();

        if let Some(restarg) = func.restarg {
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
        .map(|lo| if call_macro { Ok(lo) } else { eval(env, lo) })
        .collect::<Result<Vector<_>, _>>()?
        .into_iter();

    let mut frame = EnvFrame::new();
    for (sym, val) in func.arglist.into_iter().zip(args_iter.by_ref()) {
        frame.sym_env.insert(sym, val);
    }

    if has_restarg {
        let restarg = args_iter.collect();
        frame
            .sym_env
            .insert(func.restarg.unwrap(), LispObject::Vector(restarg));
    }

    env.push_frame(frame);

    let mut env = guard(env, |env| env.pop_frame());

    let mut result = LispObject::Nil;
    for form in func.body {
        result = eval(env.deref_mut(), form)?;
    }

    Ok(result)
}

fn call_native_fn(env: &mut Env, form: LispObject) -> error::GenResult<LispObject> {
    let form = core::to_vector(form)?;
    let prepared_form: Vector<LispObject> = form
        .clone()
        .into_iter()
        .map(|lo| eval(env, lo))
        .collect::<Result<Vector<_>, _>>()?;

    let func = nth(form, 0).unwrap();
    let func = core::to_native_function(core::to_function(func)?)?;
    func.0(env, LispObject::Vector(prepared_form))
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

            LispObject::Fn(core::Function::InterpretedFunction(_)) => {
                call_interpreted_fn(env, form.clone(), false)
            }
            LispObject::Fn(core::Function::NativeFunction(_)) => call_native_fn(env, form.clone()),

            LispObject::Special(core::NativeFnWrapper(f)) => f(env, form.clone()),

            LispObject::Macro(core::Function::NativeFunction(core::NativeFnWrapper(f))) => {
                let expanded = f(env, form.clone())?;
                eval(env, expanded)
            }
            LispObject::Macro(core::Function::InterpretedFunction(_)) => {
                let expanded = call_interpreted_fn(env, form.clone(), true)?;
                eval(env, expanded)
            }

            head => {
                let head_val = eval(env, head)?;
                let mut new_form = vec.clone().slice(1..);
                new_form.push_front(head_val);
                eval(env, LispObject::Vector(new_form))
            }
        },
    }
}
