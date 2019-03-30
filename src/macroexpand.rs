use crate::cons::List;
use crate::core::Env;
use crate::core::LispObject;
use crate::core::LispObjectResult;
use crate::core::Symbol;
use crate::error;
use crate::eval;
use crate::special;

fn macroexpand_list(env: &Env, list: &List<LispObject>) -> error::GenResult<List<LispObject>> {
    let expanded = list
        .iter()
        .map(|lo| macroexpand_all(env.clone(), lo))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(List::from_rev_iter(expanded))
}

fn macroexpand_into_list(env: &Env, list: &List<LispObject>) -> LispObjectResult {
    Ok(LispObject::List(macroexpand_list(env, list)?))
}

pub fn macroexpand_all(env: Env, form: &LispObject) -> LispObjectResult {
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
                    special::parse_quote(&list.tail())?;
                    Ok(LispObject::List(list.clone()))
                }
                LispObject::Symbol(s) if *s == Symbol::new("lambda") => {
                    let lambda_forms = &list.tail();
                    special::parse_lambda(&lambda_forms)?;

                    let expanded_body = macroexpand_list(&env, &lambda_forms.tail())?;
                    let lambda_form = expanded_body
                        .cons_rc(lambda_forms.first_rc().unwrap().clone())
                        .cons_rc(list.first_rc().unwrap().clone());
                    Ok(LispObject::List(lambda_form))
                }
                LispObject::Symbol(s) if *s == Symbol::new("let") => {
                    let let_forms = list.tail();
                    let special::ParsedLet { bindings, body } = special::parse_let(&let_forms)?;
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
                LispObject::Symbol(s) => match eval::lookup_symbol_macro(&env, s) {
                    Some(ref f) => {
                        let expanded =
                            eval::call_function_object(env.clone(), f, list.tail(), false)?;
                        macroexpand_all(env, &expanded)
                    }
                    None => macroexpand_into_list(&env, list),
                },
                _ => macroexpand_into_list(&env, list),
            }
        }
    }
}
