use im::Vector;
use im::HashMap;
use object;
use object::LispObject;
use object::Symbol;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EnvFrame {
    sym_env: HashMap<Symbol, LispObject>,
    fn_env: HashMap<Symbol, object::Function>
}

impl EnvFrame {
    pub fn new() -> EnvFrame {
        EnvFrame {
            sym_env: HashMap::new(),
            fn_env: HashMap::new()
        }
    }

}

#[derive(Clone)]
pub struct Env {
    envs: Vector<EnvFrame>
}

impl Env {
    pub fn new() -> Env {
        let frame = EnvFrame::new();
        let mut envs = Vector::new();
        envs.push_back(frame);
        Env{
            envs: envs
        }
    }

    fn push_frame(&self, frame: EnvFrame) -> Env {
        let mut new_env = self.clone();
        new_env.envs.push_front(frame);
        new_env
    }
}

fn nth(vec: Vector<LispObject>, i: usize) -> LispObject {
    vec.into_iter().nth(i).unwrap()
}

fn lookup_symbol_value(env: &mut Env, s: &Symbol) -> Option<LispObject> {
    for frame in &env.envs {
        let val = frame.sym_env.get(s);
        if val.is_some() {
            return Some(val.unwrap().clone());
        }
    }

    None
}

fn lookup_symbol_fn(env: &mut Env, s: &Symbol) -> Option<object::Function> {
    for frame in &env.envs {
        let val = frame.fn_env.get(s);
        if val.is_some() {
            return Some(val.unwrap().clone());
        }
    }

    None
}

fn let_form(env: &mut Env, form: LispObject) -> LispObject {
    let form = object::to_vector(form);
    let bindings = object::to_vector(nth(form.clone(), 1));
    let mut new_env = env.clone();

    for binding in bindings {
        let binding = object::to_vector(binding);
        let sym = object::to_symbol(nth(binding.clone(), 0));
        let val = eval(env, nth(binding.clone(), 1));
        let mut env_frame = EnvFrame::new();
        env_frame.sym_env.insert(sym, val);
        new_env = new_env.push_frame(env_frame);
    }

    let body = form.clone().slice(2..);
    let mut res = LispObject::Nil;
    for form in body {
        res = eval(&mut new_env, form);
    }

    res
}

fn add_form(env: &mut Env, form: LispObject) -> LispObject {
    let args = object::to_vector(form).slice(1..);
    let mut res = 0;
    for arg in args {
        res += object::to_i64(eval(env, arg));
    }
    LispObject::Integer(res)
}

fn quote_form(_env: &mut Env, form: LispObject) -> LispObject {
    nth(object::to_vector(form), 1)
}

fn lambda_form(_env: &mut Env, form: LispObject) -> LispObject {
    let form = object::to_vector(form);
    let arglist = object::to_vector(nth(form.clone(), 1))
        .into_iter()
        .map(|lo| object::to_symbol(lo))
        .collect();
    let body = form.clone().slice(2..);

    LispObject::Fn(object::Function {
        arglist: arglist,
        body: body
    })
}

fn call_fn(env: &mut Env, form: LispObject) -> LispObject {
    let form = object::to_vector(form);
    let func = object::to_function(nth(form.clone(), 0));
    let arg_vals: Vector<LispObject> = form.clone().slice(1..)
        .into_iter()
        .map(|lo| eval(env, lo))
        .collect();

    if func.arglist.len() != arg_vals.len() {
        panic!("Wrong number of arguments passed to a function")
    }

    let mut frame = EnvFrame::new();
    for (sym, val) in func.arglist.clone().into_iter().zip(arg_vals.into_iter()) {
        frame.sym_env.insert(sym, val);
    }

    let mut new_env = env.push_frame(frame);

    let mut result = LispObject::Nil;
    for form in func.body {
        result = eval(&mut new_env, form);
    }

    result
}

fn funcall(env: &mut Env, form: LispObject) -> LispObject {
    let form = object::to_vector(form);
    let f = object::to_function(nth(form.clone(), 1));
    let mut args = form.clone().slice(2..);
    args.push_front(LispObject::Fn(f));
    eval(env, LispObject::Vector(args))
}

fn set_fn(env: &mut Env, form: LispObject) -> LispObject {
    let form = object::to_vector(form);
    let sym = object::to_symbol(nth(form.clone(), 1));
    let f = object::to_function(eval(env, nth(form.clone(), 2)));
    let envs = &mut env.envs;
    envs.iter_mut().last().unwrap().fn_env.insert(sym, f);
    LispObject::Nil
}

pub fn eval(env: &mut Env, form: LispObject) -> LispObject {
    match form {
        LispObject::Nil => LispObject::Nil,
        LispObject::T => LispObject::T,
        LispObject::Integer(i) => LispObject::Integer(i),
        LispObject::String(s) => LispObject::String(s),
        LispObject::Symbol(s) => lookup_symbol_value(env, &s).unwrap(),
        LispObject::Fn(f) => LispObject::Fn(f),
        LispObject::Vector(ref vec) => {
            match nth(vec.clone(), 0) {
                LispObject::Fn(_) => call_fn(env, form.clone()),
                LispObject::Symbol(Symbol(ref s)) if s == "let" => let_form(env, form.clone()),
                LispObject::Symbol(Symbol(ref s)) if s == "add" => add_form(env, form.clone()),
                LispObject::Symbol(Symbol(ref s)) if s == "quote" => quote_form(env, form.clone()),
                LispObject::Symbol(Symbol(ref s)) if s == "lambda" => lambda_form(env, form.clone()),
                LispObject::Symbol(Symbol(ref s)) if s == "funcall" => funcall(env, form.clone()),
                LispObject::Symbol(Symbol(ref s)) if s == "setfn" => set_fn(env, form.clone()),
                LispObject::Symbol(s) => {
                    let f = lookup_symbol_fn(env, &s).unwrap();
                    let mut new_form = vec.clone().slice(1..);
                    new_form.push_front(LispObject::Fn(f));
                    eval(env, LispObject::Vector(new_form))
                },
                head => {
                    let head_val = eval(env, head);
                    let mut new_form = vec.clone().slice(1..);
                    new_form.push_front(head_val);
                    eval(env, LispObject::Vector(new_form))
                }
            }
        }
    }
}
