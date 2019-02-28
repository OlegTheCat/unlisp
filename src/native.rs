use core;
use core::Env;
use core::LispObject;
use core::Symbol;
use error;
use im::Vector;
use std::io::Write;
use eval;

macro_rules! define_native_fn {
    ($id:ident ($env:ident, $( $arg:ident : $converter:path ),*) -> $result_wrap:path $body:block) => {
        #[allow(unused_mut)]
        fn $id( $env: &mut core::Env, lo: LispObject ) -> error::GenResult<LispObject> {
            let mut form = core::to_vector(lo)?;
            let mut args = form.slice(1..);

            let mut parameters_count = 0;
            $( stringify!($arg); parameters_count += 1; )*

                if parameters_count != args.len() {
                    return Err(Box::new(
                        error::ArityError::new(parameters_count,
                                               args.len(),
                                               stringify!($id).to_string())));
                }

            let mut iter = args.into_iter();
            $( let mut $arg = $converter(iter.next().unwrap())?; )*

            let res = $result_wrap($body);
            Ok(res)
        }
    };

    ($id:ident ($env:ident, $( $arg:ident : $converter:path, )* ... $vararg:ident : $vconverter:path ) -> $result_wrap:path $body:block) => {
        #[allow(unused_mut)]
        fn $id( $env: &mut core::Env, lo: LispObject ) -> error::GenResult<LispObject> {
            let mut form = core::to_vector(lo)?;
            let mut args = form.slice(1..);

            #[allow(unused_mut)]
            let mut non_vararg_parameters_count = 0;
            $( stringify!($arg); non_vararg_parameters_count += 1; )*

                if non_vararg_parameters_count > args.len() {
                    return Err(Box::new(
                        error::ArityError::new(non_vararg_parameters_count,
                                               args.len(),
                                               stringify!($id).to_string())));
                }

            #[allow(unused_mut)]
            let mut iter = args.into_iter();

            $( #[allow(unused_mut)] let mut $arg = $converter(iter.next().unwrap())?; )*

            let mut $vararg: Vector<_> = iter
                .map(|lo| $vconverter(lo))
                .collect::<Result<Vector<_>, _>>()?;

            let res = $result_wrap($body);
            Ok(res)
        }
    }
}

fn native_bool_to_lisp_bool(b: bool) -> LispObject {
    if b {
        LispObject::T
    } else {
        LispObject::Nil
    }
}

define_native_fn! {
    native_stdout_write(_env, s: core::to_string) -> core::identity {
        write!(std::io::stdout(), "{}", s)?;
        LispObject::Nil
    }
}

define_native_fn! {
    native_print(_env, x: core::identity_converter) -> core::identity {
        print!("{}", &x);
        x
    }
}

define_native_fn! {
    native_println(_env, x: core::identity_converter) -> core::identity {
        println!("{}", &x);
        x
    }
}

define_native_fn! {
    native_apply(env, f: core::to_function, ... args: core::identity_converter) -> core::identity {
        let last_arg =
            core::to_vector(args.pop_back()
                            .ok_or(error::ArityError::new(
                                2,
                                1,
                                "apply".to_string()
        ))?)?;
        args.append(last_arg);
        eval::call_function_object(env, f, args, false)?
    }
}

define_native_fn! {
    native_add(_env, ... args: core::to_i64) -> LispObject::Integer {
        let mut res = 0;
        for arg in args {
            res += arg
        }
        res
    }
}

define_native_fn! {
    native_sub(_env, from: core::to_i64, ... args: core::to_i64) -> LispObject::Integer {
        let mut res = from;
        for arg in args {
            res -= arg
        }
        res
    }
}

define_native_fn! {
    native_mul(_env, from: core::to_i64, ... args: core::to_i64) -> LispObject::Integer {
        let mut res = from;
        for arg in args {
            res *= arg
        }
        res
    }
}

define_native_fn! {
    native_equal(_env, x: core::identity_converter, y: core::identity_converter) -> core::identity {
        native_bool_to_lisp_bool(x == y)
    }
}

define_native_fn! {
    native_cons(_env, item: core::identity_converter, list: core::to_vector) -> LispObject::Vector {
        list.push_front(item);
        list
    }
}

define_native_fn! {
    native_first(_env, list: core::to_vector) -> core::identity {
        let first = list.into_iter().next()
            .ok_or(
                Box::new(
                    error::GenericError::new(
                        "cannot do first on empty list".to_string())))?;
        first
    }
}

define_native_fn! {
    native_rest(_env, list: core::to_vector) -> LispObject::Vector {
        list.slice(1..)
    }
}

define_native_fn! {
    native_listp(_env, arg: core::identity_converter) -> core::identity {
        let converted = core::to_vector(arg);
        native_bool_to_lisp_bool(converted.is_ok())
    }
}

define_native_fn! {
    native_emptyp(_env, arg: core::to_vector) -> core::identity {
        native_bool_to_lisp_bool(arg.is_empty())
    }
}

define_native_fn! {
    native_symbolp(_env, arg: core::identity_converter) -> core::identity {
        let converted = core::to_symbol(arg);
        native_bool_to_lisp_bool(converted.is_ok())
    }
}

pub fn prepare_native_stdlib(env: &mut Env) {
    let mut set = |name: &str, f| {
        env.global_env.fn_env.insert(
            Symbol(name.to_string()),
            core::Function::NativeFunction(core::NativeFnWrapper(f)),
        );
    };

    set("apply", native_apply);
    set("add", native_add);
    set("sub", native_sub);
    set("mul", native_mul);
    set("equal", native_equal);
    set("cons", native_cons);
    set("first", native_first);
    set("rest", native_rest);
    set("listp", native_listp);
    set("emptyp", native_emptyp);
    set("symbolp", native_symbolp);
    set("print", native_print);
    set("println", native_println);
    set("stdout-write", native_stdout_write);
}
