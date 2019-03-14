use core;
use core::Env;
use core::LispObject;
use core::Symbol;
use error;
use im::Vector;

fn nth(vec: &Vector<LispObject>, i: usize) -> Option<LispObject> {
    vec.get(i).map(|o| o.clone())
}

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

pub fn call_function_object(env: Env, f: &core::Function, args: &Vector<LispObject>, eval_args: bool) -> error::GenResult<LispObject> {
    match f {
        core::Function::NativeFunction(native_fn) => {
            let mut args: Vector<LispObject> = args
                .into_iter()
                .map(|lo| if eval_args { eval(env.clone(), lo) } else { Ok(lo.clone()) })
                .collect::<Result<Vector<_>, _>>()?;

            args.push_front(core::LispObject::Fn(core::Function::NativeFunction(native_fn.clone())));

            native_fn.0(env, &LispObject::Vector(args))
        },
        core::Function::InterpretedFunction(interpreted_fn) => {
            let has_restarg = interpreted_fn.restarg.is_some();

            if (args.len() < interpreted_fn.arglist.len()) || (!has_restarg && interpreted_fn.arglist.len() != args.len()) {
                let expected = interpreted_fn.arglist.len();
                let actual = args.len();
                let mut arglist_as_vec = interpreted_fn
                    .arglist
                    .iter()
                    .map(|s| LispObject::Symbol(s.clone()))
                    .collect::<Vector<_>>();

                if let Some(ref restarg) = interpreted_fn.restarg {
                    arglist_as_vec.push_back(LispObject::Symbol(Symbol::new("&")));
                    arglist_as_vec.push_back(LispObject::Symbol(restarg.clone()));
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
                .map(|lo| if eval_args { eval(env.clone(), lo) } else { Ok(lo.clone()) })
                .collect::<Result<Vector<_>, _>>()?
                .into_iter();

            let mut new_env = env.clone();
            for (sym, val) in interpreted_fn.arglist.iter().zip(args_iter.by_ref()) {
                new_env.cur_env.sym_env.insert(sym.clone(), val);
            }

            if has_restarg {
                let restarg = args_iter.collect();
                new_env
                    .cur_env
                    .sym_env
                    .insert(interpreted_fn.restarg.as_ref().unwrap().clone(), LispObject::Vector(restarg));
            }

            let mut result = LispObject::Nil;
            for form in &interpreted_fn.body {
                result = eval(new_env.clone(), &form)?;
            }

            Ok(result)
        }
    }
}

fn call_fn(env: Env, form: &LispObject) -> error::GenResult<LispObject> {
    let mut form = core::to_vector(form)?;
    let func = core::to_function(&form[0])?;
    call_function_object(env, func, &form.clone().slice(1..), true)
}

fn call_macro(env: Env, form: &LispObject) -> error::GenResult<LispObject> {
    let mut form = core::to_vector(form)?;
    let func = core::to_macro(&form[0])?;
    let expanded = call_function_object(env.clone(), func, &form.clone().slice(1..), false)?;
    eval(env, &expanded)
}

fn call_symbol(env: Env, form: &LispObject) -> error::GenResult<LispObject> {
    let mut form = core::to_vector(form)?;
    let sym = core::to_symbol(&form[0])?;

    let obj = if let Some(&f) = env.global_env.as_ref().borrow().special_env.get(&sym) {
        LispObject::Special(f)
    } else if let Some(f) = lookup_symbol_fn(&env, sym) {
        LispObject::Fn(f)
    } else if let Some(f) = lookup_symbol_macro(&env, &sym) {
        LispObject::Macro(f)
    } else {
        return Err(Box::new(error::UndefinedSymbol::new(
            sym.name(),
            true,
        )));
    };

    let mut form = form.clone();

    form.pop_front();
    form.push_front(obj);
    eval(env, &LispObject::Vector(form))
}

pub fn eval(env: Env, form: &LispObject) -> error::GenResult<LispObject> {
    match form {
        self_eval @ LispObject::Nil => Ok(self_eval.clone()),
        self_eval @ LispObject::T => Ok(self_eval.clone()),
        self_eval @ LispObject::Integer(_) => Ok(self_eval.clone()),
        self_eval @ LispObject::String(_) => Ok(self_eval.clone()),
        self_eval @ LispObject::Fn(_) => Ok(self_eval.clone()),

        LispObject::Special(_) => Err(Box::new(syntax_err("standalone special"))),
        LispObject::Macro(_) => Err(Box::new(syntax_err("standalone macro"))),

        LispObject::Vector(ref vec) if vec.len() == 0 => Ok(LispObject::Vector(vec.clone())),
        LispObject::Symbol(s) => {
            lookup_symbol_value(&env, &s).ok_or(Box::new(error::UndefinedSymbol::new(s.name(), false)))
        }
        LispObject::Vector(ref vec) => match nth(vec, 0).unwrap() {
            LispObject::Symbol(_) => call_symbol(env, form),

            LispObject::Fn(_) => call_fn(env, form),
            LispObject::Macro(_) => call_macro(env, form),
            LispObject::Special(core::NativeFnWrapper(f)) => f(env, form),

            _=> Err(Box::new(syntax_err("illegal function call")))
        },
    }
}
