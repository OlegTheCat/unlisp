use cons::List;
use core;
use core::LispObject;
use core::Symbol;
use error;
use eval;
use std::io::Write;

fn identity_converter(v: &LispObject) -> error::GenResult<&LispObject> {
    Ok(&v)
}

fn identity(v: LispObject) -> LispObject {
    v
}

macro_rules! define_native_fn {
    ($id:ident ($env:ident, $( $arg:ident : $converter:path ),*) -> $result_wrap:path $body:block) => {
        #[allow(unused_mut)]
        fn $id( $env: core::Env, args: List<LispObject> ) -> error::GenResult<LispObject> {
            let args_count = args.len();
            let mut args = args.iter();
            let mut parameters_count = 0;
            $( stringify!($arg); parameters_count += 1; )*

                if parameters_count != args_count {
                    return Err(Box::new(
                        error::ArityError::new(parameters_count,
                                               args_count,
                                               stringify!($id).to_string())));
                }

            $( let mut $arg = $converter(args.next().unwrap())?; )*

            let res = $result_wrap($body);
            Ok(res)
        }
    };

    ($id:ident ($env:ident, $( $arg:ident : $converter:path, )* ... $vararg:ident : $vconverter:path ) -> $result_wrap:path $body:block) => {
        #[allow(unused_mut)]
        fn $id( $env: core::Env, args: List<LispObject> ) -> error::GenResult<LispObject> {

            let args_count = args.len();
            let mut args = args.iter();

            #[allow(unused_mut)]
            let mut non_vararg_parameters_count = 0;
            $( stringify!($arg); non_vararg_parameters_count += 1; )*

                if non_vararg_parameters_count > args_count {
                    return Err(Box::new(
                        error::ArityError::new(non_vararg_parameters_count,
                                               args_count,
                                               stringify!($id).to_string())));
                }

            #[allow(unused_mut)]

            $( #[allow(unused_mut)] let mut $arg = $converter(args.next().unwrap())?; )*

            let mut $vararg: List<_> = args
                .map(|lo| $vconverter(lo))
                .collect::<Result<List<_>, _>>()?;

            let res = $result_wrap($body);
            Ok(res)
        }
    }
}

define_native_fn! {
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
    native_stdout_write(_env, s: core::to_string) -> identity {
        write!(std::io::stdout(), "{}", s)?;
        LispObject::nil()
    }
}

define_native_fn! {
    native_print(_env, x: identity_converter) -> identity {
        print!("{}", &x);
        x.clone()
    }
}

define_native_fn! {
    native_println(_env, x: identity_converter) -> identity {
        println!("{}", &x);
        x.clone()
    }
}

fn native_apply(env: core::Env, args: List<LispObject>) -> error::GenResult<LispObject> {
    if args.len() <= 1 {
        return Err(Box::new(error::ArityError::new(2, 1, "apply".to_string())));
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

    eval::call_function_object(env, f, args, false)
}

define_native_fn! {
    native_add(_env, ... args: core::to_i64) -> LispObject::Integer {
        let mut res = 0;
        for arg in args.iter() {
            res += *arg
        }
        res
    }
}

define_native_fn! {
    native_sub(_env, from: core::to_i64, ... args: core::to_i64) -> LispObject::Integer {
        let mut res = *from;
        for arg in args.iter() {
            res -= *arg
        }
        res
    }
}

define_native_fn! {
    native_mul(_env, from: core::to_i64, ... args: core::to_i64) -> LispObject::Integer {
        let mut res = *from;
        for arg in args.iter() {
            res *= *arg
        }
        res
    }
}

define_native_fn! {
    native_equal(_env, x: identity_converter, y: identity_converter) -> identity {
        native_bool_to_lisp_bool(*x == *y)
    }
}

define_native_fn! {
    native_first(_env, list: core::to_list) -> identity {
        let first = list.first()
            .ok_or(
                Box::new(
                    error::GenericError::new(
                        "cannot do first on empty list".to_string())))?;
        first.clone()
    }
}

define_native_fn! {
    native_rest(_env, list: core::to_list) -> LispObject::List {
        list.tail()
    }
}

define_native_fn! {
    native_listp(_env, arg: identity_converter) -> identity {
        let converted = core::to_list(arg);
        native_bool_to_lisp_bool(converted.is_ok())
    }
}

define_native_fn! {
    native_emptyp(_env, arg: core::to_list) -> identity {
        native_bool_to_lisp_bool(arg.is_empty())
    }
}

define_native_fn! {
    native_symbolp(_env, arg: identity_converter) -> identity {
        let converted = core::to_symbol(arg);
        native_bool_to_lisp_bool(converted.is_ok())
    }
}

define_native_fn! {
    native_macroexpand(env, arg: identity_converter) -> identity {
        match arg {
            LispObject::List(ref list) if !list.is_empty() => {
                match list.ufirst() {
                    LispObject::Symbol(s) => {
                        eval::lookup_symbol_macro(&env, s)
                            .map_or_else(|| Ok(arg.clone()),
                                         |macro_fn| eval::call_function_object(env, &macro_fn, list.tail(), false))?
                    }
                    _ => arg.clone()
                }
            }
            _ => arg.clone()
        }
    }
}

pub fn prepare_native_stdlib(global_env: &mut core::GlobalEnvFrame) {
    let mut set = |name: &str, f| {
        global_env.fn_env.insert(
            Symbol::new(name),
            core::Function::NativeFunction(core::NativeFnWrapper(f)),
        );
    };

    set("cons", native_cons);
    set("apply", native_apply);
    set("add", native_add);
    set("sub", native_sub);
    set("mul", native_mul);
    set("equal", native_equal);
    set("first", native_first);
    set("rest", native_rest);
    set("listp", native_listp);
    set("emptyp", native_emptyp);
    set("symbolp", native_symbolp);
    set("print", native_print);
    set("println", native_println);
    set("stdout-write", native_stdout_write);
    set("macroexpand-1", native_macroexpand);
}
