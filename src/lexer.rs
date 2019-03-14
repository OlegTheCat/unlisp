use std::io;
use std::io::Read;

use pushback_reader::PushbackReader;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    LeftPar,
    RightPar,
    IntegerLiteral(i64),
    StringLiteral(String),
    Symbol(String),
    Unexpected,
}

pub struct Lexer<'a, T: Read + 'a> {
    pbr: PushbackReader<'a, T>,
}

fn valid_symbol_char(c: char) -> bool {
    c.is_alphanumeric() || c == '&' || c == '*' || c == '-'
}

impl<'a, T: Read> Lexer<'a, T> {
    pub fn create(r: &'a mut T) -> Lexer<'a, T> {
        Lexer {
            pbr: PushbackReader::create(r),
        }
    }

    fn next_char(&mut self) -> io::Result<Option<char>> {
        let mut one_byte: [u8; 1] = [0];
        match self.pbr.read_exact(&mut one_byte) {
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => Ok(None),
            Err(e) => Err(e),
            Ok(_) => Ok(Some(one_byte[0] as char)),
        }
    }

    fn unread_char(&mut self, c: char) {
        self.pbr.unread_byte(c as u8);
    }

    fn read_string_literal(&mut self) -> io::Result<String> {
        let mut buf = Vec::new();
        loop {
            let c = self.next_char()?;
            if c.is_none() {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "eof while reading string literal",
                ));
            }

            // TODO: handle escaping
            let c = c.unwrap();
            if c == '"' {
                break;
            }
            buf.push(c);
        }

        Ok(buf.into_iter().collect())
    }

    fn read_integer_literal(&mut self) -> io::Result<i64> {
        let mut buf = Vec::new();
        loop {
            let c = self.next_char()?;
            if c.is_none() {
                break;
            }

            let c = c.unwrap();
            if c.is_numeric() {
                buf.push(c);
            } else {
                self.unread_char(c);
                break;
            }
        }

        let s: String = buf.into_iter().collect();
        Ok(s.parse::<i64>().unwrap())
    }

    fn read_symbol(&mut self) -> io::Result<String> {
        let mut buf = Vec::new();
        loop {
            let c = self.next_char()?;
            if c.is_none() {
                break;
            }

            let c = c.unwrap();
            if valid_symbol_char(c) {
                buf.push(c);
            } else {
                self.unread_char(c);
                break;
            }
        }

        Ok(buf.into_iter().collect())
    }

    fn skip_line(&mut self) -> io::Result<()> {
        let mut next_char = self.next_char()?;
        while next_char.is_some() && next_char.unwrap() != '\n' {
            next_char = self.next_char()?;
        }

        Ok(())
    }

    pub fn next_token(&mut self) -> io::Result<Option<Token>> {
        let c = self.next_char()?;
        if c.is_none() {
            return Ok(None);
        }

        let c = c.unwrap();

        if c.is_whitespace() {
            return self.next_token();
        }

        let tok = match c {
            ';' => {
                self.skip_line()?;
                return self.next_token();
            }
            '(' => Token::LeftPar,
            ')' => Token::RightPar,

            c if c.is_numeric() => {
                self.unread_char(c);
                Token::IntegerLiteral(self.read_integer_literal()?)
            }

            c if valid_symbol_char(c) => {
                self.unread_char(c);
                Token::Symbol(self.read_symbol()?)
            }

            '"' => Token::StringLiteral(self.read_string_literal()?),
            _ => Token::Unexpected,
        };

        Ok(Some(tok))
    }
}
