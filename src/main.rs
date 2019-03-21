use std::io;
use std::io::Write;
use std::ops::DerefMut;
use std::thread;

extern crate im;

mod common;
mod cons;
mod core;
mod error;
mod eval;
mod lexer;
mod macroexpand;
mod native;
mod print;
mod pushback_reader;
mod reader;
mod special;
#[cfg(test)]
mod test_utils;

use crate::common::*;

fn repl() {
    let mut stdin = io::stdin();

    print!(">>> ");
    io::stdout().flush().unwrap();

    let env = core::Env::new();
    special::prepare_specials(env.global_env.as_ref().borrow_mut().deref_mut());
    native::prepare_native_stdlib(env.global_env.as_ref().borrow_mut().deref_mut());
    eval_stdlib(&env);

    // let mut s = "(mapcar (symf add) (range 1000) (range 1000))".as_bytes();
    // let mut reader = reader::Reader::create(&mut s);

    let mut reader = reader::Reader::create(&mut stdin);

    loop {
        match reader.read_form() {
            Ok(form) => match macroexpand_and_eval(env.clone(), &form) {
                Ok(lo) => {
                    println!("{}", lo);
                }
                Err(e) => println!("error: {}", e),
            },
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(ref e) => println!("Unexpected error: {}", e),
        }

        print!(">>> ");
        io::stdout().flush().unwrap();
    }
}

fn main() {
    let child = thread::Builder::new()
        .stack_size(10 * 1024 * 1024 * 1024)
        .spawn(repl)
        .unwrap();

    child.join().unwrap();
}
