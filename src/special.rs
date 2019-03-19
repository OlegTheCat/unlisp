use cons::List;
use core;
use core::Env;
use core::LispObject;
use core::Symbol;
use error;
use eval;
use eval::eval;

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
    pub body: List<LispObject>,
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
        let sym = binding_iter
            .next()
            .ok_or(syntax_err("empty binding clause"))?;
        let sym =
            core::to_symbol(sym).map_err(|_e| syntax_err("not a symbol in binding clause"))?;

        let val_form = binding_iter
            .next()
            .ok_or(syntax_err("no value in binding clause"))?;

        collected_bindings.push((sym.clone(), val_form));
    }

    Ok(ParsedLet {
        bindings: collected_bindings,
        body: args.tail(),
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
    pub body: List<LispObject>,
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
        body: body,
    })
}

fn lambda_form(_env: Env, args: List<LispObject>) -> error::GenResult<LispObject> {
    let ParsedLambda {
        simple_args,
        restarg,
        body,
    } = parse_lambda(&args)?;

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
    let f = eval::lookup_symbol_function(&env, &arg)
        .ok_or(error::UndefinedSymbol::new(arg.name(), true))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::DerefMut;
    use macroexpand::macroexpand_all;
    use reader::Reader;
    use std::io;

    fn eval(s: impl Into<String>) -> error::GenResult<LispObject> {
        let env = Env::new();
        prepare_specials(env.global_env.borrow_mut().deref_mut());
        let s = s.into();
        let mut bytes = s.as_bytes();
        let mut reader = Reader::create(&mut bytes);

        let mut res = Ok(LispObject::nil());
        loop {
            match reader.read_form() {
                Ok(form) => {
                    res = super::eval(env.clone(), &macroexpand_all(env.clone(), &form)?);
                }

                Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(Box::new(e))
            }
        }

        res
    }

    fn read(s: impl Into<String>) -> LispObject {
        let s = s.into();
        let mut bytes = s.as_bytes();
        let mut reader = Reader::create(&mut bytes);

        reader.read_form().unwrap()
    }

    #[test]
    fn test_quote() {
        assert!(eval("(quote)").unwrap_err().downcast::<error::ArityError>().is_ok());

        assert_eq!(eval("(quote 1)").unwrap(), read("1"));
        assert_eq!(eval("(quote \"foo\")").unwrap(), read("\"foo\""));
        assert_eq!(eval("(quote (add 1 2))").unwrap(), read("(add 1 2)"));
    }

    #[test]
    fn test_if() {
        assert!(eval("(if)").unwrap_err().downcast::<error::SyntaxError>().is_ok());
        assert!(eval("(if t)").unwrap_err().downcast::<error::SyntaxError>().is_ok());

        assert_eq!(eval("(if t 1 2)").unwrap(), read("1"));
        assert_eq!(eval("(if nil 1 2)").unwrap(), read("2"));
        assert_eq!(eval("(if nil 1)").unwrap(), read("nil"));
        assert_eq!(eval("(if (quote (foo bar)) 1 2)").unwrap(), read("1"));
        assert_eq!(eval("(if (quote ()) 1 2)").unwrap(), read("2"));
    }

    #[test]
    fn test_lambda_syntax() {
        assert!(eval("(lambda)").unwrap_err().downcast::<error::SyntaxError>().is_ok());
        assert!(eval("(lambda 1)").unwrap_err().downcast::<error::SyntaxError>().is_ok());
        assert!(eval("(lambda (1))").unwrap_err().downcast::<error::SyntaxError>().is_ok());

        // lambda behavior is tested in test_set_fn
        assert!(core::to_function(&eval("(lambda (x) x)").unwrap()).is_ok());
    }

    #[test]
    fn test_set_fn() {
        assert!(eval("(set-fn)").unwrap_err().downcast::<error::SyntaxError>().is_ok());
        assert!(eval("(set-fn 1)").unwrap_err().downcast::<error::SyntaxError>().is_ok());
        assert!(eval("(set-fn foo)").unwrap_err().downcast::<error::SyntaxError>().is_ok());
        assert!(eval("(set-fn foo 2)").unwrap_err().downcast::<error::CastError>().is_ok());

        assert_eq!(eval("(set-fn x (lambda () (quote x))) (x)").unwrap(), read("x"));
        assert_eq!(eval("(set-fn x (lambda (y) (if y 1 2))) (x t)").unwrap(), read("1"));
        assert_eq!(eval("(set-fn x (lambda (y) (if y 1 2))) (x nil)").unwrap(), read("2"));
    }

    #[test]
    fn test_let() {
        assert!(eval("(let)").unwrap_err().downcast::<error::SyntaxError>().is_ok());
        assert!(eval("(let 1)").unwrap_err().downcast::<error::SyntaxError>().is_ok());
        assert!(eval("(let (x))").unwrap_err().downcast::<error::SyntaxError>().is_ok());
        assert!(eval("(let ((1 1)))").unwrap_err().downcast::<error::SyntaxError>().is_ok());


        assert_eq!(eval("(let ())").unwrap(), read("nil"));
        assert_eq!(eval("(let ((x 1)) x)").unwrap(), read("1"));
        assert_eq!(eval("(let ((x 1) (y (if x 2 3))) y)").unwrap(), read("2"));
        assert_eq!(eval("(let ((x nil) (y (if x 2 3))) y)").unwrap(), read("3"));
        assert_eq!(eval("(let ((x nil) (y (let ((x t)) (if x 2 3)))) y)").unwrap(), read("2"));

    }

    #[test]
    fn test_set_macro_fn() {
        assert!(eval("(set-macro-fn)").unwrap_err().downcast::<error::SyntaxError>().is_ok());
        assert!(eval("(set-macro-fn 1)").unwrap_err().downcast::<error::SyntaxError>().is_ok());
        assert!(eval("(set-macro-fn foo)").unwrap_err().downcast::<error::SyntaxError>().is_ok());
        assert!(eval("(set-macro-fn foo 2)").unwrap_err().downcast::<error::CastError>().is_ok());

        assert_eq!(eval("(set-macro-fn x (lambda () 1)) (x)").unwrap(), read("1"));
        assert_eq!(eval("(set-macro-fn x (lambda () (quote (let ((x 1)) x)))) (x)").unwrap(), read("1"));
        assert_eq!(eval("(set-macro-fn x (lambda (y) (if y (quote (let ((x 1)) x))
                                                         (quote (let ((x 2)) x))))) (x t)").unwrap(), read("1"));
        assert_eq!(eval("(set-macro-fn x (lambda (y) (if y (quote (let ((x 1)) x))
                                                         (quote (let ((x 2)) x))))) (x nil)").unwrap(), read("2"));
    }

    #[test]
    fn test_symbol_function() {
        assert!(eval("(symbol-function)").unwrap_err().downcast::<error::SyntaxError>().is_ok());
        assert!(eval("(symbol-function 1)").unwrap_err().downcast::<error::CastError>().is_ok());

        assert!(core::to_function(&eval("(set-fn foo (lambda (x) x)) (symbol-function foo)").unwrap()).is_ok());
        assert_eq!(eval("(set-fn foo (lambda (x) x)) (set-fn bar (symbol-function foo)) (bar 1)").unwrap(), read("1"));
    }

    #[test]
    fn test_error() {
        assert!(eval("(error)").unwrap_err().downcast::<error::SyntaxError>().is_ok());
        assert!(eval("(error 1)").unwrap_err().downcast::<error::CastError>().is_ok());

        assert!(eval("(error \"foo\")").unwrap_err().downcast::<error::GenericError>().is_ok());
    }

    #[test]
    fn test_higher_order_funcs() {
        assert_eq!(eval("(set-fn ho (lambda (f) (set-fn f f) (f 5))) (set-fn foo (lambda (x) x)) (ho (symbol-function foo))").unwrap(), read("5"));
    }
}
