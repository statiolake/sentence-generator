use std::str::Chars;

use crate::ast::*;

pub struct Parser<'a> {
    chars: Chars<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &str) -> Parser {
        Parser {
            chars: source.chars(),
        }
    }

    pub fn parse(&mut self) -> Ast {
        Ast {
            program: self.parse_program(),
        }
    }
}

impl<'a> Parser<'a> {
    fn parse_program(&mut self) -> Program {
        unimplemented!()
    }

    fn parse_rule(&mut self) -> Rule {
        unimplemented!()
    }

    fn parse_sentence(&mut self) -> Sentence {
        unimplemented!()
    }

    fn parse_item(&mut self) -> Item {
        unimplemented!()
    }

    fn parse_expr(&mut self) -> Expr {
        unimplemented!()
    }
}
