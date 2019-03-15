use core;
use core::Env;
use core::LispObject;
use core::Symbol;
use error;
use cons::List;

fn syntax_err(message: &str) -> error::SyntaxError {
    error::SyntaxError::new(message.to_string())
}

fn lookup_symbol_value(env: &Env, s: &Symbol) -> Option<LispObject> {
    if let Some(val) = env.cur_env.sym_env.get(s) {
        return Some(val.clone());
    }

    env.global_env.as_ref().borrow().sym_env.get(s).map(|v| v.clone())
}

pub fn lookup_symbol_fn(env: &Env, s: &Symbol) -> Option<core::Function> {
    if let Some(val) = env.cur_env.fn_env.get(s) {
        return Some(val.clone());
    }

    env.global_env.as_ref().borrow().fn_env.get(s).map(|v| v.clone())
}

pub fn lookup_symbol_macro(env: &Env, s: &Symbol) -> Option<core::Function> {
    if let Some(val) = env.cur_env.macro_env.get(s) {
        return Some(val.clone());
    }

    env.global_env.as_ref().borrow().macro_env.get(s).map(|v| v.clone())
}

pub fn call_function_object(env: Env, f: &core::Function, args: List<LispObject>, eval_args: bool) -> error::GenResult<LispObject> {

    let args = if eval_args {
        args.iter().map(|lo| eval(env.clone(), lo)).collect::<Result<List<_>, _>>()?
    } else {
        args
    };

    match f {
        core::Function::NativeFunction(native_fn) => {
            native_fn.0(env, args)
        },
        core::Function::InterpretedFunction(interpreted_fn) => {
            let has_restarg = interpreted_fn.restarg.is_some();

            if (args.len() < interpreted_fn.arglist.len()) || (!has_restarg && interpreted_fn.arglist.len() != args.len()) {
                let expected = interpreted_fn.arglist.len();
                let actual = args.len();
                let mut arglist = interpreted_fn
                    .arglist
                    .iter()
                    .map(|s| LispObject::Symbol(s.clone()))
                    .collect::<List<_>>();

                // TODO
                // if let Some(ref restarg) = interpreted_fn.restarg {
                //     arglist.push_back(LispObject::Symbol(Symbol::new("&")));
                //     arglist.push_back(LispObject::Symbol(restarg.clone()));
                // }

                let arglist = LispObject::List(arglist);

                return Err(Box::new(error::ArityError::new(
                    expected,
                    actual,
                    format!("(lambda {} ...)", arglist),
                )));
            }

            let mut args = args.iter();

            let mut new_env = env.clone();
            for (sym, val) in interpreted_fn.arglist.iter().zip(args.by_ref()) {
                new_env.cur_env.sym_env.insert(sym.clone(), val.clone());
            }

            if has_restarg {
                let restarg = args.map(|lo| lo.clone()).collect();
                new_env
                    .cur_env
                    .sym_env
                    .insert(interpreted_fn.restarg.as_ref().unwrap().clone(), LispObject::List(restarg));
            }

            let mut result = LispObject::nil();
            for form in interpreted_fn.body.iter() {
                result = eval(new_env.clone(), form)?;
            }

            Ok(result)
        }
    }
}

fn call_fn(env: Env, form: &LispObject) -> error::GenResult<LispObject> {
    let mut form = core::to_list(form)?;
    let func = core::to_function(form.ufirst())?;

    call_function_object(env, func, form.tail(), true)
}

fn call_macro(env: Env, form: &LispObject) -> error::GenResult<LispObject> {
    let mut form = core::to_list(form)?;
    let func = core::to_macro(form.ufirst())?;

    let expanded = call_function_object(env.clone(), func, form.tail(), false)?;
    eval(env, &expanded)
}

fn call_symbol(env: Env, form: &LispObject) -> error::GenResult<LispObject> {
    let form = core::to_list(form)?;
    let sym = core::to_symbol(form.first().unwrap())?;
    let args = form.tail();

    let spec = env.global_env.as_ref().borrow().special_env.get(sym).map(|f| f.clone());

    if let Some(f) = spec {
        f.0(env.clone(), args)
    } else if let Some(ref f) = lookup_symbol_fn(&env, sym) {
        call_function_object(env.clone(), f, args, true)
    } else if let Some(ref f) = lookup_symbol_macro(&env, &sym) {
        let expanded = call_function_object(env.clone(), f, args, false)?;
        eval(env.clone(), &expanded)
    } else {
        Err(Box::new(error::UndefinedSymbol::new(
            sym.name(),
            true,
        )))
    }
}

pub fn eval(env: Env, form: &LispObject) -> error::GenResult<LispObject> {
    match form {
        self_eval @ LispObject::T => Ok(self_eval.clone()),
        self_eval @ LispObject::Integer(_) => Ok(self_eval.clone()),
        self_eval @ LispObject::String(_) => Ok(self_eval.clone()),
        self_eval @ LispObject::Fn(_) => Ok(self_eval.clone()),

        LispObject::Special(_) => Err(Box::new(syntax_err("standalone special"))),
        LispObject::Macro(_) => Err(Box::new(syntax_err("standalone macro"))),

        LispObject::List(ref list) if list.is_empty() => Ok(LispObject::List(list.clone())),
        LispObject::Symbol(s) => {
            lookup_symbol_value(&env, &s).ok_or(Box::new(error::UndefinedSymbol::new(s.name(), false)))
        }
        LispObject::List(ref list) => match list.first().unwrap() {
            LispObject::Symbol(_) => call_symbol(env, form),

            // LispObject::Fn(_) => call_fn(env, form),
            // LispObject::Macro(_) => call_macro(env, form),
            // LispObject::Special(core::NativeFnWrapper(f)) => f(env, form),

            _=> Err(Box::new(syntax_err("illegal function call")))
        },
    }
}
