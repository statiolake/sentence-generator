use std::collections::HashMap;
use std::result;

use rand::Rng;

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
    Let(Let),
    Choice(Choice),
}

#[derive(Debug)]
pub struct Let {
    pub ident: String,
    pub expr: Expr,
}

#[derive(Debug)]
pub struct Choice {
    pub weight: u32,
    pub items: Vec<Item>,
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

pub type Result = result::Result<String, String>;

impl Ast {
    pub fn generate(&self, rng: &mut impl Rng) -> Result {
        self.program.generate(rng)
    }
}

impl Program {
    pub fn generate(&self, rng: &mut impl Rng) -> Result {
        let main = self
            .rules
            .get("main")
            .ok_or_else(|| "`main` rule is not defined.".to_string())?;

        main.generate(&self.rules, rng)
    }
}

impl Rule {
    fn generate(&self, rules: &HashMap<String, Rule>, rng: &mut impl Rng) -> Result {
        let lets = self.sentences.iter().filter_map(|x| x.get_let_ref());
        let mut variables = HashMap::new();
        for l in lets {
            if variables
                .insert(&l.ident, l.expr.generate(rules, &variables, rng)?)
                .is_some()
            {
                return Err(format!("re-definition of variable {}", l.ident));
            }
        }

        let choices = self.sentences.iter().filter_map(|x| x.get_choice_ref());
        Self::choose_one(choices.collect(), rng).generate(rules, &variables, rng)
    }

    fn choose_one<'a>(choices: Vec<&'a Choice>, rng: &mut impl Rng) -> &'a Choice {
        let sum = choices
            .iter()
            .fold(0, |x, &&Choice { weight, .. }| x + weight);
        let rnd = rng.gen_range(0, sum);

        let mut ps = 0;
        for choice in choices {
            ps += choice.weight;
            if rnd < ps {
                return choice;
            }
        }

        unreachable!("choose_one finished and no choice was chosen.");
    }
}

impl Sentence {
    fn get_let_ref(&self) -> Option<&Let> {
        match *self {
            Sentence::Let(ref l) => Some(l),
            _ => None,
        }
    }
    fn get_choice_ref(&self) -> Option<&Choice> {
        match *self {
            Sentence::Choice(ref c) => Some(c),
            _ => None,
        }
    }
}

impl Choice {
    fn generate(
        &self,
        rules: &HashMap<String, Rule>,
        variables: &HashMap<&String, String>,
        rng: &mut impl Rng,
    ) -> Result {
        let mut s = String::new();
        for item in &self.items {
            if let Some(x) = item.generate(rules, variables, rng) {
                s += &x?;
            }
        }
        Ok(s)
    }
}

impl Item {
    fn generate(
        &self,
        rules: &HashMap<String, Rule>,
        variables: &HashMap<&String, String>,
        rng: &mut impl Rng,
    ) -> Option<Result> {
        let rnd = rng.gen_range(0, 100);
        if rnd < self.prob {
            Some(self.expr.generate(rules, variables, rng))
        } else {
            None
        }
    }
}

impl Expr {
    fn generate(
        &self,
        rules: &HashMap<String, Rule>,
        variables: &HashMap<&String, String>,
        rng: &mut impl Rng,
    ) -> Result {
        match *self {
            Expr::Variable(ref name) => Ok(variables
                .get(name)
                .ok_or_else(|| format!("undeclared variable: {}", name))?
                .clone()),
            Expr::Literal(ref lit) => Ok(lit.clone()),
            Expr::Rule(ref rule) => rules
                .get(rule.as_str())
                .ok_or_else(|| format!("undeclared rule: {}", rule))?
                .generate(rules, rng),
        }
    }
}
