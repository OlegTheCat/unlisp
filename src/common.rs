use crate::env;
use crate::eval;
use crate::macroexpand;
use crate::native;
use crate::object;
use crate::print;
use crate::reader;
use crate::special;

use std::fs;
use std::io;
use std::error::Error;

pub fn macroexpand_and_eval(env: env::Env, form: &object::LispObject) -> eval::EvalResult {
    let expanded = macroexpand::macroexpand_all(env.clone(), form)?;
    eval::eval(env, &expanded)
}

pub fn eval_stdlib(env: &env::Env) {
    let mut file = fs::File::open("src/stdlib.unl").expect("stdlib file not found");

    let mut reader = reader::Reader::create(&mut file);
    loop {
        match reader.read_form() {
            Ok(form) => {
                let res = macroexpand_and_eval(env.clone(), &form);
                res.map_err(|e| {
                    println!("error during stdlib eval: {}", e.err);
                    print::print_stack_trace(&e.stack_trace);
                })
                .unwrap();
            }
            ref err @ Err(_) if is_gen_eof(err) => break,

            Err(ref e) => panic!("Reader error during stdlib eval: {}", e),
        }
    }
}

pub fn init_env(env: &mut env::Env) {
    special::prepare_specials(env);
    native::prepare_natives(env);
    eval_stdlib(env);
}

pub fn is_gen_eof<T>(result: &Result<T, Box<Error>>) -> bool {
    match result {
        Err(e) => match e.downcast_ref::<io::Error>() {
            Some(io_err) => io_err.kind() == io::ErrorKind::UnexpectedEof,
            None => false,
        },
        _ => false,
    }
}
