use crate::cons::List;
use crate::env::Env;
use crate::error::*;
use crate::eval::{eval, EvalResult};
use crate::object;
use crate::object::LispObject;
use crate::object::Symbol;
use std::error::Error;

type GenResult<T> = Result<T, Box<Error>>;

pub struct ParsedQuote(pub LispObject);

pub fn parse_quote(args: &List<LispObject>) -> GenResult<ParsedQuote> {
    if args.len() != 1 {
        Err(ArityError::new(1, args.len(), false, "quote".to_string()))?
    }

    Ok(ParsedQuote(args.ufirst().clone()))
}

fn quote_form(env: Env, args: List<LispObject>) -> EvalResult {
    Ok(env.attach_st_box(parse_quote(&args))?.0)
}

pub struct ParsedLet<'a> {
    pub bindings: Vec<(Symbol, &'a LispObject)>,
    pub body: List<LispObject>,
}

pub fn parse_let<'a>(args: &'a List<LispObject>) -> GenResult<ParsedLet<'a>> {
    let bindings = args
        .first()
        .ok_or_else(|| (SyntaxError::new("no bindings in let")))?;
    let bindings =
        object::to_list(bindings).map_err(|_e| SyntaxError::new("let bindings are not a list"))?;

    let mut collected_bindings = vec![];

    for binding in bindings.iter() {
        let binding =
            object::to_list(binding).map_err(|_e| SyntaxError::new("let binding is not a list"))?;
        let mut binding_iter = binding.iter();
        let sym = binding_iter
            .next()
            .ok_or_else(|| SyntaxError::new("empty binding clause"))?;
        let sym = object::to_symbol(sym)
            .map_err(|_e| SyntaxError::new("not a symbol in binding clause"))?;

        let val_form = binding_iter
            .next()
            .ok_or_else(|| SyntaxError::new("no value in binding clause"))?;

        collected_bindings.push((sym.clone(), val_form));
    }

    Ok(ParsedLet {
        bindings: collected_bindings,
        body: args.tail(),
    })
}

fn let_form(env: Env, args: List<LispObject>) -> EvalResult {
    let ParsedLet { bindings, body } = env.attach_st_box(parse_let(&args))?;

    let mut new_env = env;

    for (sym, val_form) in bindings {
        let val = eval(new_env.clone(), val_form)?;
        new_env.set_local_value(sym, val);
    }

    let mut res = LispObject::nil();

    for form in body.iter() {
        res = eval(new_env.clone(), &form)?;
    }

    Ok(res)
}

pub struct ParsedLambda {
    pub name: Option<Symbol>,
    pub simple_args: List<Symbol>,
    pub restarg: Option<Symbol>,
    pub body: List<LispObject>,
}

fn parse_arglist(arglist: Vec<Symbol>) -> GenResult<(List<Symbol>, Option<Symbol>)> {
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
            Err(SyntaxError::new("wrong syntax near '&' in lambda"))?
        } else {
            restargs.into_iter().next()
        }
    };

    Ok((simple_args, restarg))
}

pub fn parse_lambda(args: &List<LispObject>) -> GenResult<ParsedLambda> {
    let no_arglist = || SyntaxError::new("no arglist in lambda");

    let name_or_arglist = args.first().ok_or_else(no_arglist)?;

    let mut name = None;
    let arglist;
    let body;

    if let Ok(sym) = object::to_symbol(name_or_arglist) {
        name = Some(sym.clone());
        arglist = args.iter().nth(1).ok_or_else(no_arglist)?;
        body = args.tailn(2);
    } else {
        arglist = name_or_arglist;
        body = args.tail()
    }

    let arglist =
        object::to_list(arglist).map_err(|_e| SyntaxError::new("lambda arglist in not a list"))?;
    let arglist = arglist
        .iter()
        .map(|lo| {
            object::to_symbol(lo)
                .map(|s| s.clone())
                .map_err(|_e| SyntaxError::new("expected symbol in arglist"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let (simple_args, restarg) = parse_arglist(arglist)?;

    Ok(ParsedLambda {
        name: name,
        simple_args: simple_args,
        restarg: restarg,
        body: body,
    })
}

fn lambda_form(env: Env, args: List<LispObject>) -> EvalResult {
    let ParsedLambda {
        name,
        simple_args,
        restarg,
        body,
    } = env.attach_st_box(parse_lambda(&args))?;

    Ok(LispObject::Fn(object::Function::new_interpreted(
        name,
        simple_args,
        restarg,
        body,
    )))
}

fn set_fn(mut env: Env, args: List<LispObject>) -> EvalResult {
    let mut args = args.iter();
    let sym = args
        .next()
        .ok_or_else(|| env.st_err(SyntaxError::new("no symbol in set-fn")))?;
    let sym = object::to_symbol(sym)
        .map_err(|_| env.st_err(SyntaxError::new("not a symbol in set-fn")))?;

    let func = args
        .next()
        .ok_or_else(|| env.st_err(SyntaxError::new("no function in set-fn")))?;
    let func = env.attach_st(object::to_function_owned(eval(env.clone(), &func)?))?;

    env.set_global_function(sym.clone(), func);
    Ok(LispObject::nil())
}

fn set_macro_fn(mut env: Env, args: List<LispObject>) -> EvalResult {
    let mut args = args.iter();
    let sym = args
        .next()
        .ok_or_else(|| env.st_err(SyntaxError::new("no symbol in set-macro-fn")))?;
    let sym = object::to_symbol(sym)
        .map_err(|_| env.st_err(SyntaxError::new("not a symbol in set-macro-fn")))?;

    let func = args
        .next()
        .ok_or_else(|| env.st_err(SyntaxError::new("no function in set-macro-fn")))?;
    let func = env.attach_st(object::to_function_owned(eval(env.clone(), &func)?))?;

    env.set_global_macro(sym.clone(), func);
    Ok(LispObject::nil())
}

fn if_form(env: Env, args: List<LispObject>) -> EvalResult {
    let mut args = args.iter();
    let cond = args
        .next()
        .ok_or_else(|| env.st_err(SyntaxError::new("no condition in if")))?;
    let then_form = args
        .next()
        .ok_or_else(|| env.st_err(SyntaxError::new("no then in if")))?;
    let nil = LispObject::nil();
    let else_form = args.next().unwrap_or(&nil);

    let cond = eval(env.clone(), cond)?;
    if cond == nil {
        eval(env, else_form)
    } else {
        eval(env, then_form)
    }
}

pub fn prepare_specials(env: &mut Env) {
    let mut set = |s: &str, f| {
        env.set_global_special(Symbol::new(s), object::NativeFnWrapper(f));
    };

    set("quote", quote_form);
    set("if", if_form);
    set("let", let_form);
    set("set-fn", set_fn);
    set("set-macro-fn", set_macro_fn);
    set("lambda", lambda_form);
}

#[cfg(test)]
mod tests {
    use crate::error;
    use crate::object;
    use crate::test_utils::*;

    fn ctx() -> Context {
        Context::new(true, false, false)
    }

    #[test]
    fn test_quote() {
        let ctx = ctx();
        assert_err!(ctx, "(quote)", error::ArityError);

        assert_ok!(ctx, "(quote 1)", "1");
        assert_ok!(ctx, "(quote \"foo\")", "\"foo\"");
        assert_ok!(ctx, "(quote (add 1 2))", "(add 1 2)");
    }

    #[test]
    fn test_if() {
        let ctx = ctx();
        assert_err!(ctx, "(if)", error::SyntaxError);
        assert_err!(ctx, "(if t)", error::SyntaxError);

        assert_ok!(ctx, "(if t 1 2)", "1");
        assert_ok!(ctx, "(if nil 1 2)", "2");
        assert_ok!(ctx, "(if nil 1)", "nil");
        assert_ok!(ctx, "(if (quote (foo bar)) 1 2)", "1");
        assert_ok!(ctx, "(if (quote ()) 1 2)", "2");
    }

    #[test]
    fn test_lambda_syntax() {
        let ctx = ctx();
        assert_err!(ctx, "(lambda)", error::SyntaxError);
        assert_err!(ctx, "(lambda 1)", error::SyntaxError);
        assert_err!(ctx, "(lambda (1))", error::SyntaxError);
        assert_err!(ctx, "(lambda foo)", error::SyntaxError);
        assert_err!(ctx, "(lambda foo (1))", error::SyntaxError);

        // lambda behavior is tested in test_set_fn
        assert!(object::to_function(&ctx.ok_eval("(lambda (x) x)")).is_ok());
        assert!(object::to_function(&ctx.ok_eval("(lambda foo (x) x)")).is_ok());
    }

    #[test]
    fn test_set_fn() {
        let ctx = ctx();
        assert_err!(ctx, "(set-fn)", error::SyntaxError);
        assert_err!(ctx, "(set-fn 1)", error::SyntaxError);
        assert_err!(ctx, "(set-fn foo)", error::SyntaxError);
        assert_err!(ctx, "(set-fn foo 2)", error::CastError);

        assert_ok!(ctx, "(set-fn x (lambda () (quote x))) (x)", "x");
        assert_ok!(ctx, "(set-fn x (lambda (y) (if y 1 2))) (x t)", "1");
        assert_ok!(ctx, "(set-fn x (lambda (y) (if y 1 2))) (x nil)", "2");
    }

    #[test]
    fn test_let() {
        let ctx = ctx();
        assert_err!(ctx, "(let)", error::SyntaxError);
        assert_err!(ctx, "(let 1)", error::SyntaxError);
        assert_err!(ctx, "(let (x))", error::SyntaxError);
        assert_err!(ctx, "(let ((1 1)))", error::SyntaxError);

        assert_ok!(ctx, "(let ())", "nil");
        assert_ok!(ctx, "(let ((x 1)) x)", "1");
        assert_ok!(ctx, "(let ((x 1) (y (if x 2 3))) y)", "2");
        assert_ok!(ctx, "(let ((x nil) (y (if x 2 3))) y)", "3");
        assert_ok!(ctx, "(let ((x nil) (y (let ((x t)) (if x 2 3)))) y)", "2");
    }

    #[test]
    fn test_set_macro_fn() {
        let ctx = ctx();
        assert_err!(ctx, "(set-macro-fn)", error::SyntaxError);
        assert_err!(ctx, "(set-macro-fn 1)", error::SyntaxError);
        assert_err!(ctx, "(set-macro-fn foo)", error::SyntaxError);
        assert_err!(ctx, "(set-macro-fn foo 2)", error::CastError);

        assert_ok!(ctx, "(set-macro-fn x (lambda () 1)) (x)", "1");
        assert_ok!(
            ctx,
            "(set-macro-fn x (lambda () (quote (let ((x 1)) x)))) (x)",
            "1"
        );
        assert_ok!(
            ctx,
            "(set-macro-fn x (lambda (y)
                               (if y
                                   (quote (let ((x 1)) x))
                                   (quote (let ((x 2)) x)))))
                  (x t)",
            "1"
        );
        assert_ok!(
            ctx,
            "(set-macro-fn x (lambda (y)
                               (if y
                                   (quote (let ((x 1)) x))
                                   (quote (let ((x 2)) x)))))
                 (x nil)",
            "2"
        );
    }
}
