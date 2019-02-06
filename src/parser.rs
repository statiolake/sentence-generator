use std::borrow::Cow;
use std::collections::HashMap;
use std::error;
use std::fmt;
use std::iter::Peekable;
use std::num::ParseIntError;
use std::result;
use std::vec::IntoIter;

use crate::ast::*;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    UnexpectedEof,
    UnexpectedToken {
        expected: Vec<&'static str>,
        found: String,
    },
    ReDefinitionOfRule {
        name: String,
    },
    ParseIntError(ParseIntError),
}

impl fmt::Display for Error {
    fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::UnexpectedEof => write!(b, "unexpected end of file"),
            Error::UnexpectedToken {
                ref expected,
                ref found,
            } => write!(
                b,
                "unexpected token: expected one of {:?}, found {:?}",
                expected, found
            ),
            Error::ReDefinitionOfRule { ref name } => write!(b, "rule {} re-defined.", name),
            Error::ParseIntError(ref e) => write!(b, "invalid digit: {}", e),
        }
    }
}

impl error::Error for Error {}

impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Error {
        Error::ParseIntError(e)
    }
}

pub struct Parser {
    tokens: Peekable<IntoIter<String>>,
}

impl Parser {
    pub fn new(source: &str) -> Parser {
        Parser {
            tokens: into_token(source).into_iter().peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<Ast> {
        Ok(Ast {
            program: self.parse_program()?,
        })
    }
}

fn into_token(s: &str) -> Vec<String> {
    fn flush(tokens: &mut Vec<String>, token: &mut String) {
        use std::mem::replace;
        if !token.is_empty() {
            tokens.push(replace(token, String::new()));
        }
    }

    let mut in_dbl_quote = false;
    let mut escaped = false;
    let mut next_escaped;
    let mut token = String::new();
    let mut tokens = Vec::new();

    for ch in s.chars() {
        next_escaped = false;
        if in_dbl_quote {
            if escaped {
                token.push(ch)
            } else {
                match ch {
                    '\\' => next_escaped = true,
                    '"' => {
                        in_dbl_quote = false;
                        flush(&mut tokens, &mut token);
                        tokens.push("\"".into());
                    }
                    _ => token.push(ch),
                }
            }
        } else {
            assert!(
                !escaped,
                "internal error: char is escaped outside a string."
            );
            match ch {
                '"' => {
                    in_dbl_quote = true;
                    flush(&mut tokens, &mut token);
                    tokens.push("\"".into());
                }
                ' ' | '\n' | '\r' => flush(&mut tokens, &mut token),
                '{' | '}' | '(' | ')' | '[' | ']' | '?' | '%' | '$' => {
                    flush(&mut tokens, &mut token);
                    tokens.push(ch.to_string());
                }
                _ => token.push(ch),
            }
        }

        escaped = next_escaped;
    }

    flush(&mut tokens, &mut token);

    tokens
}

fn check_unexpected<'a>(
    expected: &[&'static str],
    found: impl Into<Cow<'a, str>>,
) -> Result<String> {
    let found = found.into();
    if expected.iter().any(|&e| e == found) {
        Ok(found.into_owned())
    } else {
        Err(Error::UnexpectedToken {
            expected: expected.into(),
            found: found.into_owned(),
        })
    }
}

impl Parser {
    fn peek_token(&mut self) -> Result<&String> {
        self.tokens.peek().ok_or(Error::UnexpectedEof)
    }

    fn next_token(&mut self) -> Result<String> {
        self.tokens.next().ok_or(Error::UnexpectedEof)
    }

    fn multi_predict(&mut self, expected: &[&'static str]) -> bool {
        self.peek_token()
            .and_then(|found| check_unexpected(expected, found))
            .is_ok()
    }

    fn multi_eat(&mut self, expected: &[&'static str]) -> Result<String> {
        self.next_token()
            .and_then(|found| check_unexpected(expected, found))
    }

    fn predict(&mut self, expected: &'static str) -> bool {
        self.multi_predict(&[expected])
    }

    fn eat(&mut self, expected: &'static str) -> Result<()> {
        self.multi_eat(&[expected]).map(drop)
    }

    fn check_finished(&mut self) -> bool {
        self.peek_token().is_err()
    }

    fn parse_program(&mut self) -> Result<Program> {
        let mut rules = HashMap::new();

        while !self.check_finished() {
            self.eat("rule")?;
            let name = self.next_token()?;
            self.eat("{")?;
            let rule = self.parse_rule()?;
            if rules.insert(name.clone(), rule).is_some() {
                return Err(Error::ReDefinitionOfRule { name });
            }
            self.eat("}")?;
        }

        Ok(Program { rules })
    }

    fn parse_rule(&mut self) -> Result<Rule> {
        let mut sentences = Vec::new();
        while !self.predict("}") {
            sentences.push(self.parse_sentence()?);
        }
        Ok(Rule { sentences })
    }

    fn parse_sentence(&mut self) -> Result<Sentence> {
        let res = match self.multi_eat(&["let", "l", "choice", "c"])?.as_str() {
            "let" | "l" => {
                let ident = self.next_token()?;
                let expr = self.parse_expr()?;
                Ok(Sentence::Let(Let { ident, expr }))
            }
            "choice" | "c" => {
                let weight = self.next_token()?.parse()?;
                self.eat("(")?;
                let mut items = Vec::new();
                while !self.predict(")") {
                    let item = self.parse_item()?;
                    items.push(item);
                }
                self.eat(")")?;
                Ok(Sentence::Choice(Choice { weight, items }))
            }
            _ => unreachable!("multi_eat returned unexpected value"),
        };
        self.eat(";")?;
        res
    }

    fn parse_item(&mut self) -> Result<Item> {
        if self.predict("?") {
            self.eat("?")?;
            Ok(Item {
                prob: self.next_token()?.parse()?,
                expr: {
                    self.eat("%")?;
                    self.parse_expr()?
                },
            })
        } else {
            Ok(Item {
                prob: 100,
                expr: self.parse_expr()?,
            })
        }
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        Ok(match self.multi_eat(&["$", "\"", "["])?.as_str() {
            "$" => Expr::Variable(self.next_token()?),
            "\"" => {
                let expr = Expr::Literal(self.next_token()?);
                self.eat("\"")?;
                expr
            }
            "[" => {
                let expr = Expr::Rule(self.next_token()?);
                self.eat("]")?;
                expr
            }
            _ => unreachable!("internal error: multieat reaturned other string than expectations."),
        })
    }
}
