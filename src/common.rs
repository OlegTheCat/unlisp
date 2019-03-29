use crate::core;
use crate::error;
use crate::eval;
use crate::macroexpand;
use crate::native;
use crate::reader;
use crate::special;

use std::fs;
use std::io;

pub fn macroexpand_and_eval(
    env: core::Env,
    form: &core::LispObject,
) -> error::GenResult<core::LispObject> {
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
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => break,

            Err(ref e) => panic!("Unexpected error during stdlib eval: {}", e),
        }
    }
}

pub fn init_env(env: &mut core::Env) {
    special::prepare_specials(&mut env.global_env_mut());
    native::prepare_native_stdlib(&mut env.global_env_mut());
    eval_stdlib(env);
}
