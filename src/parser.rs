use std::collections::HashMap;
use std::error;
use std::fmt;
use std::num::ParseIntError;
use std::result;
use std::vec::IntoIter;

use crate::ast::*;

// #[derive(Debug)]
// pub struct FilePos {
//     linum: u32,
// }

// impl fmt::Display for FilePos {
//     fn fmt(&self, b: &mut fmt::Formatter) -> fmt::Result {
//         write!(b, "in line {}", self.linum)
//     }
// }

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
    tokens: IntoIter<String>,
}

impl Parser {
    pub fn new(source: &str) -> Parser {
        Parser {
            tokens: into_token(source).into_iter(),
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
                '{' | '}' | '(' | ')' | '[' | ']' | '?' | '%' | '+' | ',' | '$' => {
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

impl Parser {
    fn next_token(&mut self) -> Result<String> {
        self.tokens.next().ok_or(Error::UnexpectedEof)
    }

    fn eat(&mut self, expect: &'static str) -> Result<()> {
        self.multieat(&[expect]).map(drop)
    }

    fn multieat(&mut self, expect: &[&'static str]) -> Result<String> {
        self.next_token().and_then(|token| {
            if expect.iter().any(|&e| token == e) {
                Ok(token)
            } else {
                Err(Error::UnexpectedToken {
                    expected: expect.to_owned(),
                    found: token,
                })
            }
        })
    }

    fn parse_program(&mut self) -> Result<Program> {
        let mut rules = HashMap::new();
        loop {
            if let Err(e) = self.eat("rule") {
                return if let Error::UnexpectedEof = e {
                    Ok(Program { rules })
                } else {
                    Err(e)
                };
            }
            let name = self.next_token()?;
            self.eat("{")?;
            let rule = self.parse_rule()?;
            if rules.insert(name.clone(), rule).is_some() {
                return Err(Error::ReDefinitionOfRule { name });
            }
        }
    }

    fn parse_rule(&mut self) -> Result<Rule> {
        let mut sentences = Vec::new();
        loop {
            let sentence = self.parse_sentence()?;
            match sentence {
                Some(sentence) => {
                    self.eat(";")?;
                    sentences.push(sentence)
                }
                None => {
                    return Ok(Rule { sentences });
                }
            }
        }
    }

    fn parse_sentence(&mut self) -> Result<Option<Sentence>> {
        match self.multieat(&["let", "choice", "}"])?.as_str() {
            "let" => {
                let ident = self.next_token()?;
                let expr = self.parse_expr()?;
                Ok(Some(Sentence::Let(Let { ident, expr })))
            }
            "choice" => {
                let weight = self.next_token()?.parse()?;
                self.eat("(")?;
                let mut items = Vec::new();
                loop {
                    let item = self.parse_item()?;
                    items.push(item);
                    if self.multieat(&[",", ")"])? == ")" {
                        return Ok(Some(Sentence::Choice(Choice { weight, items })));
                    }
                }
            }
            "}" => Ok(None),
            _ => unreachable!("internal error: multieat reaturned other string than expectations."),
        }
    }

    fn parse_item(&mut self) -> Result<Item> {
        match self.multieat(&["+", "?"])?.as_str() {
            "+" => Ok(Item {
                prob: 100,
                expr: self.parse_expr()?,
            }),
            "?" => Ok(Item {
                prob: self.next_token()?.parse()?,
                expr: {
                    self.eat("%")?;
                    self.parse_expr()?
                },
            }),
            _ => unreachable!("internal error: multieat reaturned other string than expectations."),
        }
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        Ok(match self.multieat(&["$", "\"", "["])?.as_str() {
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
