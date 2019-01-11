use std::io;
use std::io::Read;
use core::LispObject;
use core::Symbol;

use super::lexer::Token;
use super::lexer::Lexer;
use im::Vector;

pub struct Reader<'a, T: Read + 'a> {
    lexer: Lexer<'a, T>
}

impl <'a, T: Read + 'a> Reader<'a, T> {
    pub fn create(r: &'a mut T) -> Reader<'a, T> {
        Reader {
            lexer: Lexer::create(r)
        }
    }

    fn eof(&self) -> io::Result<Token> {
        Err(io::Error::new(io::ErrorKind::UnexpectedEof,
                           "eof while retrieving token"))
    }

    fn read_tok_or_eof(&mut self) -> io::Result<Token> {
        let tok = self.lexer.next_token()?;

        if tok.is_none() {
            return self.eof();
        }

        Ok(tok.unwrap())
    }

    fn tok_to_trivial_form(&self, tok: &Token) -> Option<LispObject> {
        match tok {
            Token::Symbol(s) if s == "nil" => Some(LispObject::Nil),
            Token::Symbol(s) if s == "t" => Some(LispObject::T),
            Token::Symbol(s) => Some(LispObject::Symbol(Symbol(s.to_string()))),
            Token::IntegerLiteral(i) => Some(LispObject::Integer(*i)),
            Token::StringLiteral(s) => Some(LispObject::String(s.to_string())),
            _ => None
        }
    }

    fn read_list_form(&mut self) -> io::Result<LispObject> {
        let mut vec = Vector::new();

        let mut tok = self.read_tok_or_eof()?;

        while tok != Token::RightPar {

            let form;

            if let Some(t_form) = self.tok_to_trivial_form(&tok) {
                form = t_form;
            } else {
                form = match tok {
                    Token::LeftPar => self.read_list_form()?,
                    Token::RightPar => break,
                    _ => panic!("Unexpected token")
                }
            }

            vec.push_back(form);
            tok = self.read_tok_or_eof()?;
        }

        Ok(LispObject::Vector(vec))
    }

    pub fn read_form(&mut self) -> io::Result<LispObject> {
        let tok = self.read_tok_or_eof()?;

        let trivial_form = self.tok_to_trivial_form(&tok);
        let form = match trivial_form {
            Some(form) => form,
            None => match tok {
                Token::LeftPar => self.read_list_form()?,
                Token::RightPar => panic!("unbalanced pars"),
                _ => panic!("Unexpected token")
            }
        };

        Ok(form)
    }
}
