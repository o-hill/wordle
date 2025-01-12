use std::{borrow::Cow, ops::Neg};

use crate::{Correctness, Guess, Guesser, DICTIONARY};

#[derive(Default)]
pub struct VecRem {
    remaining: Vec<(&'static str, usize)>,
}

impl VecRem {
    pub fn new() -> Self {
        Self {
            remaining: Vec::from_iter(DICTIONARY.lines().map(|line| {
                let (word, count) = line
                    .split_once(' ')
                    .expect("every line is `word frequency`");
                let count: usize = count
                    .parse()
                    .unwrap_or_else(|_| panic!("counts should be numbers {}", count));
                (word, count)
            })),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Candidate {
    word: &'static str,
    goodness: f64,
}

impl Guesser for VecRem {
    fn guess(&mut self, history: &[crate::Guess]) -> String {
        let mut best: Option<Candidate> = None;

        if let Some(last) = history.last() {
            self.remaining.retain(|(word, _count)| last.matches(word));
        }

        if history.is_empty() {
            return "tares".to_string();
        }

        let remaining_count: usize = self.remaining.iter().map(|&(_, count)| count).sum();

        for &(word, _) in &self.remaining {
            let mut sum = 0.0;
            for pattern in Correctness::patterns() {
                // Considers guessing this word and computes how much
                // information (aka how large is the remaining search space,
                // including the likelihood of the constituents)
                // we will have gained from the guess.
                let mut in_pattern_total = 0;
                for (candidate, count) in &self.remaining {
                    let g = Guess {
                        word: Cow::Borrowed(word),
                        mask: pattern,
                    };

                    if g.matches(candidate) {
                        in_pattern_total += count;
                    }
                }

                if in_pattern_total == 0 {
                    continue;
                }

                // TODO: Apply sigmoid.
                let p_pattern = in_pattern_total as f64 / remaining_count as f64;
                sum += p_pattern * p_pattern.log2();
            }

            let goodness = sum.neg();
            if let Some(c) = best {
                if goodness > c.goodness {
                    best = Some(Candidate { word, goodness })
                }
            } else {
                best = Some(Candidate { word, goodness })
            }
        }
        best.unwrap().word.to_string()
    }
}
