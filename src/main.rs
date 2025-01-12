use std::usize;

use clap::Parser;
use wordle::{
    algorithms::{allocs::Allocs, Naive},
    Guesser,
};

const GAMES: &str = include_str!("../data/answers.txt");

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    implementation: Implementation,

    #[clap(short, long)]
    max: Option<usize>,
}

#[derive(clap::ValueEnum, Debug, Clone)]
enum Implementation {
    Naive,
    Allocs,
}

fn main() {
    let args = Args::parse();

    match args.implementation {
        Implementation::Naive => play(Naive::new, args.max),
        Implementation::Allocs => play(Allocs::new, args.max),
    }
}

fn play<G>(mut mk: impl FnMut() -> G, max: Option<usize>)
where
    G: Guesser,
{
    let w = wordle::Wordle::new();
    for answer in GAMES.split_whitespace().take(max.unwrap_or(usize::MAX)) {
        let guesser = (mk)();
        if let Some(score) = w.play(answer, guesser) {
            println!("guessed {} in {} rounds", answer, score);
        } else {
            eprintln!("failed to guess")
        };
    }
}
