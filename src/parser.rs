use std::error;
use std::fmt;
use std::result;
use std::str::Chars;

use crate::ast::*;

#[derive(Debug)]
pub struct FilePos {
    linum: u32,
}

impl fmt::Display for FilePos {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        write!(b, "in line {}", self.linum)
    }
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    UnexpectedToken { pos: FilePos, found: String },
}

impl fmt::Display for Error {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::UnexpectedToken { ref pos, ref found } => {
                write!(b, "{}: unexpected token {}", pos, found)
            }
        }
    }
}

impl error::Error for Error {}

pub struct Parser<'a> {
    chars: Chars<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &str) -> Parser {
        Parser {
            chars: source.chars(),
        }
    }

    pub fn parse(&mut self) -> Result<Ast> {
        Ok(Ast {
            program: self.parse_program()?,
        })
    }
}

impl<'a> Parser<'a> {
    fn parse_program(&mut self) -> Result<Program> {
        unimplemented!()
    }

    fn parse_rule(&mut self) -> Result<Rule> {
        unimplemented!()
    }

    fn parse_sentence(&mut self) -> Result<Sentence> {
        unimplemented!()
    }

    fn parse_item(&mut self) -> Result<Item> {
        unimplemented!()
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        unimplemented!()
    }
}
