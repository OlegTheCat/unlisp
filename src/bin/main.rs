extern crate im;
extern crate unlisp;

use std::io;
use std::io::Write;
use std::thread;

use unlisp::common::*;
use unlisp::env;
use unlisp::reader;

fn repl() {
    let mut stdin = io::stdin();

    print!(">>> ");
    io::stdout().flush().unwrap();

    let mut env = env::Env::new();
    init_env(&mut env);

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
            ref err @ Err(_) if is_gen_eof(err) => break,
            Err(ref e) => println!("reader error: {}", e),
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
