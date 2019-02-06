use std::collections::HashMap;

pub struct Ast {
    pub program: Program,
}

pub struct Program {
    pub rules: HashMap<String, Rule>,
}

pub struct Rule {
    pub sentences: Vec<Sentence>,
}

pub enum Sentence {
    Let { ident: String, expr: Expr },
    Choice { weight: u32, items: Vec<Item> },
}

pub struct Item {
    pub prob: u32,
    pub expr: Expr,
}

pub enum Expr {
    Literal(String),
    Rule(String),
    Ident(String),
}
