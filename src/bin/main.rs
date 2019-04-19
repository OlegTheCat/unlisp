extern crate im;
extern crate unlisp;

use std::io;
use std::io::Write;
use std::thread;

use unlisp::common::*;
use unlisp::env;
use unlisp::print::print_stack_trace;
use unlisp::reader;

fn repl() {
    let mut stdin = io::stdin();

    let prompt = || {
        print!(">>> ");
        io::stdout().flush().unwrap();
    };

    let mut env = env::Env::new();
    init_env(&mut env);

    let mut reader = reader::Reader::create(&mut stdin);

    prompt();
    loop {
        match reader.read_form() {
            Ok(Some(form)) => match macroexpand_and_eval(env.clone(), &form) {
                Ok(lo) => {
                    println!("{}", lo);
                }
                Err(e) => {
                    println!("error: {}", e.err);
                    print_stack_trace(&e.stack_trace);
                }
            },
            Ok(None) => break,
            Err(ref e) => println!("reader error: {}", e),
        }
        prompt();
    }
}

fn main() {
    let child = thread::Builder::new()
        .stack_size(10 * 1024 * 1024 * 1024)
        .spawn(repl)
        .unwrap();

    child.join().unwrap();
}
