use rand::Rng;

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

mod ast;
mod parser;

struct Content {
    content: String,
    prob: u32,
}

struct Contents(Vec<Content>);

struct Element {
    contents: Contents,
    prob: u32,
}

struct Elements(Vec<Element>);

impl Contents {
    fn choose_one(&self, rng: &mut impl Rng) -> &Content {
        let Contents(ref contents) = *self;
        let sum = contents.iter().fold(0, |x, &Content { prob, .. }| x + prob);
        let rnd = rng.gen_range(0, sum);

        let mut ps = 0;
        for content in contents {
            ps += content.prob;
            if rnd < ps {
                return content;
            }
        }

        unreachable!("choose_one finished and no content was chosen.");
    }
}

impl Elements {
    fn generate_sentence(&self, rng: &mut impl Rng) -> String {
        let mut res = String::new();
        let Elements(ref contents) = *self;
        for item in contents {
            let rnd = rng.gen_range(0, 100);
            if rnd < item.prob {
                let content = item.contents.choose_one(rng);
                res += &content.content;
            }
        }
        res
    }
}

fn main() -> io::Result<()> {
    let mut rng = rand::thread_rng();

    let file_name = env::args()
        .nth(1)
        .unwrap_or_else(|| "index.txt".to_string());

    let list = read_index(Path::new(&file_name))?;

    println!("{}", list.generate_sentence(&mut rng));

    Ok(())
}

fn read_index(file_name: &Path) -> io::Result<Elements> {
    let f = File::open(file_name)?;
    let br = BufReader::new(f);
    let mut list = Vec::new();
    for l in br.lines() {
        let l = l?;
        let mut l = l.split(',');
        let (prob, file_name) = (
            l.next().unwrap().parse().unwrap(),
            Path::new(l.next().unwrap()),
        );
        let contents = read_contents(&file_name)?;
        list.push(Element { contents, prob });
    }

    Ok(Elements(list))
}

fn read_contents(file_name: &Path) -> io::Result<Contents> {
    let f = File::open(file_name)?;
    let br = BufReader::new(f);

    let mut contents = Vec::new();
    for l in br.lines() {
        let l = l?;
        let mut l = l.split(',');
        let (prob, content) = (l.next().unwrap().parse().unwrap(), l.next().unwrap().into());
        contents.push(Content { content, prob });
    }

    Ok(Contents(contents))
}
