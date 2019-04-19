#![feature(test)]

extern crate test;
extern crate unlisp;
use test::Bencher;

use unlisp::common::*;
use unlisp::env::Env;
use unlisp::object::LispObject;
use unlisp::reader::Reader;

fn read(s: impl Into<String>) -> LispObject {
    let s = s.into();
    let mut bytes = s.as_bytes();
    let mut reader = Reader::create(&mut bytes);
    reader.read_form().unwrap().unwrap()
}

fn env() -> Env {
    let mut env = Env::new();
    init_env(&mut env);

    env
}

#[bench]
fn bench_mapcar(b: &mut Bencher) {
    let env = env();
    let form = read("(mapcar (symf +) (range 100) (range 100))");

    b.iter(|| macroexpand_and_eval(env.clone(), &form).unwrap());
}

#[bench]
fn bench_tested_mapcars(b: &mut Bencher) {
    let env = env();
    let form = read("(reduce (symf +) 0 (mapcar-single (symf +) (mapcar-single (symf +) (mapcar-single (symf +) (range 200)))))");

    b.iter(|| macroexpand_and_eval(env.clone(), &form).unwrap());
}
