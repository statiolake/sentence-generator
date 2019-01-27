use rand::seq::SliceRandom;

const MSG: &[&str] = &[
    "落ちついて",
    "よく見ると",
    "Uniformな",
    "Constant",
];

fn main() {
    let mut rng = rand::thread_rng();
    println!("{}", MSG.choose(&mut rng).unwrap());
}
