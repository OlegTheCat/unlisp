use core;
use core::Env;
use core::LispObject;
use core::Symbol;
use error;
use eval;
use eval::eval;
use im::Vector;

fn nth(vec: Vector<LispObject>, i: usize) -> Option<LispObject> {
    vec.into_iter().nth(i)
}

fn syntax_err(message: &str) -> error::SyntaxError {
    error::SyntaxError::new(message.to_string())
}

fn quote_form(_env: Env, form: &LispObject) -> error::GenResult<LispObject> {
    let form = core::to_vector(form)?;
    if form.len() != 2 {
        return Err(Box::new(error::ArityError::new(
            1,
            form.len() - 1,
            "quote".to_string(),
        )));
    }

    Ok(form[1].clone())
}

fn let_form(env: Env, form: &LispObject) -> error::GenResult<LispObject> {
    let form = core::to_vector(form)?;
    let bindings = form.get(1).ok_or(syntax_err("no bindings in let"))?;
    let bindings =
        core::to_vector(bindings).map_err(|_e| syntax_err("let bindings are not a list"))?;

    let mut new_env = env.clone();

    for binding in bindings {
        let binding =
            core::to_vector(binding).map_err(|_e| syntax_err("let binding is not a list"))?;
        let sym = binding.get(0).ok_or(syntax_err("empty binding clause"))?;
        let sym =
            core::to_symbol(sym).map_err(|_e| syntax_err("not a symbol in binding clause"))?;

        let val = binding.get(1).ok_or(syntax_err("no value in binding clause"))?;
        let val = eval(new_env.clone(), val)?;

        new_env.cur_env.sym_env.insert(sym.clone(), val);
    }

    let body = form.clone().slice(2..);
    let mut res = LispObject::Nil;

    for form in body {
        res = eval(new_env.clone(), &form)?;
    }

    Ok(res)
}

fn parse_arglist(arglist: Vector<Symbol>) -> error::GenResult<(Vector<Symbol>, Option<Symbol>)> {
    let mut iter = arglist.into_iter();
    let simple_args = iter
        .by_ref()
        .take_while(|s| *s != Symbol::new("&"))
        .collect();

    let restargs = iter.collect::<Vec<_>>();
    let restarg = if restargs.is_empty() {
        None
    } else {
        if restargs.len() != 1 {
            return Err(Box::new(syntax_err("wrong syntax near '&' in lambda")));
        } else {
            restargs.into_iter().next()
        }
    };

    Ok((simple_args, restarg))
}

fn lambda_form(_env: Env, form: &LispObject) -> error::GenResult<LispObject> {
    let form = core::to_vector(form)?;

    let arglist = form.get(1).ok_or(syntax_err("no arglist in lambda"))?;
    let arglist =
        core::to_vector(arglist).map_err(|_e| syntax_err("lambda arglist in not a list"))?;
    let arglist = arglist
        .into_iter()
        .map(|lo| core::to_symbol(lo).map(|s| s.clone()).map_err(|_e| syntax_err("expected symbol in arglist")))
        .collect::<Result<Vector<_>, _>>()?;

    let (simple_args, restarg) = parse_arglist(arglist)?;

    let body = form.clone().slice(2..);

    Ok(LispObject::Fn(core::Function::InterpretedFunction(
        core::InterpretedFn {
            arglist: simple_args,
            body: body,
            restarg: restarg,
        },
    )))
}

fn set_fn(env: Env, form: &LispObject) -> error::GenResult<LispObject> {
    let form = core::to_vector(form)?;

    let sym = form.get(1).ok_or(syntax_err("no symbol in set-fn"))?;
    let sym = core::to_symbol(sym).map_err(|_e| syntax_err("not a symbol in set-fn"))?;

    let func = form.get(2).ok_or(syntax_err("no function in set-fn"))?;
    let func = core::to_function_owned(eval(env.clone(), &func)?)?;

    env.global_env.as_ref().borrow_mut().fn_env.insert(sym.clone(), func);
    Ok(LispObject::Nil)
}

fn set_macro_fn(env: Env, form: &LispObject) -> error::GenResult<LispObject> {
    let form = core::to_vector(form)?;

    let sym = form.get(1).ok_or(syntax_err("no symbol in set-macro-fn"))?;
    let sym = core::to_symbol(sym).map_err(|_e| syntax_err("not a symbol in set-macro-fn"))?;

    let func = form.get(2).ok_or(syntax_err("no function in set-macro-fn"))?;
    let func = core::to_function_owned(eval(env.clone(), &func)?)?;

    env.global_env.as_ref().borrow_mut().macro_env.insert(sym.clone(), func);
    Ok(LispObject::Nil)
}

fn if_form(env: Env, form: &LispObject) -> error::GenResult<LispObject> {
    let form = core::to_vector(form)?;

    let cond = form.get(1).ok_or(syntax_err("no condition in if"))?;
    let then_form = form.get(2).ok_or(syntax_err("no then in if"))?;
    let nil = LispObject::Nil;
    let else_form = form.get(3).unwrap_or(&nil);

    let cond = eval(env.clone(), cond)?;
    if cond == LispObject::Nil {
        eval(env, else_form)
    } else {
        eval(env, then_form)
    }
}

// fn macroexpand_1(env: Env, form: LispObject) -> error::GenResult<LispObject> {
//     let not_a_macro = || syntax_err("arg to macroexpand1 must be a macro call");

//     let form = core::to_vector(form)?;
//     let arg_form = nth(form, 1).ok_or(syntax_err("no arg in macroexpand1"))?;
//     let mut arg_form = core::to_vector(arg_form).map_err(|_e| not_a_macro())?;

//     let macro_fn = core::to_symbol(nth(arg_form.clone(), 0).ok_or(not_a_macro())?)
//         .map_err(|_e| not_a_macro())?;
//     let macro_fn = eval::lookup_symbol_macro(&env, &macro_fn).ok_or(not_a_macro())?;

//     arg_form.pop_front();

//     eval::call_function_object(env, macro_fn, arg_form, false)
// }

fn raise_error(_env: Env, form: &LispObject) -> error::GenResult<LispObject> {
    let form = core::to_vector(form)?;
    let arg_form = form.get(1).ok_or(syntax_err("no arg in error"))?;
    let arg = core::to_string(arg_form)?;
    Err(Box::new(error::GenericError::new(arg.clone())))
}

fn symbol_function(env: Env, form: &LispObject) -> error::GenResult<LispObject> {
    let form = core::to_vector(form)?;
    let arg = form.get(1).ok_or(syntax_err("no arg in symbol-function"))?;
    let arg = core::to_symbol(arg)?;
    let f = eval::lookup_symbol_fn(&env, &arg).ok_or(error::UndefinedSymbol::new(
        arg.name(),
        true,
    ))?;


    Ok(LispObject::Fn(f))
}

pub fn prepare_specials(global_env: &mut core::GlobalEnvFrame) {
    let mut set = |s: &str, f| {
        global_env
            .special_env
            .insert(Symbol::new(s), core::NativeFnWrapper(f));
    };

    set("quote", quote_form);
    set("if", if_form);
    set("let", let_form);
    set("set-fn", set_fn);
    set("set-macro-fn", set_macro_fn);
    set("lambda", lambda_form);
    // set("macroexpand-1", macroexpand_1);
    set("error", raise_error);
    set("symbol-function", symbol_function);
}
