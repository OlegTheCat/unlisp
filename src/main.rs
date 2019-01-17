use std::io;
use std::io::BufRead;
use std::io::Write;

extern crate im;

mod pushback_reader;
mod lexer;
mod core;
mod native_fn_helpers;
mod reader;
mod error;
mod eval;
mod print;


fn main() {
    let stdin = io::stdin();

    print!(">>> ");
    io::stdout().flush().unwrap();

    let mut env = core::Env::new();
    eval::prepare_stdlib(&mut env);

    for line in stdin.lock().lines() {

        let line = line.unwrap();
        let mut bytes = line.as_bytes();
        let mut reader = reader::Reader::create(&mut bytes);

        match reader.read_form() {
            Ok(form) => {
                match eval::eval(&mut env, form) {
                    Ok(lo) => {
                        println!("{}", lo);
                    },
                    Err(e) => println!("error: {}", e)
                }
            },
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof =>
                println!("EOF error"),
            Err(_) => println!("Unexpected error.")
        }

        print!(">>> ");
        io::stdout().flush().unwrap();
    }
}
