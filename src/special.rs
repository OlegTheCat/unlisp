use im::Vector;
use core;
use core::LispObject;
use core::Symbol;
use core::Env;
use core::EnvFrame;
use error;
use eval::eval;
use std::ops::DerefMut;
use scopeguard::guard;

fn nth(vec: Vector<LispObject>, i: usize) -> Option<LispObject> {
    vec.into_iter().nth(i)
}

fn syntax_err(message: &str) -> error::SyntaxError {
    error::SyntaxError::new(message.to_string())
}

fn let_form(env: &mut Env, form: LispObject) -> error::GenResult<LispObject> {
    let form = core::to_vector(form)?;
    let bindings = nth(form.clone(), 1).ok_or(syntax_err("no bindings in let"))?;
    let bindings = core::to_vector(bindings)
        .map_err(|_e| syntax_err("let bindings are not a list"))?;

    let bindings_len = bindings.len();

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

        env.push_frame(env_frame);
    }

    let mut env = guard(env, |env| {
        for _ in 0..bindings_len {
            env.pop_frame();
        }
    });

    let body = form.clone().slice(2..);
    let mut res = LispObject::Nil;

    for form in body {
        res = eval(env.deref_mut(), form)?;
    }

    Ok(res)
}

fn quote_form(_env: &mut Env, form: LispObject) -> error::GenResult<LispObject> {
    let form = core::to_vector(form)?;
    if form.len() != 2 {
        return Err(
            Box::new(
                error::ArityError::new(1, form.len() - 1,
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

    env.global_env.fn_env.insert(sym, func);
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

pub fn prepare_specials(env: &mut core::Env) {
    let mut set = |s: &str, f| {
        env.global_env.special_env.insert(Symbol(s.to_string()),
                                          core::NativeFnWrapper(f));
    };

    set("if", if_form);
    set("let", let_form);
    set("setfn", set_fn);
    set("funcall", funcall);
    set("lambda", lambda_form);
    set("quote", quote_form);
}
