use cons::List;
use core;
use core::Env;
use core::LispObject;
use core::Symbol;
use error;

fn syntax_err(message: &str) -> error::SyntaxError {
    error::SyntaxError::new(message.to_string())
}

macro_rules! lookup_symbol {
    ($env:ident, $lookup_env:ident, $sym:expr) => {{
        let global = $env.global_env.borrow();
        $env.cur_env
            .$lookup_env
            .get($sym)
            .or_else(|| global.$lookup_env.get($sym))
            .map(|v| v.clone())
    }};
}

pub fn lookup_symbol_value(env: &Env, s: &Symbol) -> Option<LispObject> {
    lookup_symbol!(env, sym_env, s)
}

pub fn lookup_symbol_function(env: &Env, s: &Symbol) -> Option<core::Function> {
    lookup_symbol!(env, fn_env, s)
}

pub fn lookup_symbol_macro(env: &Env, s: &Symbol) -> Option<core::Function> {
    lookup_symbol!(env, macro_env, s)
}

pub fn call_function_object(
    env: Env,
    f: &core::Function,
    args: List<LispObject>,
    eval_args: bool,
) -> error::GenResult<LispObject> {
    let args = if eval_args {
        args.iter()
            .map(|lo| eval(env.clone(), lo))
            .collect::<Result<List<_>, _>>()?
    } else {
        args
    };

    match f {
        core::Function::NativeFunction(native_fn) => native_fn.0(env, args),
        core::Function::InterpretedFunction(interpreted_fn) => {
            let has_restarg = interpreted_fn.restarg.is_some();

            if (args.len() < interpreted_fn.arglist.len())
                || (!has_restarg && interpreted_fn.arglist.len() != args.len())
            {
                let expected = interpreted_fn.arglist.len();
                let actual = args.len();
                let mut arglist = interpreted_fn
                    .arglist
                    .iter()
                    .map(|s| LispObject::Symbol(s.clone()))
                    .collect::<Vec<_>>();

                if let Some(ref restarg) = interpreted_fn.restarg {
                    arglist.push(LispObject::Symbol(Symbol::new("&")));
                    arglist.push(LispObject::Symbol(restarg.clone()));
                }

                let arglist = LispObject::List(List::from_rev_iter(arglist));

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
                new_env.cur_env.sym_env.insert(
                    interpreted_fn.restarg.as_ref().unwrap().clone(),
                    LispObject::List(restarg),
                );
            }

            let mut result = LispObject::nil();
            for form in interpreted_fn.body.iter() {
                result = eval(new_env.clone(), form)?;
            }

            Ok(result)
        }
    }
}

fn call_symbol(env: Env, form: &LispObject) -> error::GenResult<LispObject> {
    let form = core::to_list(form)?;
    let sym = core::to_symbol(form.first().unwrap())?;
    let args = form.tail();

    let spec = env
        .global_env
        .borrow()
        .special_env
        .get(sym)
        .map(|f| f.clone());

    if let Some(f) = spec {
        f.0(env.clone(), args)
    } else if let Some(ref f) = lookup_symbol_function(&env, sym) {
        call_function_object(env.clone(), f, args, true)
    } else {
        Err(Box::new(error::UndefinedSymbol::new(sym.name(), true)))
    }
}

pub fn eval(env: Env, form: &LispObject) -> error::GenResult<LispObject> {
    match form {
        self_eval @ LispObject::T => Ok(self_eval.clone()),
        self_eval @ LispObject::Integer(_) => Ok(self_eval.clone()),
        self_eval @ LispObject::String(_) => Ok(self_eval.clone()),
        self_eval @ LispObject::Fn(_) => Ok(self_eval.clone()),

        LispObject::List(ref list) if list.is_empty() => Ok(LispObject::nil()),
        LispObject::Symbol(s) => lookup_symbol_value(&env, &s)
            .ok_or(Box::new(error::UndefinedSymbol::new(s.name(), false))),
        LispObject::List(ref list) => match list.ufirst() {
            LispObject::Symbol(_) => call_symbol(env, form),
            _ => Err(Box::new(syntax_err("illegal function call"))),
        },
    }
}
