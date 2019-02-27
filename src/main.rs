use std::io;
// use std::io::BufRead;
use std::fs;
use std::io::Write;
use std::thread;

extern crate im;
extern crate scopeguard;

mod native_fn_helpers;
mod core;
mod error;
mod lexer;
mod pushback_reader;
mod reader;
mod special;
mod eval;
mod print;


fn eval_stdlib(env: &mut core::Env) {
    let mut file = fs::File::open("src/stdlib.unl").expect("stdlib file not found");

    let mut reader = reader::Reader::create(&mut file);
    loop {
        match reader.read_form() {
            Ok(form) => {
                eval::eval(env, form).expect("error during stdlib eval");
            }
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => break,

            Err(ref e) => panic!("Unexpected error during stdlib eval: {}", e),
        }
    }
}

fn repl() {
    let mut stdin = io::stdin();

    print!(">>> ");
    io::stdout().flush().unwrap();

    let mut env = core::Env::new();
    special::prepare_specials(&mut env);
    eval::prepare_native_stdlib(&mut env);
    eval_stdlib(&mut env);

    let mut reader = reader::Reader::create(&mut stdin);

    loop {
        match reader.read_form() {
            Ok(form) => match eval::eval(&mut env, form) {
                Ok(lo) => {
                    println!("{}", lo);
                }
                Err(e) => println!("error: {}", e),
            },
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => println!("EOF error"),
            Err(ref e) => println!("Unexpected error: {}", e),
        }

        print!(">>> ");
        io::stdout().flush().unwrap();
    }
}

fn main() {
    let child = thread::Builder::new()
        .stack_size(1024 * 1024 * 1024)
        .spawn(repl)
        .unwrap();

    child.join().unwrap();
}
