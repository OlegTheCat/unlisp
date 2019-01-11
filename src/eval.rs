use im::Vector;
use core;
use core::LispObject;
use core::Symbol;
use core::Env;
use core::EnvFrame;


define_vararg_native_fn! {
    add(_env, ... args: core::to_i64) -> LispObject::Integer {
        let mut res = 0;
        for arg in args {
            res += arg
        }
        res
    }
}

define_vararg_native_fn! {
    sub(_env, from: core::to_i64, ... args: core::to_i64) -> LispObject::Integer {
        let mut res = from;
        for arg in args {
            res -= arg
        }
        res
    }
}


define_vararg_native_fn! {
    list(_env, ... args: core::identity) -> LispObject::Vector {
        args
    }
}

define_native_fn!{
    cons(_env, item: core::identity, list: core::to_vector) -> LispObject::Vector {
        list.push_front(item);
        list
    }
}

fn fill_stdlib(global_frame: &mut EnvFrame) {
    global_frame.fn_env.insert(Symbol("add".to_owned()),
                               core::Function::NativeFunction(
                                   core::NativeFnWrapper(add)));
    global_frame.fn_env.insert(Symbol("list".to_owned()),
                               core::Function::NativeFunction(
                                   core::NativeFnWrapper(list)));
    global_frame.fn_env.insert(Symbol("cons".to_owned()),
                               core::Function::NativeFunction(
                                   core::NativeFnWrapper(cons)));
    global_frame.fn_env.insert(Symbol("sub".to_owned()),
                               core::Function::NativeFunction(
                                   core::NativeFnWrapper(sub)));
}

pub fn prepare_stdlib(env: &mut Env) {
    fill_stdlib(env.envs.iter_mut().last().unwrap())
}

fn nth(vec: Vector<LispObject>, i: usize) -> LispObject {
    vec.into_iter().nth(i).unwrap()
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

fn let_form(env: &mut Env, form: LispObject) -> LispObject {
    let form = core::to_vector(form);
    let bindings = core::to_vector(nth(form.clone(), 1));
    let mut new_env = env.clone();

    for binding in bindings {
        let binding = core::to_vector(binding);
        let sym = core::to_symbol(nth(binding.clone(), 0));
        let val = eval(env, nth(binding.clone(), 1));
        let mut env_frame = EnvFrame::new();
        env_frame.sym_env.insert(sym, val);
        new_env = new_env.push_frame(env_frame);
    }

    let body = form.clone().slice(2..);
    let mut res = LispObject::Nil;
    for form in body {
        res = eval(&mut new_env, form);
    }

    res
}

fn quote_form(_env: &mut Env, form: LispObject) -> LispObject {
    nth(core::to_vector(form), 1)
}

fn lambda_form(_env: &mut Env, form: LispObject) -> LispObject {
    let form = core::to_vector(form);
    let arglist = core::to_vector(nth(form.clone(), 1))
        .into_iter()
        .map(|lo| core::to_symbol(lo))
        .collect();
    let body = form.clone().slice(2..);

    LispObject::Fn(
        core::Function::InterpretedFunction(
            core::InterpretedFnWrapper {
                arglist: arglist,
                body: body
        }))
}

fn call_interpreted_fn(env: &mut Env, form: LispObject) -> LispObject {
    let form = core::to_vector(form);
    let func = core::to_interpreted_function(
        core::to_function(nth(form.clone(), 0)));
    let arg_vals: Vector<LispObject> = form.clone().slice(1..)
        .into_iter()
        .map(|lo| eval(env, lo))
        .collect();

    if func.arglist.len() != arg_vals.len() {
        panic!("Wrong number of arguments passed to a function")
    }

    let mut frame = EnvFrame::new();
    for (sym, val) in func.arglist.clone().into_iter().zip(arg_vals.into_iter()) {
        frame.sym_env.insert(sym, val);
    }

    let mut new_env = env.push_frame(frame);

    let mut result = LispObject::Nil;
    for form in func.body {
        result = eval(&mut new_env, form);
    }

    result
}

fn call_native_fn(env: &mut Env, form: LispObject) -> LispObject {
    let vec = core::to_vector(form);
    let prepared_form: Vector<LispObject> = vec.clone()
        .into_iter()
        .map(|lo| eval(env, lo))
        .collect();

    let f = core::to_native_function(core::to_function(nth(vec, 0)));
    f.0(env, LispObject::Vector(prepared_form))
}

fn funcall(env: &mut Env, form: LispObject) -> LispObject {
    let form = core::to_vector(form);
    let f = core::to_function(eval(env, nth(form.clone(), 1)));
    let mut args = form.clone().slice(2..);
    args.push_front(LispObject::Fn(f));
    eval(env, LispObject::Vector(args))
}

fn set_fn(env: &mut Env, form: LispObject) -> LispObject {
    let form = core::to_vector(form);
    let sym = core::to_symbol(nth(form.clone(), 1));
    let f = core::to_function(eval(env, nth(form.clone(), 2)));
    let envs = &mut env.envs;
    envs.iter_mut().last().unwrap().fn_env.insert(sym, f);
    LispObject::Nil
}

pub fn eval(env: &mut Env, form: LispObject) -> LispObject {
    match form {
        LispObject::Nil => LispObject::Nil,
        LispObject::T => LispObject::T,
        LispObject::Integer(i) => LispObject::Integer(i),
        LispObject::String(s) => LispObject::String(s),
        LispObject::Symbol(s) => lookup_symbol_value(env, &s).unwrap(),
        LispObject::Fn(f) => LispObject::Fn(f),
        LispObject::Vector(ref vec) => {
            match nth(vec.clone(), 0) {
                LispObject::Fn(core::Function::InterpretedFunction(_)) => call_interpreted_fn(env, form.clone()),
                LispObject::Fn(core::Function::NativeFunction(_)) => call_native_fn(env, form.clone()),
                LispObject::Symbol(Symbol(ref s)) if s == "let" => let_form(env, form.clone()),
                LispObject::Symbol(Symbol(ref s)) if s == "quote" => quote_form(env, form.clone()),
                LispObject::Symbol(Symbol(ref s)) if s == "lambda" => lambda_form(env, form.clone()),
                LispObject::Symbol(Symbol(ref s)) if s == "funcall" => funcall(env, form.clone()),
                LispObject::Symbol(Symbol(ref s)) if s == "setfn" => set_fn(env, form.clone()),
                LispObject::Symbol(s) => {
                    let f = lookup_symbol_fn(env, &s).unwrap();
                    let mut new_form = vec.clone().slice(1..);
                    new_form.push_front(LispObject::Fn(f));
                    eval(env, LispObject::Vector(new_form))
                },
                head => {
                    let head_val = eval(env, head);
                    let mut new_form = vec.clone().slice(1..);
                    new_form.push_front(head_val);
                    eval(env, LispObject::Vector(new_form))
                }
            }
        }
    }
}
