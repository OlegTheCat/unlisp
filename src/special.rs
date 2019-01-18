use im::Vector;
use core;
use core::LispObject;
use core::Symbol;
use core::Env;
use core::EnvFrame;
use error;
use eval;
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


fn defmacro(env: &mut Env, form: LispObject) -> error::GenResult<LispObject> {
    let mut form = core::to_vector(form)?;

    let name = nth(form.clone(), 1).ok_or(syntax_err("no name in defmacro"))?;
    let name = core::to_symbol(name)
        .map_err(|_e| syntax_err("macro name must be a symbol"))?;

    let arglist = nth(form.clone(), 2)
        .ok_or(syntax_err("no arglist in defmacro"))?;
    let arglist = core::to_vector(arglist)
        .map_err(|_e| syntax_err("defmacro arglist is not a list"))?;
    let arglist = arglist.into_iter()
        .map(|lo| core::to_symbol(lo))
        .collect::<Result<Vector<_>, _>>()?;

    let body = form.slice(3..);

    let macro_fn = core::Function::InterpretedFunction(
        core::InterpretedFn {
            arglist: arglist,
            body: body
        });

    env.global_env.macro_env.insert(name, macro_fn);

    Ok(LispObject::Nil)
}

fn macroexpand1(env: &mut Env, form: LispObject) -> error::GenResult<LispObject> {

    let not_a_macro = || syntax_err("arg to macroexpand1 must be a macro call");

    let form = core::to_vector(form)?;
    let arg_form = nth(form, 1).ok_or(syntax_err("no arg in macroexpand1"))?;
    let mut arg_form = core::to_vector(arg_form) .map_err(|_e| not_a_macro())?;

    let macro_fn = core::to_symbol(nth(arg_form.clone(), 0).ok_or(not_a_macro())?)
        .map_err(|_e| not_a_macro())?;
    let macro_fn = eval::lookup_symbol_macro(env, &macro_fn).ok_or(not_a_macro())?;

    arg_form.pop_front();
    arg_form.push_front(LispObject::Macro(macro_fn.clone()));

    match macro_fn {
        core::Function::InterpretedFunction(_) =>
            eval::call_interpreted_macro(env, LispObject::Vector(arg_form)),
        core::Function::NativeFunction(core::NativeFnWrapper(f)) =>
            f(env, LispObject::Vector(arg_form))
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
    set("defmacro", defmacro);
    set("macroexpand1", macroexpand1);
}
