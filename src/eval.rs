use crate::cons::List;
use crate::env::Env;
use crate::error;
use crate::object;
use crate::object::LispObject;
use crate::object::Symbol;

pub type EvalResult = Result<LispObject, error::ErrorWithStackTrace>;

pub fn call_function_object(
    mut env: Env,
    function: &object::Function,
    args: List<LispObject>,
    eval_args: bool,
    name_hint: Option<&Symbol>,
) -> EvalResult {
    let args = if eval_args {
        args.iter()
            .map(|lo| eval(env.clone(), lo))
            .collect::<Result<List<_>, _>>()?
    } else {
        args
    };

    let has_restarg = function.sig.restarg.is_some();

    if (args.len() < function.sig.arglist.len())
        || (!has_restarg && function.sig.arglist.len() != args.len())
    {
        let expected = function.sig.arglist.len();
        let actual = args.len();

        Err(env.st_err(error::ArityError::new(
            expected,
            actual,
            has_restarg,
            name_hint.map_or_else(|| format!("{}", function.sig), Symbol::name),
        )))?
    }

    match name_hint {
        Some(name) => env.push_stack_frame_name(name.clone()),
        None => env.push_stack_frame_sig(function.sig.clone()),
    }

    match function.body {
        object::FunctionBody::Native(ref native_body) => native_body.0(env, args),
        object::FunctionBody::Interpreted(ref interpreted_body) => {
            let mut args = args.iter();

            let mut new_env = env.clone();
            for (sym, val) in function.sig.arglist.iter().zip(args.by_ref()) {
                new_env.set_local_value(sym.clone(), val.clone());
            }

            if has_restarg {
                let restarg = args.map(|lo| lo.clone()).collect();
                new_env.set_local_value(
                    function.sig.restarg.clone().unwrap(),
                    LispObject::List(restarg),
                );
            }

            let mut result = LispObject::nil();
            for form in interpreted_body.iter() {
                result = eval(new_env.clone(), form)?;
            }

            Ok(result)
        }
    }
}

fn call_symbol(env: Env, form: &LispObject) -> EvalResult {
    let form = env.attach_st(object::to_list(form))?;
    let sym = env.attach_st(object::to_symbol(form.first().unwrap()))?;
    let args = form.tail();

    let spec = env.lookup_symbol_special(sym);

    if let Some(f) = spec {
        f.0(env, args)
    } else if let Some(ref f) = env.lookup_symbol_function(sym) {
        call_function_object(env, f, args, true, Some(sym))
    } else {
        Err(env.st_err(error::UndefinedSymbol::new(sym.name(), true)))?
    }
}

pub fn eval(env: Env, form: &LispObject) -> EvalResult {
    match form {
        self_eval @ LispObject::T => Ok(self_eval.clone()),
        self_eval @ LispObject::Integer(_) => Ok(self_eval.clone()),
        self_eval @ LispObject::String(_) => Ok(self_eval.clone()),
        self_eval @ LispObject::Fn(_) => Ok(self_eval.clone()),

        LispObject::List(ref list) if list.is_empty() => Ok(LispObject::nil()),
        LispObject::Symbol(s) => {
            let val = env
                .lookup_symbol_value(&s)
                .ok_or_else(|| env.st_err(error::UndefinedSymbol::new(s.name(), false)))?;
            Ok(val)
        }
        LispObject::List(ref list) => match list.ufirst() {
            LispObject::Symbol(_) => call_symbol(env, form),
            _ => Err(env.st_err(error::SyntaxError::new("illegal function call")))?,
        },
    }
}
