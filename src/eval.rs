use crate::cons::List;
use crate::core;
use crate::core::Env;
use crate::core::LispObject;
use crate::core::LispObjectResult;
use crate::core::Symbol;
use crate::error;

macro_rules! lookup_symbol {
    ($env:ident, $lookup_env:ident, $sym:expr) => {{
        let global = $env.global_env();
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
    function: &core::Function,
    args: List<LispObject>,
    eval_args: bool,
    name_hint: Option<&Symbol>,
) -> LispObjectResult {
    let args = if eval_args {
        args.iter()
            .map(|lo| eval(env.clone(), lo))
            .collect::<Result<List<_>, _>>()?
    } else {
        args
    };

    let has_restarg = function.restarg.is_some();

    if (args.len() < function.arglist.len())
        || (!has_restarg && function.arglist.len() != args.len())
    {
        let render_signature = || {
            let mut arglist = function
                .arglist
                .iter()
                .map(|s| LispObject::Symbol(s.clone()))
                .collect::<Vec<_>>();

            let name_padded = function
                .name
                .as_ref()
                .map_or("".to_string(), |s| format!("{} ", s.name()));

            let body = match function.body {
                core::FunctionBody::Native(_) => "<native code>",
                core::FunctionBody::Interpreted(_) => "...",
            };

            if let Some(ref restarg) = function.restarg {
                arglist.push(LispObject::Symbol(Symbol::new("&")));
                arglist.push(LispObject::Symbol(restarg.clone()));
            }

            let arglist = LispObject::List(List::from_rev_iter(arglist));
            format!("(lambda {}{} {})", name_padded, arglist, body)
        };

        let expected = function.arglist.len();
        let actual = args.len();

        Err(error::ArityError::new(
            expected,
            actual,
            has_restarg,
            name_hint.map(Symbol::name).unwrap_or_else(render_signature),
        ))?
    }

    match function.body {
        core::FunctionBody::Native(ref native_body) => native_body.0(env, args),
        core::FunctionBody::Interpreted(ref interpreted_body) => {
            let mut args = args.iter();

            let mut new_env = env.clone();
            for (sym, val) in function.arglist.iter().zip(args.by_ref()) {
                new_env.cur_env.sym_env.insert(sym.clone(), val.clone());
            }

            if has_restarg {
                let restarg = args.map(|lo| lo.clone()).collect();
                new_env.cur_env.sym_env.insert(
                    function.restarg.as_ref().unwrap().clone(),
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

fn call_symbol(env: Env, form: &LispObject) -> LispObjectResult {
    let form = core::to_list(form)?;
    let sym = core::to_symbol(form.first().unwrap())?;
    let args = form.tail();

    let spec = env.global_env().special_env.get(sym).map(|f| f.clone());

    if let Some(f) = spec {
        f.0(env, args)
    } else if let Some(ref f) = lookup_symbol_function(&env, sym) {
        call_function_object(env, f, args, true, Some(sym))
    } else {
        Err(error::UndefinedSymbol::new(sym.name(), true))?
    }
}

pub fn eval(env: Env, form: &LispObject) -> LispObjectResult {
    match form {
        self_eval @ LispObject::T => Ok(self_eval.clone()),
        self_eval @ LispObject::Integer(_) => Ok(self_eval.clone()),
        self_eval @ LispObject::String(_) => Ok(self_eval.clone()),
        self_eval @ LispObject::Fn(_) => Ok(self_eval.clone()),

        LispObject::List(ref list) if list.is_empty() => Ok(LispObject::nil()),
        LispObject::Symbol(s) => {
            let val = lookup_symbol_value(&env, &s)
                .ok_or_else(|| error::UndefinedSymbol::new(s.name(), false))?;
            Ok(val)
        }
        LispObject::List(ref list) => match list.ufirst() {
            LispObject::Symbol(_) => call_symbol(env, form),
            _ => Err(error::SyntaxError::new("illegal function call"))?,
        },
    }
}
