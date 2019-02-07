use std::env;
use std::error;
use std::fs::File;
use std::io::prelude::*;

mod ast;
mod parser;

fn main() {
    if let Err(e) = logic_main() {
        println!("error: {}", e);
    }
}

fn logic_main() -> Result<(), Box<dyn error::Error>> {
    let mut rng = rand::thread_rng();

    let mut args = env::args().skip(1);

    let file_name = args.next().unwrap_or_else(|| "index.txt".to_string());
    let count: usize = args.next().unwrap_or_else(|| "1".to_string()).parse()?;

    let mut source = String::new();
    File::open(&file_name)?.read_to_string(&mut source)?;
    let mut parser = parser::Parser::new(&source);
    let ast = parser.parse()?;

    for _ in 0..count {
        println!("{}", ast.generate(&mut rng)?);
    }

    Ok(())
}
