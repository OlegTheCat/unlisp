use crate::common::is_gen_eof;
use crate::cons::List;
use crate::error::SyntaxError;
use crate::lexer::Lexer;
use crate::lexer::Token;
use crate::object::LispObject;
use crate::object::Symbol;
use std::error::Error;
use std::io::Read;

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

    fn read_list_form(&mut self) -> Result<LispObject, Box<Error>> {
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
                    tok => panic!("unexpected token {:?}", tok),
                }
            }

            vec.push(form);
            tok = self.lexer.next_token()?;
        }

        Ok(LispObject::List(List::from_rev_iter(vec)))
    }

    pub fn read_form(&mut self) -> Result<Option<LispObject>, Box<Error>> {
        let tok = self.lexer.next_token();

        if is_gen_eof(&tok) {
            return Ok(None);
        }

        let tok = tok?;

        let trivial_form = self.tok_to_trivial_form(&tok);
        let form = match trivial_form {
            Some(form) => form,
            None => match tok {
                Token::LeftPar => self.read_list_form()?,
                Token::RightPar => Err(SyntaxError::new("unbalanced parens"))?,
                tok => panic!("unexpected token {:?}", tok),
            },
        };

        Ok(Some(form))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::is_gen_eof;
    use crate::cons::List;

    #[test]
    fn test_empty() {
        let mut input = "".as_bytes();
        let mut reader = Reader::create(&mut input);

        assert_eq!(reader.read_form().unwrap(), None);

        let mut input = "foo".as_bytes();
        let mut reader = Reader::create(&mut input);

        let _ = reader.read_form().unwrap().unwrap();
        assert_eq!(reader.read_form().unwrap(), None);
    }

    #[test]
    fn test_integer_literal() {
        let mut input = "1 12 1000 2019".as_bytes();
        let mut reader = Reader::create(&mut input);

        assert_eq!(reader.read_form().unwrap().unwrap(), LispObject::Integer(1));
        assert_eq!(
            reader.read_form().unwrap().unwrap(),
            LispObject::Integer(12)
        );
        assert_eq!(
            reader.read_form().unwrap().unwrap(),
            LispObject::Integer(1000)
        );
        assert_eq!(
            reader.read_form().unwrap().unwrap(),
            LispObject::Integer(2019)
        );
    }

    #[test]
    fn test_string_literal() {
        let mut input = "\"\" \"foo\" \"bar\"".as_bytes();
        let mut reader = Reader::create(&mut input);

        assert_eq!(
            reader.read_form().unwrap().unwrap(),
            LispObject::String("".to_string())
        );
        assert_eq!(
            reader.read_form().unwrap().unwrap(),
            LispObject::String("foo".to_string())
        );
        assert_eq!(
            reader.read_form().unwrap().unwrap(),
            LispObject::String("bar".to_string())
        );
    }

    #[test]
    fn test_symbol() {
        let mut input = "x foo bar*".as_bytes();
        let mut reader = Reader::create(&mut input);

        assert_eq!(
            reader.read_form().unwrap().unwrap(),
            LispObject::Symbol(Symbol::new("x"))
        );
        assert_eq!(
            reader.read_form().unwrap().unwrap(),
            LispObject::Symbol(Symbol::new("foo"))
        );
        assert_eq!(
            reader.read_form().unwrap().unwrap(),
            LispObject::Symbol(Symbol::new("bar*"))
        );
    }

    #[test]
    fn test_list() {
        let mut input = "() (foo bar) (foo (bar baz) quux)".as_bytes();
        let mut reader = Reader::create(&mut input);

        let sym = |x| LispObject::Symbol(Symbol::new(x));

        assert_eq!(reader.read_form().unwrap().unwrap(), LispObject::nil());
        assert_eq!(
            reader.read_form().unwrap().unwrap(),
            LispObject::List(List::from_rev_iter(vec![sym("foo"), sym("bar")]))
        );

        assert_eq!(
            reader.read_form().unwrap().unwrap(),
            LispObject::List(List::from_rev_iter(vec![
                sym("foo"),
                LispObject::List(List::from_rev_iter(vec![sym("bar"), sym("baz")])),
                sym("quux")
            ]))
        );
    }

    #[test]
    fn test_nil_t() {
        let mut input = "nil t".as_bytes();
        let mut reader = Reader::create(&mut input);

        assert_eq!(reader.read_form().unwrap().unwrap(), LispObject::nil());
        assert_eq!(reader.read_form().unwrap().unwrap(), LispObject::T);
    }

    #[test]
    fn test_incomplete_list() {
        let mut input = "(foo".as_bytes();
        let mut reader = Reader::create(&mut input);
        assert!(is_gen_eof(&reader.read_form()));
    }

    //TODO: tests on unbalanced pars
}
