use cons::List;
use core;
use core::Env;
use core::LispObject;
use core::Symbol;
use error;
use eval::eval;
use eval;

fn syntax_err(message: &str) -> error::SyntaxError {
    error::SyntaxError::new(message.to_string())
}

pub struct ParsedQuote(pub LispObject);

pub fn parse_quote(args: &List<LispObject>) -> error::GenResult<ParsedQuote> {
    if args.len() != 1 {
        return Err(Box::new(error::ArityError::new(
            1,
            args.len(),
            "quote".to_string(),
        )));
    }

    Ok(ParsedQuote(args.ufirst().clone()))
}

fn quote_form(_env: Env, args: List<LispObject>) -> error::GenResult<LispObject> {
    Ok(parse_quote(&args)?.0)
}

pub struct ParsedLet<'a> {
    pub bindings: Vec<(Symbol, &'a LispObject)>,
    pub body: List<LispObject>
}

pub fn parse_let<'a>(args: &'a List<LispObject>) -> error::GenResult<ParsedLet<'a>> {
    let bindings = args.first().ok_or(syntax_err("no bindings in let"))?;
    let bindings =
        core::to_list(bindings).map_err(|_e| syntax_err("let bindings are not a list"))?;

    let mut collected_bindings = vec![];

    for binding in bindings.iter() {
        let binding =
            core::to_list(binding).map_err(|_e| syntax_err("let binding is not a list"))?;
        let mut binding_iter = binding.iter();
        let sym = binding_iter.next().ok_or(syntax_err("empty binding clause"))?;
        let sym =
            core::to_symbol(sym).map_err(|_e| syntax_err("not a symbol in binding clause"))?;

        let val_form = binding_iter.next().ok_or(syntax_err("no value in binding clause"))?;

        collected_bindings.push((sym.clone(), val_form));
    }

    Ok(ParsedLet {
        bindings: collected_bindings,
        body: args.tail()
    })
}

fn let_form(env: Env, args: List<LispObject>) -> error::GenResult<LispObject> {
    let ParsedLet { bindings, body } = parse_let(&args)?;

    let mut new_env = env;

    for (sym, val_form) in bindings {
        let val = eval(new_env.clone(), val_form)?;
        new_env.cur_env.sym_env.insert(sym, val);
    }

    let mut res = LispObject::nil();

    for form in body.iter() {
        res = eval(new_env.clone(), &form)?;
    }

    Ok(res)
}

pub struct ParsedLambda {
    pub simple_args: List<Symbol>,
    pub restarg: Option<Symbol>,
    pub body: List<LispObject>
}

fn parse_arglist(arglist: Vec<Symbol>) -> error::GenResult<(List<Symbol>, Option<Symbol>)> {
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

pub fn parse_lambda(args: &List<LispObject>) -> error::GenResult<ParsedLambda> {

    let arglist = args.first().ok_or(syntax_err("no arglist in lambda"))?;
    let arglist =
        core::to_list(arglist).map_err(|_e| syntax_err("lambda arglist in not a list"))?;
    let arglist = arglist
        .iter()
        .map(|lo| {
            core::to_symbol(lo)
                .map(|s| s.clone())
                .map_err(|_e| syntax_err("expected symbol in arglist"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let (simple_args, restarg) = parse_arglist(arglist)?;

    let body = args.tail();

    Ok(ParsedLambda {
        simple_args: simple_args,
        restarg: restarg,
        body: body
    })
}

fn lambda_form(_env: Env, args: List<LispObject>) -> error::GenResult<LispObject> {
    let ParsedLambda { simple_args, restarg, body } = parse_lambda(&args)?;

    Ok(LispObject::Fn(core::Function::InterpretedFunction(
        core::InterpretedFn {
            arglist: simple_args,
            body: body,
            restarg: restarg,
        },
    )))
}

fn set_fn(env: Env, args: List<LispObject>) -> error::GenResult<LispObject> {
    let mut args = args.iter();
    let sym = args.next().ok_or(syntax_err("no symbol in set-fn"))?;
    let sym = core::to_symbol(sym).map_err(|_e| syntax_err("not a symbol in set-fn"))?;

    let func = args.next().ok_or(syntax_err("no function in set-fn"))?;
    let func = core::to_function_owned(eval(env.clone(), &func)?)?;

    env.global_env
        .as_ref()
        .borrow_mut()
        .fn_env
        .insert(sym.clone(), func);
    Ok(LispObject::nil())
}

fn set_macro_fn(env: Env, args: List<LispObject>) -> error::GenResult<LispObject> {
    let mut args = args.iter();
    let sym = args.next().ok_or(syntax_err("no symbol in set-macro-fn"))?;
    let sym = core::to_symbol(sym).map_err(|_e| syntax_err("not a symbol in set-macro-fn"))?;

    let func = args
        .next()
        .ok_or(syntax_err("no function in set-macro-fn"))?;
    let func = core::to_function_owned(eval(env.clone(), &func)?)?;

    env.global_env
        .as_ref()
        .borrow_mut()
        .macro_env
        .insert(sym.clone(), func);
    Ok(LispObject::nil())
}

fn if_form(env: Env, args: List<LispObject>) -> error::GenResult<LispObject> {
    let mut args = args.iter();
    let cond = args.next().ok_or(syntax_err("no condition in if"))?;
    let then_form = args.next().ok_or(syntax_err("no then in if"))?;
    let nil = LispObject::nil();
    let else_form = args.next().unwrap_or(&nil);

    let cond = eval(env.clone(), cond)?;
    if cond == nil {
        eval(env, else_form)
    } else {
        eval(env, then_form)
    }
}

fn raise_error(_env: Env, args: List<LispObject>) -> error::GenResult<LispObject> {
    let mut args = args.iter();
    let arg_form = args.next().ok_or(syntax_err("no arg in error"))?;
    let arg = core::to_string(arg_form)?;
    Err(Box::new(error::GenericError::new(arg.clone())))
}

fn symbol_function(env: Env, args: List<LispObject>) -> error::GenResult<LispObject> {
    let mut args = args.iter();
    let arg = args.next().ok_or(syntax_err("no arg in symbol-function"))?;
    let arg = core::to_symbol(arg)?;
    let f = eval::lookup_symbol_function(&env, &arg).ok_or(error::UndefinedSymbol::new(arg.name(), true))?;
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
    set("error", raise_error);
    set("symbol-function", symbol_function);
}
