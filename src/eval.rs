use im::Vector;
use core;
use core::LispObject;
use core::Symbol;
use core::Env;
use core::EnvFrame;
use error;
use scopeguard::guard;

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

define_native_fn!{
    native_cons(_env, item: core::identity_converter, list: core::to_vector) -> LispObject::Vector {
        list.push_front(item);
        list
    }
}

define_native_fn!{
    native_list_star(_env, ... args: core::identity_converter) -> LispObject::Vector {
        let len = args.len();
        let mut list = core::to_vector(args.remove(len - 1))?;
        for arg in args.into_iter().rev() {
            list.push_front(arg);
        }

        list
    }
}


fn fill_stdlib(frame: &mut core::GlobalEnvFrame) {
    frame.fn_env.insert(Symbol("add".to_owned()),
                        core::Function::NativeFunction(
                            core::NativeFnWrapper(native_add)));
    frame.fn_env.insert(Symbol("list".to_owned()),
                        core::Function::NativeFunction(
                            core::NativeFnWrapper(native_list)));
    frame.fn_env.insert(Symbol("cons".to_owned()),
                        core::Function::NativeFunction(
                            core::NativeFnWrapper(native_cons)));
    frame.fn_env.insert(Symbol("sub".to_owned()),
                        core::Function::NativeFunction(
                            core::NativeFnWrapper(native_sub)));
    frame.fn_env.insert(Symbol("mul".to_owned()),
                        core::Function::NativeFunction(
                            core::NativeFnWrapper(native_mul)));
    frame.fn_env.insert(Symbol("inteq".to_owned()),
                        core::Function::NativeFunction(
                            core::NativeFnWrapper(native_int_eq)));
    frame.fn_env.insert(Symbol("liststar".to_owned()),
                        core::Function::NativeFunction(
                            core::NativeFnWrapper(native_list_star)));
}

pub fn prepare_stdlib(env: &mut Env) {
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

    if let Some(val) = env.global_env.sym_env.get(s) {
        Some(val.clone())
    } else {
        None
    }
}

fn lookup_symbol_fn(env: &mut Env, s: &Symbol) -> Option<core::Function> {
    for frame in &env.envs {
        if let Some(val) = frame.fn_env.get(s) {
            return Some(val.clone());
        }
    }

    if let Some(val) = env.global_env.fn_env.get(s) {
        Some(val.clone())
    } else {
        None
    }
}

fn call_interpreted_fn(env: &mut Env, form: LispObject) -> error::GenResult<LispObject> {
    let form = core::to_vector(form)?;
    let func = nth(form.clone(), 0).unwrap();
    let func = core::to_interpreted_function(core::to_function(func)?)?;

    let args = form.clone().slice(1..);
    if func.arglist.len() != args.len() {
        let expected = func.arglist.len();
        let actual = args.len();
        let arglist_as_vec = LispObject::Vector(
            func.arglist
                .into_iter()
                .map(|s| LispObject::Symbol(s))
                .collect());

        return Err(Box::new(
            error::ArityError::new(expected,
                                   actual,
                                   format!("(lambda {} ...)",
                                           arglist_as_vec))));
    }

    let args = args
        .into_iter()
        .map(|lo| eval(env, lo))
        .collect::<Result<Vector<_>, _>>()?;

    let mut frame = EnvFrame::new();
    for (sym, val) in func.arglist.into_iter().zip(args.into_iter()) {
        frame.sym_env.insert(sym, val);
    }

    env.push_frame(frame);

    let mut result = LispObject::Nil;
    for form in func.body {
        result = eval(env, form)?;
    }

    Ok(result)
}

fn call_native_fn(env: &mut Env, form: LispObject) -> error::GenResult<LispObject> {
    let form = core::to_vector(form)?;
    let prepared_form: Vector<LispObject> = form.clone()
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

    if let Some(&core::NativeFnWrapper(f)) = env.global_env.special_env.get(&sym) {
        f(env, LispObject::Vector(form))
    } else if let Some(lo) = lookup_symbol_fn(env, &sym) {
        form.pop_front();
        form.push_front(LispObject::Fn(lo));
        eval(env, LispObject::Vector(form))
    } else {
        Err(Box::new(error::UndefinedSymbol::new(sym.0.to_string(), true)))
    }
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

        LispObject::Vector(ref vec) if vec.len() == 0 => {
            Ok(LispObject::Vector(vec.clone()))
        },
        LispObject::Symbol(s) => {
            lookup_symbol_value(env, &s)
                .ok_or(Box::new(error::UndefinedSymbol::new(s.0, false)))
        },
        LispObject::Vector(ref vec) => {
            match nth(vec.clone(), 0).unwrap() {
                LispObject::Fn(core::Function::InterpretedFunction(_)) => call_interpreted_fn(env, form.clone()),
                LispObject::Fn(core::Function::NativeFunction(_)) => call_native_fn(env, form.clone()),
                LispObject::Symbol(_) => call_symbol(env, form.clone()),
                head => {
                    let head_val = eval(env, head)?;
                    let mut new_form = vec.clone().slice(1..);
                    new_form.push_front(head_val);
                    eval(env, LispObject::Vector(new_form))
                }
            }
        }
    }
}
