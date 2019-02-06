// use rand::Rng;

use std::env;
use std::error;
use std::fs::File;
use std::io::prelude::*;

mod ast;
mod parser;

// impl Contents {
//     fn choose_one(&self, rng: &mut impl Rng) -> &Content {
//         let Contents(ref contents) = *self;
//         let sum = contents.iter().fold(0, |x, &Content { prob, .. }| x + prob);
//         let rnd = rng.gen_range(0, sum);

//         let mut ps = 0;
//         for content in contents {
//             ps += content.prob;
//             if rnd < ps {
//                 return content;
//             }
//         }

//         unreachable!("choose_one finished and no content was chosen.");
//     }
// }

// impl Elements {
//     fn generate_sentence(&self, rng: &mut impl Rng) -> String {
//         let mut res = String::new();
//         let Elements(ref contents) = *self;
//         for item in contents {
//             let rnd = rng.gen_range(0, 100);
//             if rnd < item.prob {
//                 let content = item.contents.choose_one(rng);
//                 res += &content.content;
//             }
//         }
//         res
//     }
// }

fn main() {
    if let Err(e) = logic_main() {
        println!("error: {}", e);
    }
}

fn logic_main() -> Result<(), Box<dyn error::Error>> {
    // let mut rng = rand::thread_rng();

    let file_name = env::args()
        .nth(1)
        .unwrap_or_else(|| "index.txt".to_string());

    let mut source = String::new();
    File::open(&file_name)?.read_to_string(&mut source)?;
    let mut parser = parser::Parser::new(&source);

    let ast = dbg!(parser.parse())?;

    Ok(())
}
