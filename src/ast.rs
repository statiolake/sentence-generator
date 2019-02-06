use std::collections::HashMap;

#[derive(Debug)]
pub struct Ast {
    pub program: Program,
}

#[derive(Debug)]
pub struct Program {
    pub rules: HashMap<String, Rule>,
}

#[derive(Debug)]
pub struct Rule {
    pub sentences: Vec<Sentence>,
}

#[derive(Debug)]
pub enum Sentence {
    Let { ident: String, expr: Expr },
    Choice { weight: u32, items: Vec<Item> },
}

#[derive(Debug)]
pub struct Item {
    pub prob: u32,
    pub expr: Expr,
}

#[derive(Debug)]
pub enum Expr {
    Variable(String),
    Literal(String),
    Rule(String),
}
