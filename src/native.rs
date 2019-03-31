use crate::cons::List;
use crate::core;
use crate::core::LispObject;
use crate::core::LispObjectResult;
use crate::core::Symbol;
use crate::error;
use crate::eval;
use std::io::Write;

fn identity_converter(v: &LispObject) -> error::GenResult<&LispObject> {
    Ok(&v)
}

fn identity(v: LispObject) -> LispObject {
    v
}

macro_rules! define_native_fn {
    ($maker:ident, $id:ident ($env:ident, $( $arg:ident : $converter:path ),*) -> $result_wrap:path $body:block) => {
        #[allow(unused_mut)]
        fn $id( $env: core::Env, args: List<LispObject> ) -> LispObjectResult {
            let mut args = args.iter();

            $( let mut $arg = $converter(args.next().unwrap())?; )*

            let res = $result_wrap($body);
            Ok(res)
        }

        fn $maker() -> core::Function {
            let args = List::from_rev_iter(vec![$( Symbol::new(stringify!($arg)), )*]);
            core::Function::new_native(args , None, core::NativeFnWrapper($id))
        }

    };

    ($maker:ident, $id:ident ($env:ident, $( $arg:ident : $converter:path, )* ... $vararg:ident : $vconverter:path ) -> $result_wrap:path $body:block) => {
        #[allow(unused_mut)]
        fn $id( $env: core::Env, args: List<LispObject> ) -> LispObjectResult {

            let mut args = args.iter();

            $( #[allow(unused_mut)] let mut $arg = $converter(args.next().unwrap())?; )*

            let mut $vararg: List<_> = args
                .map(|lo| $vconverter(lo))
                .collect::<Result<List<_>, _>>()?;

            let res = $result_wrap($body);
            Ok(res)
        }

        fn $maker() -> core::Function {
            let args = List::from_rev_iter(vec![$( Symbol::new(stringify!($arg)), )*]);
            let restarg = Some(Symbol::new(stringify!($vararg)));
            core::Function::new_native(args , restarg, core::NativeFnWrapper($id))
        }
    }
}

define_native_fn! {
    make_cons,
    native_cons(_env, item: identity_converter, list: core::to_list) -> LispObject::List {
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
    native_stdout_write(_env, s: core::to_string) -> identity {
        write!(std::io::stdout(), "{}", s)?;
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

fn native_apply(env: core::Env, args: List<LispObject>) -> LispObjectResult {
    if args.len() <= 1 {
        Err(error::ArityError::new(2, 1, true, "apply".to_string()))?
    }

    let f = core::to_function(args.first().unwrap())?;

    let args = args.tail();
    let mut args_iter = args.rc_iter();

    let unspliced = args_iter.by_ref().take(args.len() - 1).collect::<Vec<_>>();

    let last_arg = args_iter.next().unwrap();
    let last_arg = core::to_list(last_arg.as_ref())?;
    let mut args = last_arg.clone();

    for x in unspliced.into_iter().rev() {
        args = args.cons_rc(x);
    }

    eval::call_function_object(env, f, args, false, None)
}

fn make_apply() -> core::Function {
    core::Function::new_native(
        List::empty().cons(Symbol::new("fn")),
        Some(Symbol::new("args")),
        core::NativeFnWrapper(native_apply),
    )
}

define_native_fn! {
    make_add,
    native_add(_env, ... args: core::to_i64) -> LispObject::Integer {
        let mut res = 0;
        for arg in args.iter() {
            res += *arg
        }
        res
    }
}

define_native_fn! {
    make_sub,
    native_sub(_env, from: core::to_i64, ... args: core::to_i64) -> LispObject::Integer {
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
    native_mul(_env, ... args: core::to_i64) -> LispObject::Integer {
        let mut res = 1;
        for arg in args.iter() {
            res *= *arg
        }
        res
    }
}

define_native_fn! {
    make_lt,
    native_lt(_env, x: core::to_i64, y: core::to_i64) -> identity {
        native_bool_to_lisp_bool(x < y)
    }
}

define_native_fn! {
    make_gt,
    native_gt(_env, x: core::to_i64, y: core::to_i64) -> identity {
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
    native_first(_env, list: core::to_list) -> identity {
        let first = list.first()
            .ok_or_else(|| error::GenericError::new(
                "cannot do first on empty list".to_string()))?;
        first.clone()
    }
}

define_native_fn! {
    make_rest,
    native_rest(_env, list: core::to_list) -> LispObject::List {
        list.tail()
    }
}

define_native_fn! {
    make_listp,
    native_listp(_env, arg: identity_converter) -> identity {
        let converted = core::to_list(arg);
        native_bool_to_lisp_bool(converted.is_ok())
    }
}

define_native_fn! {
    make_emptyp,
    native_emptyp(_env, arg: core::to_list) -> identity {
        native_bool_to_lisp_bool(arg.is_empty())
    }
}

define_native_fn! {
    make_symbolp,
    native_symbolp(_env, arg: identity_converter) -> identity {
        let converted = core::to_symbol(arg);
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
                        eval::lookup_symbol_macro(&env, s)
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

pub fn prepare_native_stdlib(global_env: &mut core::GlobalEnvFrame) {
    let mut save = |name, maker: fn() -> core::Function| {
        global_env.fn_env.insert(Symbol::new(name), maker());
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
}

#[cfg(test)]
mod tests {
    use crate::error;
    use crate::test_utils::*;

    fn ctx() -> Context {
        Context::new(true, true, false)
    }

    #[test]
    fn test_cons() {
        let ctx = ctx();

        assert!(ctx
            .err_eval("(cons)")
            .downcast::<error::ArityError>()
            .is_ok());
        assert!(ctx
            .err_eval("(cons 1)")
            .downcast::<error::ArityError>()
            .is_ok());

        assert_eq!(ctx.ok_eval("(cons 1 nil)"), read("(1)"));
        assert_eq!(ctx.ok_eval("(cons 1 ())"), read("(1)"));
        assert_eq!(ctx.ok_eval("(cons 1 (quote (2 3)))"), read("(1 2 3)"));
        assert_eq!(
            ctx.ok_eval("(cons (quote (1 2 3)) (quote (2 3)))"),
            read("((1 2 3) 2 3)")
        );
    }

    #[test]
    fn test_first() {
        let ctx = ctx();

        assert!(ctx
            .err_eval("(first)")
            .downcast::<error::ArityError>()
            .is_ok());
        assert!(ctx
            .err_eval("(first nil)")
            .downcast::<error::GenericError>()
            .is_ok());

        assert_eq!(ctx.ok_eval("(first (quote (1)))"), read("1"));
        assert_eq!(ctx.ok_eval("(first (quote (1 2)))"), read("1"));
        assert_eq!(ctx.ok_eval("(first (quote ((1 2 3) 2)))"), read("(1 2 3)"));
    }

    #[test]
    fn test_rest() {
        let ctx = ctx();
        assert!(ctx
            .err_eval("(rest)")
            .downcast::<error::ArityError>()
            .is_ok());

        assert_eq!(ctx.ok_eval("(rest nil)"), read("nil"));
        assert_eq!(ctx.ok_eval("(rest (quote (1)))"), read("nil"));
        assert_eq!(
            ctx.ok_eval("(rest (quote (1 (1 2 3) 4 5)))"),
            read("((1 2 3) 4 5)")
        );
    }

    #[test]
    fn test_equal() {
        let ctx = ctx();
        assert!(ctx
            .err_eval("(equal)")
            .downcast::<error::ArityError>()
            .is_ok());
        assert!(ctx
            .err_eval("(equal 1)")
            .downcast::<error::ArityError>()
            .is_ok());

        assert_eq!(ctx.ok_eval("(equal 1 1)"), read("t"));
        assert_eq!(ctx.ok_eval("(equal 1 2)"), read("nil"));
        assert_eq!(ctx.ok_eval("(equal 1 (quote foo))"), read("nil"));
        assert_eq!(ctx.ok_eval("(equal (quote foo) (quote foo))"), read("t"));
        assert_eq!(
            ctx.ok_eval("(equal (quote (x y (z 1))) (quote (x y (z 1))))"),
            read("t")
        );
        assert_eq!(
            ctx.ok_eval("(equal (quote (x y (z 1))) (quote (x y (z 2))))"),
            read("nil")
        );
    }

    #[test]
    fn test_apply() {
        let ctx = ctx();

        assert!(ctx
            .err_eval("(apply)")
            .downcast::<error::ArityError>()
            .is_ok());
        assert!(ctx
            .err_eval("(set-fn x (lambda ())) (apply (symbol-function x))")
            .downcast::<error::ArityError>()
            .is_ok());
        assert!(ctx
            .err_eval("(set-fn x (lambda ())) (apply (symbol-function x) (quote (1 2)))")
            .downcast::<error::ArityError>()
            .is_ok());

        assert_eq!(
            ctx.ok_eval("(apply (symbol-function +) (quote (1 2)))"),
            read("3")
        );
        assert_eq!(
            ctx.ok_eval("(apply (symbol-function +) (quote (1 2 5)))"),
            read("8")
        );
        assert_eq!(
            ctx.ok_eval("(apply (symbol-function cons) 1 (quote ((2))))"),
            read("(1 2)")
        );
        assert_eq!(
            ctx.ok_eval("(apply (symbol-function apply) (symbol-function +) 1 (quote ((2))))"),
            read("3")
        );
    }

    #[test]
    fn test_add() {
        let ctx = ctx();

        assert_eq!(ctx.ok_eval("(+)"), read("0"));
        assert_eq!(ctx.ok_eval("(+ 1)"), read("1"));
        assert_eq!(ctx.ok_eval("(+ 1 2)"), read("3"));
        assert_eq!(ctx.ok_eval("(+ 1 2 3 4 5)"), read("15"));
    }

    #[test]
    fn test_sub() {
        let ctx = ctx();

        assert!(ctx.err_eval("(-)").downcast::<error::ArityError>().is_ok());

        assert_eq!(ctx.ok_eval("(+ (- 1) 1)"), read("0"));
        assert_eq!(ctx.ok_eval("(+ 1 (- 1 2))"), read("0"));
        assert_eq!(ctx.ok_eval("(+ 13 (- 1 2 3 4 5))"), read("0"));
    }

    #[test]
    fn test_mul() {
        let ctx = ctx();

        assert_eq!(ctx.ok_eval("(*)"), read("1"));
        assert_eq!(ctx.ok_eval("(* 1)"), read("1"));
        assert_eq!(ctx.ok_eval("(* 1 2)"), read("2"));
        assert_eq!(ctx.ok_eval("(* 1 2 3 4 5)"), read("120"));
    }

    #[test]
    fn test_lt() {
        let ctx = ctx();
        assert!(ctx.err_eval("(<)").downcast::<error::ArityError>().is_ok());
        assert!(ctx
            .err_eval("(< 1)")
            .downcast::<error::ArityError>()
            .is_ok());
        assert!(ctx
            .err_eval("(< 1 (quote x))")
            .downcast::<error::CastError>()
            .is_ok());

        assert_eq!(ctx.ok_eval("(< 1 2)"), read("t"));
        assert_eq!(ctx.ok_eval("(< 2 1)"), read("nil"));
    }

    #[test]
    fn test_gt() {
        let ctx = ctx();
        assert!(ctx.err_eval("(>)").downcast::<error::ArityError>().is_ok());
        assert!(ctx
            .err_eval("(> 1)")
            .downcast::<error::ArityError>()
            .is_ok());
        assert!(ctx
            .err_eval("(> 1 (quote x))")
            .downcast::<error::CastError>()
            .is_ok());

        assert_eq!(ctx.ok_eval("(> 1 2)"), read("nil"));
        assert_eq!(ctx.ok_eval("(> 2 1)"), read("t"));
    }

    #[test]
    fn test_listp() {
        let ctx = ctx();
        assert!(ctx
            .err_eval("(listp)")
            .downcast::<error::ArityError>()
            .is_ok());

        assert_eq!(ctx.ok_eval("(listp (quote 1))"), read("nil"));
        assert_eq!(ctx.ok_eval("(listp nil)"), read("t"));
        assert_eq!(ctx.ok_eval("(listp (quote (1 2 3)))"), read("t"));
    }

    #[test]
    fn test_emptyp() {
        let ctx = ctx();
        assert!(ctx
            .err_eval("(emptyp)")
            .downcast::<error::ArityError>()
            .is_ok());

        assert_eq!(ctx.ok_eval("(emptyp nil)"), read("t"));
        assert_eq!(ctx.ok_eval("(emptyp ())"), read("t"));
        assert_eq!(ctx.ok_eval("(emptyp (quote (1 2 3)))"), read("nil"));
    }

    #[test]
    fn test_symbolp() {
        let ctx = ctx();
        assert!(ctx
            .err_eval("(symbolp)")
            .downcast::<error::ArityError>()
            .is_ok());

        assert_eq!(ctx.ok_eval("(symbolp (quote x))"), read("t"));

        assert_eq!(ctx.ok_eval("(symbolp 1)"), read("nil"));

        // TODO: is this ok?
        assert_eq!(ctx.ok_eval("(symbolp nil)"), read("nil"));
        assert_eq!(ctx.ok_eval("(symbolp t)"), read("nil"));
    }

    #[test]
    fn test_macroexpand_1() {
        let ctx = ctx();
        assert!(ctx
            .err_eval("(macroexpand-1)")
            .downcast::<error::ArityError>()
            .is_ok());

        assert_eq!(ctx.ok_eval("(set-macro-fn x (lambda (arg) (if arg (quote x) (quote y)))) (macroexpand-1 (quote (x t)))"), read("x"));
        assert_eq!(ctx.ok_eval("(set-macro-fn x (lambda (arg) (if arg (quote x) (quote y)))) (macroexpand-1 (quote (x nil)))"), read("y"));
        assert_eq!(
            ctx.ok_eval("(macroexpand-1 (quote (cons 1 nil)))"),
            read("(cons 1 nil)")
        );
    }
}
