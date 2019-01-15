use im::Vector;
use core;
use core::LispObject;
use core::Symbol;
use core::Env;
use core::EnvFrame;
use error;

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

fn fill_stdlib(global_frame: &mut EnvFrame) {
    global_frame.fn_env.insert(Symbol("add".to_owned()),
                               core::Function::NativeFunction(
                                   core::NativeFnWrapper(native_add)));
    global_frame.fn_env.insert(Symbol("list".to_owned()),
                               core::Function::NativeFunction(
                                   core::NativeFnWrapper(native_list)));
    global_frame.fn_env.insert(Symbol("cons".to_owned()),
                               core::Function::NativeFunction(
                                   core::NativeFnWrapper(native_cons)));
    global_frame.fn_env.insert(Symbol("sub".to_owned()),
                               core::Function::NativeFunction(
                                   core::NativeFnWrapper(native_sub)));
    global_frame.fn_env.insert(Symbol("mul".to_owned()),
                               core::Function::NativeFunction(
                                   core::NativeFnWrapper(native_mul)));
    global_frame.fn_env.insert(Symbol("inteq".to_owned()),
                               core::Function::NativeFunction(
                                   core::NativeFnWrapper(native_int_eq)));
}

fn syntax_err(message: &str) -> error::SyntaxError {
    error::SyntaxError::new(message.to_string())
}

pub fn prepare_stdlib(env: &mut Env) {
    fill_stdlib(env.envs.iter_mut().last().unwrap())
}

fn nth(vec: Vector<LispObject>, i: usize) -> Option<LispObject> {
    vec.into_iter().nth(i)
}

fn lookup_symbol_value(env: &mut Env, s: &Symbol) -> Option<LispObject> {
    for frame in &env.envs {
        let val = frame.sym_env.get(s);
        if val.is_some() {
            return Some(val.unwrap().clone());
        }
    }

    None
}

fn lookup_symbol_fn(env: &mut Env, s: &Symbol) -> Option<core::Function> {
    for frame in &env.envs {
        let val = frame.fn_env.get(s);
        if val.is_some() {
            return Some(val.unwrap().clone());
        }
    }

    None
}

fn let_form(env: &mut Env, form: LispObject) -> error::GenResult<LispObject> {
    let form = core::to_vector(form)?;
    let bindings = nth(form.clone(), 1).ok_or(syntax_err("no bindings in let"))?;
    let bindings = core::to_vector(bindings)
        .map_err(|_e| syntax_err("let bindings are not a list"))?;
    let mut new_env = env.clone();

    for binding in bindings {
        let binding = core::to_vector(binding)
            .map_err(|_e| syntax_err("let binding is not a list"))?;
        let sym = nth(binding.clone(), 0)
            .ok_or(syntax_err("empty binding clause"))?;
        let sym = core::to_symbol(sym)
            .map_err(|_e| syntax_err("not a symbol in binding clause"))?;

        let val = nth(binding.clone(), 1)
            .ok_or(syntax_err("no value in binding clause"))?;
        let val = eval(env, val)?;
        let mut env_frame = EnvFrame::new();
        env_frame.sym_env.insert(sym, val);
        new_env = new_env.push_frame(env_frame);
    }

    let body = form.clone().slice(2..);
    let mut res = LispObject::Nil;
    for form in body {
        res = eval(&mut new_env, form)?;
    }

    Ok(res)
}

fn quote_form(_env: &mut Env, form: LispObject) -> error::GenResult<LispObject> {
    let form = core::to_vector(form)?;
    if form.len() != 2 {
        return Err(Box::new(error::ArityError::new(1, form.len() - 1,
                                                   "quote".to_string())))
    }

    Ok(nth(form, 1).unwrap())
}

fn lambda_form(_env: &mut Env, form: LispObject) -> error::GenResult<LispObject> {
    let form = core::to_vector(form)?;

    let arglist = nth(form.clone(), 1)
        .ok_or(syntax_err("no arglist in lambda"))?;
    let arglist = core::to_vector(arglist)
        .map_err(|_e| syntax_err("lambda arglist in not a list"))?;
    let arglist = arglist
        .into_iter()
        .map(|lo| core::to_symbol(lo)
             .map_err(|_e| syntax_err("expected symbol in arglist")))
        .collect::<Result<Vector<_>, _>>()?;

    let body = form.clone().slice(2..);

    Ok(LispObject::Fn(
        core::Function::InterpretedFunction(
            core::InterpretedFn {
                arglist: arglist,
                body: body
            })))
}

fn call_interpreted_fn(env: &mut Env, form: LispObject) -> error::GenResult<LispObject> {
    let form = core::to_vector(form)?;
    let func = nth(form.clone(), 0).unwrap();
    let func = core::to_interpreted_function(core::to_function(func)?)?;

    let args = form.clone().slice(1..);
    if func.arglist.len() != args.len() {
        return Err(Box::new(
            error::ArityError::new(func.arglist.len(),
                                   args.len(),
                                   format!("(lambda {:?})", &func.arglist))));
    }

    let args = args
        .into_iter()
        .map(|lo| eval(env, lo))
        .collect::<Result<Vector<_>, _>>()?;

    let mut frame = EnvFrame::new();
    for (sym, val) in func.arglist.into_iter().zip(args.into_iter()) {
        frame.sym_env.insert(sym, val);
    }

    let mut new_env = env.push_frame(frame);

    let mut result = LispObject::Nil;
    for form in func.body {
        result = eval(&mut new_env, form)?;
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

fn funcall(env: &mut Env, form: LispObject) -> error::GenResult<LispObject> {
    let form = core::to_vector(form)?;

    let func = nth(form.clone(), 1).unwrap();
    let func = core::to_function(eval(env, func)?)?;
    let mut args = form.clone().slice(2..);
    args.push_front(LispObject::Fn(func));
    eval(env, LispObject::Vector(args))
}

fn set_fn(env: &mut Env, form: LispObject) -> error::GenResult<LispObject> {
    let form = core::to_vector(form)?;

    let sym = nth(form.clone(), 1).ok_or(syntax_err("no symbol in setfn"))?;
    let sym = core::to_symbol(sym).map_err(|_e| syntax_err("not a symbol in setfn"))?;

    let func = nth(form.clone(), 2).ok_or(syntax_err("no function in setfn"))?;
    let func = core::to_function(eval(env, func)?)?;
    let envs = &mut env.envs;
    envs.iter_mut().last().unwrap().fn_env.insert(sym, func);
    Ok(LispObject::Nil)
}

fn if_form(env: &mut Env, form: LispObject) -> error::GenResult<LispObject> {
    let form = core::to_vector(form)?;

    let cond = nth(form.clone(), 1).ok_or(syntax_err("no condition in if"))?;
    let then_form = nth(form.clone(), 2).ok_or(syntax_err("no then in if"))?;
    let else_form = nth(form.clone(), 3).unwrap_or(LispObject::Nil);

    let cond = eval(env, cond)?;
    if cond == LispObject::Nil {
        eval(env, else_form)
    } else {
        eval(env, then_form)
    }

}

pub fn eval(env: &mut Env, form: LispObject) -> error::GenResult<LispObject> {
    match form {
        self_eval @ LispObject::Nil => Ok(self_eval),
        self_eval @ LispObject::T => Ok(self_eval),
        self_eval @ LispObject::Integer(_) => Ok(self_eval),
        self_eval @ LispObject::String(_) => Ok(self_eval),
        self_eval @ LispObject::Fn(_) => Ok(self_eval),
        LispObject::Vector(ref vec) if vec.len() == 0 => {
            Ok(LispObject::Vector(vec.clone()))
        },
        LispObject::Symbol(s) => {
            lookup_symbol_value(env, &s)
                .ok_or(Box::new(error::UndefinedSymbol::new(s.0)))
        },
        LispObject::Vector(ref vec) => {
            match nth(vec.clone(), 0).unwrap() {
                LispObject::Fn(core::Function::InterpretedFunction(_)) => call_interpreted_fn(env, form.clone()),
                LispObject::Fn(core::Function::NativeFunction(_)) => call_native_fn(env, form.clone()),
                LispObject::Symbol(Symbol(ref s)) if s == "let" => let_form(env, form.clone()),
                LispObject::Symbol(Symbol(ref s)) if s == "if" => if_form(env, form.clone()),
                LispObject::Symbol(Symbol(ref s)) if s == "quote" => quote_form(env, form.clone()),
                LispObject::Symbol(Symbol(ref s)) if s == "lambda" => lambda_form(env, form.clone()),
                LispObject::Symbol(Symbol(ref s)) if s == "funcall" => funcall(env, form.clone()),
                LispObject::Symbol(Symbol(ref s)) if s == "setfn" => set_fn(env, form.clone()),
                LispObject::Symbol(s) => {
                    let f = lookup_symbol_fn(env, &s)
                        .ok_or(Box::new(error::UndefinedSymbol::new(s.0)))?;
                    let mut new_form = vec.clone().slice(1..);
                    new_form.push_front(LispObject::Fn(f));
                    eval(env, LispObject::Vector(new_form))
                },
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
