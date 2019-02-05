use rand::Rng;

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

struct Entry {
    msg: String,
    prob: u32,
}

struct Entries(Vec<Entry>);

struct ListItem {
    entries: Entries,
    prob: u32,
}

struct List(Vec<ListItem>);

impl Entries {
    fn choose_one(&self, rng: &mut impl Rng) -> &Entry {
        let Entries(ref entries) = *self;
        let sum = entries.iter().fold(0, |x, &Entry { prob, .. }| x + prob);
        let rnd = rng.gen_range(0, sum);

        let mut ps = 0;
        for entry in entries {
            ps += entry.prob;
            if rnd < ps {
                return entry;
            }
        }

        unreachable!("choose_one finished and no entry was chosen.");
    }
}

impl List {
    fn generate_sentence(&self, rng: &mut impl Rng) -> String {
        let mut res = String::new();
        let List(ref entries) = *self;
        for item in entries {
            let rnd = rng.gen_range(0, 100);
            if rnd < item.prob {
                let entry = item.entries.choose_one(rng);
                res += &entry.msg;
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

fn read_index(file_name: &Path) -> io::Result<List> {
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
        let entries = read_entries(&file_name)?;
        list.push(ListItem {
            entries: Entries(entries),
            prob,
        });
    }

    Ok(List(list))
}

fn read_entries(file_name: &Path) -> io::Result<Vec<Entry>> {
    let f = File::open(file_name)?;
    let br = BufReader::new(f);

    let mut entries = Vec::new();
    for l in br.lines() {
        let l = l?;
        let mut l = l.split(",");
        let (prob, msg) = (l.next().unwrap().parse().unwrap(), l.next().unwrap().into());
        entries.push(Entry { msg, prob });
    }

    Ok(entries)
}
