use std::io;
// use std::io::BufRead;
use std::io::Write;

extern crate im;
extern crate scopeguard;

mod pushback_reader;
mod lexer;
mod core;
mod native_fn_helpers;
mod reader;
mod error;
mod eval;
mod print;
mod special;

fn main() {

    let mut stdin = io::stdin();

    print!(">>> ");
    io::stdout().flush().unwrap();

    let mut env = core::Env::new();
    special::prepare_specials(&mut env);
    eval::prepare_stdlib(&mut env);

    let mut reader = reader::Reader::create(&mut stdin);

    loop {
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
            Err(ref e) => println!("Unexpected error: {}", e)
        }

        print!(">>> ");
        io::stdout().flush().unwrap();
    }
}
