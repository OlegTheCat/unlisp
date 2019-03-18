use core::LispObject;
use core::Symbol;
use std::io;
use std::io::Read;

use super::lexer::Lexer;
use super::lexer::Token;
use cons::List;

pub struct Reader<'a, T: Read + 'a> {
    lexer: Lexer<'a, T>,
}

impl<'a, T: Read + 'a> Reader<'a, T> {
    pub fn create(r: &'a mut T) -> Reader<'a, T> {
        Reader {
            lexer: Lexer::create(r),
        }
    }

    fn tok_to_trivial_form(&self, tok: &Token) -> Option<LispObject> {
        match tok {
            Token::Symbol(s) if s == "nil" => Some(LispObject::List(List::empty())),
            Token::Symbol(s) if s == "t" => Some(LispObject::T),
            Token::Symbol(s) => Some(LispObject::Symbol(Symbol::new(s.clone()))),
            Token::IntegerLiteral(i) => Some(LispObject::Integer(*i)),
            Token::StringLiteral(s) => Some(LispObject::String(s.to_string())),
            _ => None,
        }
    }

    fn read_list_form(&mut self) -> io::Result<LispObject> {
        let mut vec = Vec::new();

        let mut tok = self.lexer.next_token()?;

        while tok != Token::RightPar {
            let form;

            if let Some(t_form) = self.tok_to_trivial_form(&tok) {
                form = t_form;
            } else {
                form = match tok {
                    Token::LeftPar => self.read_list_form()?,
                    Token::RightPar => break,
                    _ => panic!("Unexpected token"),
                }
            }

            vec.push(form);
            tok = self.lexer.next_token()?;
        }

        Ok(LispObject::List(List::from_rev_iter(vec.into_iter())))
    }

    pub fn read_form(&mut self) -> io::Result<LispObject> {
        let tok = self.lexer.next_token()?;

        let trivial_form = self.tok_to_trivial_form(&tok);
        let form = match trivial_form {
            Some(form) => form,
            None => match tok {
                Token::LeftPar => self.read_list_form()?,
                Token::RightPar => panic!("unbalanced pars"),
                _ => panic!("Unexpected token"),
            },
        };

        Ok(form)
    }
}
