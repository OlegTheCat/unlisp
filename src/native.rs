use crate::cons::List;
use crate::env;
use crate::error;
use crate::eval;
use crate::eval::EvalResult;
use crate::object;
use crate::object::LispObject;
use crate::object::Symbol;
use std::error::Error;
use std::fmt;
use std::io::Write;

#[derive(Debug)]
struct DummyError;

impl Error for DummyError {}

impl fmt::Display for DummyError {
    fn fmt(&self, _f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        panic!("tried to display DummyError");
    }
}

fn identity_converter(v: &LispObject) -> Result<&LispObject, DummyError> {
    Ok(&v)
}

fn identity(v: LispObject) -> LispObject {
    v
}

macro_rules! define_native_fn {
    ($maker:ident, $id:ident ($env:ident, $( $arg:ident : $converter:path ),*) -> $result_wrap:path $body:block) => {
        #[allow(unused_mut)]
        fn $id( $env: env::Env, args: List<LispObject> ) -> EvalResult {
            let mut args = args.iter();

            $( let $arg = $env.attach_st($converter(args.next().unwrap()))?; )*

            #[allow(unused)]
            let res = $result_wrap($body);
            Ok(res)
        }

        fn $maker(name: impl Into<String>) -> object::Function {
            let name = Some(Symbol::new(name));
            let args = List::from_rev_iter(vec![$( Symbol::new(stringify!($arg)), )*]);
            object::Function::new_native(name, args , None, object::NativeFnWrapper($id))
        }

    };

    ($maker:ident, $id:ident ($env:ident, $( $arg:ident : $converter:path, )* ... $vararg:ident : $vconverter:path ) -> $result_wrap:path $body:block) => {
        #[allow(unused_mut)]
        fn $id( $env: env::Env, args: List<LispObject> ) -> EvalResult {
            let mut args = args.iter();

            $(  let $arg = $env.attach_st($converter(args.next().unwrap()))?; )*

            let $vararg = args
                .map(|lo| $env.attach_st($vconverter(lo)))
                .collect::<Result<List<_>, _>>()?;

            let res = $result_wrap($body);
            Ok(res)
        }

        fn $maker(name: impl Into<String>) -> object::Function {
            let name = Some(Symbol::new(name));
            let args = List::from_rev_iter(vec![$( Symbol::new(stringify!($arg)), )*]);
            let restarg = Some(Symbol::new(stringify!($vararg)));
            object::Function::new_native(name, args , restarg, object::NativeFnWrapper($id))
        }
    }
}

define_native_fn! {
    make_cons,
    native_cons(_env, item: identity_converter, list: object::to_list) -> LispObject::List {
        list.cons(item.clone())
    }

}

fn native_bool_to_lisp_bool(b: bool) -> LispObject {
    if b {
        LispObject::T
    } else {
        LispObject::nil()
    }
}

define_native_fn! {
    make_stdout_write,
    native_stdout_write(env, s: object::to_string) -> identity {
        env.attach_st(write!(std::io::stdout(), "{}", s))?;
        LispObject::nil()
    }
}

define_native_fn! {
    make_print,
    native_print(_env, x: identity_converter) -> identity {
        print!("{}", &x);
        x.clone()
    }
}

define_native_fn! {
    make_println,
    native_println(_env, x: identity_converter) -> identity {
        println!("{}", &x);
        x.clone()
    }
}

fn native_apply(env: env::Env, args: List<LispObject>) -> EvalResult {
    let f = env.attach_st(object::to_function(args.first().unwrap()))?;

    let args = args.tail();
    let mut args_iter = args.rc_iter();

    let unspliced = args_iter.by_ref().take(args.len() - 1).collect::<Vec<_>>();

    let last_arg = args_iter.next().unwrap();
    let last_arg = env.attach_st(object::to_list(last_arg.as_ref()))?;
    let mut args = last_arg.clone();

    for x in unspliced.into_iter().rev() {
        args = args.cons_rc(x);
    }

    eval::call_function_object(env, f, args, false, None)
}

fn make_apply(name: impl Into<String>) -> object::Function {
    object::Function::new_native(
        Some(Symbol::new(name)),
        List::empty()
            .cons(Symbol::new("fn"))
            .cons(Symbol::new("arg")),
        Some(Symbol::new("args")),
        object::NativeFnWrapper(native_apply),
    )
}

define_native_fn! {
    make_add,
    native_add(_env, ... args: object::to_i64) -> LispObject::Integer {
        let mut res = 0;
        for arg in args.iter() {
            res += *arg
        }
        res
    }
}

define_native_fn! {
    make_sub,
    native_sub(_env, from: object::to_i64, ... args: object::to_i64) -> LispObject::Integer {
        let mut res = *from;

        if args.is_empty() {
            res = -res;
        } else {
            for arg in args.iter() {
                res -= *arg
            }
        }

        res
    }
}

define_native_fn! {
    make_mul,
    native_mul(_env, ... args: object::to_i64) -> LispObject::Integer {
        let mut res = 1;
        for arg in args.iter() {
            res *= *arg
        }
        res
    }
}

define_native_fn! {
    make_lt,
    native_lt(_env, x: object::to_i64, y: object::to_i64) -> identity {
        native_bool_to_lisp_bool(x < y)
    }
}

define_native_fn! {
    make_gt,
    native_gt(_env, x: object::to_i64, y: object::to_i64) -> identity {
        native_bool_to_lisp_bool(x > y)
    }
}

define_native_fn! {
    make_equal,
    native_equal(_env, x: identity_converter, y: identity_converter) -> identity {
        native_bool_to_lisp_bool(*x == *y)
    }
}

define_native_fn! {
    make_first,
    native_first(env, list: object::to_list) -> identity {
        let first = list.first()
            .ok_or_else(|| env.st_err(error::GenericError::new(
                "cannot do first on empty list")))?;
        first.clone()
    }
}

define_native_fn! {
    make_rest,
    native_rest(_env, list: object::to_list) -> LispObject::List {
        list.tail()
    }
}

define_native_fn! {
    make_listp,
    native_listp(_env, arg: identity_converter) -> identity {
        let converted = object::to_list(arg);
        native_bool_to_lisp_bool(converted.is_ok())
    }
}

define_native_fn! {
    make_emptyp,
    native_emptyp(_env, arg: object::to_list) -> identity {
        native_bool_to_lisp_bool(arg.is_empty())
    }
}

define_native_fn! {
    make_symbolp,
    native_symbolp(_env, arg: identity_converter) -> identity {
        let converted = object::to_symbol(arg);
        native_bool_to_lisp_bool(converted.is_ok())
    }
}

define_native_fn! {
    make_macroexpand,
    native_macroexpand(env, arg: identity_converter) -> identity {
        match arg {
            LispObject::List(ref list) if !list.is_empty() => {
                match list.ufirst() {
                    LispObject::Symbol(s) => {
                        env.lookup_symbol_macro(s)
                            .map_or_else(|| Ok(arg.clone()),
                                         |macro_fn| eval::call_function_object(env, &macro_fn, list.tail(), false, Some(s)))?
                    }
                    _ => arg.clone()
                }
            }
            _ => arg.clone()
        }
    }
}

define_native_fn! {
    make_symbol_function,
    native_symbol_function(env, arg: object::to_symbol) -> identity {
        let f = env
            .lookup_symbol_function(&arg)
            .ok_or_else(|| env.st_err(error::UndefinedSymbol::new(arg.name(), true)))?;
        LispObject::Fn(f)
    }
}

define_native_fn! {
    make_raise_error,
    native_raise_error(env, arg: object::to_string) -> identity {
        let mut err = env.st_err(error::GenericError::new(arg.clone()));

        // drop one frame, so error function is not present in stact trace
        err.stack_trace = err.stack_trace.tail();
        return Err(err);
    }
}

pub fn prepare_natives(env: &mut env::Env) {
    let mut save = |name: &str, maker: fn(String) -> object::Function| {
        env.set_global_function(Symbol::new(name), maker(name.to_string()));
    };

    save("cons", make_cons);
    save("first", make_first);
    save("rest", make_rest);
    save("equal", make_equal);
    save("apply", make_apply);

    save("+", make_add);
    save("-", make_sub);
    save("*", make_mul);
    save("<", make_lt);
    save(">", make_gt);

    save("listp", make_listp);
    save("emptyp", make_emptyp);
    save("symbolp", make_symbolp);

    save("print", make_print);
    save("println", make_println);
    save("stdout-write", make_stdout_write);

    save("macroexpand-1", make_macroexpand);

    save("error", make_raise_error);
    save("symbol-function", make_symbol_function);
}

#[cfg(test)]
mod tests {
    use crate::object;
    use crate::error;
    use crate::test_utils::*;

    fn ctx() -> Context {
        Context::new(true, true, false)
    }

    #[test]
    fn test_cons() {
        let ctx = ctx();

        assert_err!(ctx, "(cons)", error::ArityError);
        assert_err!(ctx, "(cons 1)", error::ArityError);

        assert_ok!(ctx, "(cons 1 nil)", "(1)");
        assert_ok!(ctx, "(cons 1 ())", "(1)");
        assert_ok!(ctx, "(cons 1 (quote (2 3)))", "(1 2 3)");
        assert_ok!(ctx, "(cons (quote (1 2 3)) (quote (2 3)))", "((1 2 3) 2 3)");
    }

    #[test]
    fn test_first() {
        let ctx = ctx();

        assert_err!(ctx, "(first)", error::ArityError);
        assert_err!(ctx, "(first nil)", error::GenericError);

        assert_ok!(ctx, "(first (quote (1)))", "1");
        assert_ok!(ctx, "(first (quote (1 2)))", "1");
        assert_ok!(ctx, "(first (quote ((1 2 3) 2)))", "(1 2 3)");
    }

    #[test]
    fn test_rest() {
        let ctx = ctx();
        assert_err!(ctx, "(rest)", error::ArityError);

        assert_ok!(ctx, "(rest nil)", "nil");
        assert_ok!(ctx, "(rest (quote (1)))", "nil");
        assert_ok!(ctx, "(rest (quote (1 (1 2 3) 4 5)))", "((1 2 3) 4 5)");
    }

    #[test]
    fn test_equal() {
        let ctx = ctx();
        assert_err!(ctx, "(equal)", error::ArityError);
        assert_err!(ctx, "(equal 1)", error::ArityError);

        assert_ok!(ctx, "(equal 1 1)", "t");
        assert_ok!(ctx, "(equal 1 2)", "nil");
        assert_ok!(ctx, "(equal 1 (quote foo))", "nil");
        assert_ok!(ctx, "(equal (quote foo) (quote foo))", "t");
        assert_ok!(ctx, "(equal (quote (x y (z 1))) (quote (x y (z 1))))", "t");
        assert_ok!(
            ctx,
            "(equal (quote (x y (z 1))) (quote (x y (z 2))))",
            "nil"
        );
    }

    #[test]
    fn test_apply() {
        let ctx = ctx();

        assert_err!(ctx, "(apply)", error::ArityError);
        assert_err!(
            ctx,
            "(set-fn x (lambda ())) (apply (symbol-function (quote x)))",
            error::ArityError
        );
        assert_err!(
            ctx,
            "(set-fn x (lambda ())) (apply (symbol-function (quote x)) (quote (1 2)))",
            error::ArityError
        );

        assert_ok!(ctx, "(apply (symbol-function (quote +)) (quote (1 2)))", "3");
        assert_ok!(ctx, "(apply (symbol-function (quote +)) (quote (1 2 5)))", "8");
        assert_ok!(
            ctx,
            "(apply (symbol-function (quote cons)) 1 (quote ((2))))",
            "(1 2)"
        );
        assert_ok!(
            ctx,
            "(apply (symbol-function (quote apply)) (symbol-function (quote +)) 1 (quote ((2))))",
            "3"
        );
    }

    #[test]
    fn test_add() {
        let ctx = ctx();

        assert_ok!(ctx, "(+)", "0");
        assert_ok!(ctx, "(+ 1)", "1");
        assert_ok!(ctx, "(+ 1 2)", "3");
        assert_ok!(ctx, "(+ 1 2 3 4 5)", "15");
    }

    #[test]
    fn test_sub() {
        let ctx = ctx();

        assert_err!(ctx, "(-)", error::ArityError);

        assert_ok!(ctx, "(+ (- 1) 1)", "0");
        assert_ok!(ctx, "(+ 1 (- 1 2))", "0");
        assert_ok!(ctx, "(+ 13 (- 1 2 3 4 5))", "0");
    }

    #[test]
    fn test_mul() {
        let ctx = ctx();

        assert_ok!(ctx, "(*)", "1");
        assert_ok!(ctx, "(* 1)", "1");
        assert_ok!(ctx, "(* 1 2)", "2");
        assert_ok!(ctx, "(* 1 2 3 4 5)", "120");
    }

    #[test]
    fn test_lt() {
        let ctx = ctx();
        assert_err!(ctx, "(<)", error::ArityError);
        assert_err!(ctx, "(< 1)", error::ArityError);
        assert_err!(ctx, "(< 1 (quote x))", error::CastError);

        assert_ok!(ctx, "(< 1 2)", "t");
        assert_ok!(ctx, "(< 2 1)", "nil");
    }

    #[test]
    fn test_gt() {
        let ctx = ctx();
        assert_err!(ctx, "(>)", error::ArityError);
        assert_err!(ctx, "(> 1)", error::ArityError);
        assert_err!(ctx, "(> 1 (quote x))", error::CastError);

        assert_ok!(ctx, "(> 1 2)", "nil");
        assert_ok!(ctx, "(> 2 1)", "t");
    }

    #[test]
    fn test_listp() {
        let ctx = ctx();
        assert_err!(ctx, "(listp)", error::ArityError);

        assert_ok!(ctx, "(listp (quote 1))", "nil");
        assert_ok!(ctx, "(listp nil)", "t");
        assert_ok!(ctx, "(listp (quote (1 2 3)))", "t");
    }

    #[test]
    fn test_emptyp() {
        let ctx = ctx();
        assert_err!(ctx, "(emptyp)", error::ArityError);

        assert_ok!(ctx, "(emptyp nil)", "t");
        assert_ok!(ctx, "(emptyp ())", "t");
        assert_ok!(ctx, "(emptyp (quote (1 2 3)))", "nil");
    }

    #[test]
    fn test_symbolp() {
        let ctx = ctx();
        assert_err!(ctx, "(symbolp)", error::ArityError);

        assert_ok!(ctx, "(symbolp (quote x))", "t");

        assert_ok!(ctx, "(symbolp 1)", "nil");

        // TODO: is this ok?
        assert_ok!(ctx, "(symbolp nil)", "nil");
        assert_ok!(ctx, "(symbolp t)", "nil");
    }

    #[test]
    fn test_macroexpand_1() {
        let ctx = ctx();
        assert_err!(ctx, "(macroexpand-1)", error::ArityError);

        assert_ok!(ctx, "(set-macro-fn x (lambda (arg) (if arg (quote x) (quote y)))) (macroexpand-1 (quote (x t)))", "x");
        assert_ok!(ctx, "(set-macro-fn x (lambda (arg) (if arg (quote x) (quote y)))) (macroexpand-1 (quote (x nil)))", "y");
        assert_ok!(ctx, "(macroexpand-1 (quote (cons 1 nil)))", "(cons 1 nil)");
    }

    #[test]
    fn test_error() {
        let ctx = ctx();
        assert_err!(ctx, "(error)", error::ArityError);
        assert_err!(ctx, "(error 1)", error::CastError);

        assert_err!(ctx, "(error \"foo\")", error::GenericError);
    }

    #[test]
    fn test_symbol_function() {
        let ctx = ctx();
        assert_err!(ctx, "(symbol-function)", error::ArityError);
        assert_err!(ctx, "(symbol-function 1)", error::CastError);

        assert!(object::to_function(
            &ctx.ok_eval("(set-fn foo (lambda (x) x)) (symbol-function (quote foo))")
        )
                .is_ok());
        assert_ok!(
            ctx,
            "(set-fn foo (lambda (x) x)) (set-fn bar (symbol-function (quote foo))) (bar 1)",
            "1"
        );
    }


    #[test]
    fn test_higher_order_funcs() {
        let ctx = ctx();
        assert_ok!(
            ctx,
            "(set-fn ho (lambda (f) (set-fn f f) (f 5)))
             (set-fn foo (lambda (x) x))
             (ho (symbol-function (quote foo)))",
            "5"
        );
    }

}
