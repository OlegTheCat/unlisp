use crate::core;
use crate::error;
use crate::eval;
use crate::macroexpand;
use crate::native;
use crate::reader;
use crate::special;

use std::fs;
use std::io;

pub fn macroexpand_and_eval(env: core::Env, form: &core::LispObject) -> core::LispObjectResult {
    let expanded = macroexpand::macroexpand_all(env.clone(), form)?;
    eval::eval(env, &expanded)
}

pub fn eval_stdlib(env: &core::Env) {
    let mut file = fs::File::open("src/stdlib.unl").expect("stdlib file not found");

    let mut reader = reader::Reader::create(&mut file);
    loop {
        match reader.read_form() {
            Ok(form) => {
                macroexpand_and_eval(env.clone(), &form).expect("error during stdlib eval");
            }
            ref err @ Err(_) if is_gen_eof(err) => break,

            Err(ref e) => panic!("Unexpected error during stdlib eval: {}", e),
        }
    }
}

pub fn init_env(env: &mut core::Env) {
    special::prepare_specials(env);
    native::prepare_natives(env);
    eval_stdlib(env);
}

pub fn is_gen_eof<T>(result: &error::GenResult<T>) -> bool {
    match result {
        Err(e) => match e.downcast_ref::<io::Error>() {
            Some(io_err) => io_err.kind() == io::ErrorKind::UnexpectedEof,
            None => false,
        },
        _ => false,
    }
}
