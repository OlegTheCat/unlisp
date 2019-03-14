use std::io;
// use std::io::BufRead;
use std::fs;
use std::io::Write;
use std::thread;
use std::ops::DerefMut;

extern crate im;
extern crate scopeguard;

mod core;
mod error;
mod lexer;
mod pushback_reader;
mod reader;
mod special;
mod native;
mod eval;
mod print;


fn eval_stdlib(env: &core::Env) {
    let mut file = fs::File::open("src/stdlib.unl").expect("stdlib file not found");

    let mut reader = reader::Reader::create(&mut file);
    loop {
        match reader.read_form() {
            Ok(form) => {
                eval::eval(env.clone(), form).expect("error during stdlib eval");
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

    let env = core::Env::new();
    special::prepare_specials(env.global_env.as_ref().borrow_mut().deref_mut());
    native::prepare_native_stdlib(env.global_env.as_ref().borrow_mut().deref_mut());
    eval_stdlib(&env);

    let mut reader = reader::Reader::create(&mut stdin);

    loop {
        match reader.read_form() {
            Ok(form) => match eval::eval(env.clone(), form) {
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
