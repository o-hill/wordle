const GAMES: &str = include_str!("../data/answers.txt");

fn main() {
    let w = wordle::Wordle::new();
    for answer in GAMES.split_whitespace() {
        let guesser = wordle::algorithms::Naive::new();
        if let Some(score) = w.play(answer, guesser) {
            println!("guessed {} in {} rounds", answer, score);
        } else {
            eprintln!("failed to guess")
        };
    }
}
