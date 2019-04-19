use crate::cons::List;
use crate::env::Env;
use crate::error;
use crate::eval;
use crate::eval::EvalResult;
use crate::object::LispObject;
use crate::object::Symbol;
use crate::special;

fn macroexpand_list(
    env: &Env,
    list: &List<LispObject>,
) -> Result<List<LispObject>, error::ErrorWithStackTrace> {
    let expanded = list
        .iter()
        .map(|lo| macroexpand_all(env.clone(), lo))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(List::from_rev_iter(expanded))
}

fn macroexpand_into_list(env: &Env, list: &List<LispObject>) -> EvalResult {
    Ok(LispObject::List(macroexpand_list(env, list)?))
}

pub fn macroexpand_all(env: Env, form: &LispObject) -> EvalResult {
    match form {
        self_expand @ LispObject::T
        | self_expand @ LispObject::Integer(_)
        | self_expand @ LispObject::String(_)
        | self_expand @ LispObject::Fn(_)
        | self_expand @ LispObject::Symbol(_) => Ok(self_expand.clone()),

        LispObject::List(list) if list.is_empty() => Ok(LispObject::nil()),

        LispObject::List(list) => {
            match list.ufirst() {
                LispObject::Symbol(s) if *s == Symbol::new("quote") => {
                    env.attach_st_box(special::parse_quote(&list.tail()))?;
                    Ok(LispObject::List(list.clone()))
                }
                LispObject::Symbol(s) if *s == Symbol::new("lambda") => {
                    let special::ParsedLambda { name, body, .. } =
                        env.attach_st_box(special::parse_lambda(&list.tail()))?;

                    let expanded_body = macroexpand_list(&env, &body)?;

                    let mut to_recons = vec![];
                    let lambda_form_iter = list.rc_iter();

                    if name.is_some() {
                        // reconsing lambda symbol, arglist and name
                        to_recons.extend(lambda_form_iter.take(3));
                    } else {
                        // reconsing lambda symbol and name
                        to_recons.extend(lambda_form_iter.take(2));
                    }

                    let mut reconsed_lambda = expanded_body;

                    for el in to_recons.into_iter().rev() {
                        reconsed_lambda = reconsed_lambda.cons_rc(el);
                    }

                    Ok(LispObject::List(reconsed_lambda))
                }
                LispObject::Symbol(s) if *s == Symbol::new("let") => {
                    let let_forms = list.tail();
                    let special::ParsedLet { bindings, body } =
                        env.attach_st_box(special::parse_let(&let_forms))?;
                    let expanded_body = macroexpand_list(&env, &body)?;

                    let mut expanded_bindings = List::empty();

                    // implementation with try_fold cannot be compiled for some reason
                    for (sym, val_form) in bindings.into_iter().rev() {
                        let expanded_val = macroexpand_all(env.clone(), val_form)?;
                        let reconstructed_binding = List::empty()
                            .cons(expanded_val)
                            .cons(LispObject::Symbol(sym));
                        expanded_bindings =
                            expanded_bindings.cons(LispObject::List(reconstructed_binding));
                    }

                    Ok(LispObject::List(
                        expanded_body
                            .cons(LispObject::List(expanded_bindings))
                            .cons_rc(list.first_rc().unwrap().clone()),
                    ))
                }
                LispObject::Symbol(s) => match env.lookup_symbol_macro(s) {
                    Some(ref f) => {
                        let expanded = eval::call_function_object(
                            env.clone(),
                            f,
                            list.tail(),
                            false,
                            Some(s),
                        )?;
                        macroexpand_all(env, &expanded)
                    }
                    None => macroexpand_into_list(&env, list),
                },
                _ => macroexpand_into_list(&env, list),
            }
        }
    }
}
