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
    ReDefinitionOfVocab {
        name: String,
    },
    ReDefinitionOfVocabForm {
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
            Error::ReDefinitionOfVocab { ref name } => write!(b, "vocab {} re-defined.", name),
            Error::ReDefinitionOfVocabForm { ref name } => {
                write!(b, "vocab form {} re-defined.", name)
            }
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
                match ch {
                    'n' => token.push('\n'),
                    _ => token.push(ch),
                }
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
                '{' | '}' | '[' | ']' | '(' | ')' | '|' | '?' | '%' | '$' => {
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

fn check_unexpected<'a>(expected: &[&'static str], found: &'a str) -> Result<&'a str> {
    if expected.iter().any(|&e| e == found) {
        Ok(found)
    } else {
        Err(Error::UnexpectedToken {
            expected: expected.into(),
            found: found.into(),
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

    fn multi_eat(&mut self, expected: &[&'static str]) -> Result<String> {
        self.next_token()
            .and_then(|found| check_unexpected(expected, found.as_str()).map(|x| x.into()))
    }

    fn multi_predict(&mut self, expected: &[&'static str]) -> Result<&str> {
        self.peek_token()
            .and_then(|found| check_unexpected(expected, found.as_str()))
    }

    fn predict(&mut self, expected: &'static str) -> bool {
        self.multi_predict(&[expected]).is_ok()
    }

    fn may_eat(&mut self, expected: &'static str) -> bool {
        if self.predict(expected) {
            self.eat(expected)
                .expect("internal error: expected token was proved to exist but eat failed.");
            true
        } else {
            false
        }
    }

    fn eat(&mut self, expected: &'static str) -> Result<()> {
        self.multi_eat(&[expected]).map(drop)
    }

    fn check_finished(&mut self) -> bool {
        self.peek_token().is_err()
    }

    fn parse_program(&mut self) -> Result<Program> {
        let mut rules = HashMap::new();
        let mut vocabs = HashMap::new();

        while !self.check_finished() {
            match self.multi_predict(&["rule", "vocab"])? {
                "rule" => {
                    let rule = self.parse_rule()?;
                    let name = rule.name.clone();
                    if rules.insert(rule.name.clone(), rule).is_some() {
                        return Err(Error::ReDefinitionOfRule { name });
                    }
                }
                "vocab" => {
                    let vocab = self.parse_vocab()?;
                    let name = vocab.name.clone();
                    if vocabs.insert(vocab.name.clone(), vocab).is_some() {
                        return Err(Error::ReDefinitionOfVocab { name });
                    }
                }
                _ => unreachable!("internal error: multi_predict returned an unexpected value"),
            }
        }

        Ok(Program { rules, vocabs })
    }

    fn parse_rule(&mut self) -> Result<Rule> {
        self.eat("rule")?;
        let name = self.next_token()?;
        self.eat("{")?;
        let mut sentences = Vec::new();
        while !self.predict("}") {
            sentences.push(self.parse_sentence()?);
        }
        self.eat("}")?;

        Ok(Rule { name, sentences })
    }

    fn parse_vocab(&mut self) -> Result<Vocab> {
        self.eat("vocab")?;
        let name = self.next_token()?;
        let mut index = HashMap::new();
        self.eat("(")?;
        let mut idx = 0;
        while !self.predict(")") {
            let name = self.next_token()?;
            if index.insert(name.clone(), idx).is_some() {
                return Err(Error::ReDefinitionOfVocabForm { name });
            }
            idx += 1;
        }
        self.eat(")")?;
        self.eat("{")?;
        let mut sets = Vec::new();
        while !self.predict("}") {
            let set = self.parse_vocabset(idx)?;
            sets.push(set);
        }
        self.eat("}")?;
        Ok(Vocab { name, index, sets })
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
                let mut items = Vec::new();
                while !self.predict(";") {
                    let item = self.parse_item()?;
                    items.push(item);
                }
                Ok(Sentence::Choice(Choice { weight, items }))
            }
            _ => unreachable!("multi_eat returned unexpected value"),
        };
        self.eat(";")?;
        res
    }

    fn parse_vocabset(&mut self, n: usize) -> Result<VocabSet> {
        self.eat("set")?;
        let weight = self.next_token()?.parse()?;
        let mut forms = Vec::new();
        for _ in 0..n {
            self.eat("\"")?;
            forms.push(self.next_token()?);
            self.eat("\"")?;
        }
        self.eat(";")?;
        Ok(VocabSet { weight, forms })
    }

    fn parse_item(&mut self) -> Result<Item> {
        if self.may_eat("?") {
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
        Ok(match self.multi_eat(&["$", "\"", "[", "("])?.as_str() {
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
            "(" => {
                let expr = Expr::Vocab {
                    name: self.next_token()?,
                    label: self.next_token()?,
                };
                self.eat(")")?;
                expr
            }
            _ => unreachable!("internal error: multieat reaturned other string than expectations."),
        })
    }
}
