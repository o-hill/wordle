use std::{borrow::Cow, collections::HashSet};

pub mod algorithms;

#[derive(Default, Debug)]
pub struct Wordle {
    dictionary: HashSet<&'static str>,
}

pub const DICTIONARY: &str = include_str!("../data/library-counts.txt");

impl Wordle {
    pub fn new() -> Self {
        Self {
            dictionary: HashSet::from_iter(DICTIONARY.lines().map(|line| {
                line.split_once(' ')
                    .expect("every line is `word frequency`")
                    .0
            })),
        }
    }

    pub fn play<G: Guesser>(&self, answer: &'static str, mut guesser: G) -> Option<usize> {
        let mut history = Vec::new();

        for round in 1..=32 {
            let guess = guesser.guess(&history[..]);
            if guess == answer {
                return Some(round);
            }

            assert!(self.dictionary.contains(guess.as_str()), "guess {}", guess);

            let correctness = Correctness::compute(answer, guess.as_str());
            history.push(Guess {
                word: Cow::Owned(guess),
                mask: correctness,
            });
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Correctness {
    /// Green
    Correct,
    /// Yellow
    Misplaced,
    /// Gray
    Wrong,
}

impl Correctness {
    fn compute(answer: &str, guess: &str) -> [Self; 5] {
        assert_eq!(answer.len(), 5);
        assert_eq!(guess.len(), 5);
        let mut c = [Correctness::Wrong; 5];
        for (i, (a, g)) in answer.chars().zip(guess.chars()).enumerate() {
            if a == g {
                c[i] = Correctness::Correct;
            }
        }

        let mut used = [false; 5];
        for (i, c) in c.iter().enumerate() {
            if *c == Correctness::Correct {
                used[i] = true;
            }
        }

        for (i, g) in guess.chars().enumerate() {
            if c[i] == Correctness::Correct {
                continue;
            }
            if answer.chars().enumerate().any(|(j, a)| {
                if a == g && !used[j] {
                    used[j] = true;
                    return true;
                }
                false
            }) {
                c[i] = Correctness::Misplaced;
            }
        }

        c
    }

    pub fn patterns() -> impl Iterator<Item = [Self; 5]> {
        itertools::iproduct!(
            [Self::Correct, Self::Misplaced, Self::Wrong],
            [Self::Correct, Self::Misplaced, Self::Wrong],
            [Self::Correct, Self::Misplaced, Self::Wrong],
            [Self::Correct, Self::Misplaced, Self::Wrong],
            [Self::Correct, Self::Misplaced, Self::Wrong],
        )
        .map(|(a, b, c, d, e)| [a, b, c, d, e])
    }
}

pub struct Guess<'a> {
    word: Cow<'a, str>,
    mask: [Correctness; 5],
}

impl Guess<'_> {
    fn matches(&self, word: &str) -> bool {
        assert_eq!(self.word.len(), 5);
        assert_eq!(word.len(), 5);

        let gchars = self.word.as_bytes();
        let wchars = word.as_bytes();

        let mut used = [false; 5];
        for (i, ((g, &m), w)) in self
            .word
            .bytes()
            .zip(self.mask.iter())
            .zip(word.bytes())
            .enumerate()
        {
            if m == Correctness::Correct {
                if g != w {
                    return false;
                } else {
                    used[i] = true;
                    continue;
                }
            }
        }
        for (i, (w, &m)) in word.bytes().zip(&self.mask).enumerate() {
            if m == Correctness::Correct {
                continue;
            }

            let mut plausible = true;
            if self
                .word
                .bytes()
                .zip(&self.mask)
                .enumerate()
                .any(|(j, (g, m))| {
                    if g != w {
                        return false;
                    }
                    // This position has already been used to support a charcter.
                    if used[j] {
                        return false;
                    }
                    // We're looking at an `w` in `word` and have found an `w` in the previous guess.
                    // The color of that previous `w` will tell us if this `w` _might_ be okay.
                    match m {
                        Correctness::Correct => unreachable!(
                            "all correct guesses should have resulted in return or be used"
                        ),
                        Correctness::Misplaced if j == i => {
                            // `w` was yellow in this same position last time around, which means that
                            // `word` cannot be the answer.
                            plausible = false;
                            false
                        }
                        Correctness::Misplaced => {
                            // `w` was yellow in this same position last time around, which means that
                            // `word` cannot be the answer.
                            used[j] = true;
                            true
                        }
                        Correctness::Wrong => {
                            plausible = false;
                            false
                        }
                    }
                })
                && plausible
            {
                // The character `w` was yellow in the previous guess.
                assert!(plausible);
            } else if !plausible {
                return false;
            } else {
                // We have no information about character `w` so `word` might still match.
            }
        }

        true
    }
}

pub trait Guesser {
    fn guess(&mut self, history: &[Guess]) -> String;
}

impl Guesser for fn(history: &[Guess]) -> String {
    fn guess(&mut self, history: &[Guess]) -> String {
        (*self)(history)
    }
}

#[cfg(test)]
macro_rules! guesser {
    (|$history:ident| $impl:block) => {{
        struct G;
        impl $crate::Guesser for G {
            fn guess(&mut self, $history: &[Guess]) -> String {
                $impl
            }
        }
        G
    }};
}

#[cfg(test)]
mod tests {
    macro_rules! mask {
        (C) => {$crate::Correctness::Correct};
        (M) => {$crate::Correctness::Misplaced};
        (W) => {$crate::Correctness::Wrong};
        ($($c:tt)+) => {[
            $(mask!($c)),+
        ]}
    }

    mod guess_matcher {
        use crate::Guess;
        use std::borrow::Cow;

        macro_rules! check {
            ($prev:literal + [$($mask:tt)+] allows $next:literal) => {
                assert!(Guess {
                    word: Cow::Owned($prev.to_string()),
                    mask: mask![$($mask )+]
                }
                .matches($next));
            };

            ($prev:literal + [$($mask:tt)+] disallows $next:literal) => {
                assert!(!Guess {
                    word: Cow::Owned($prev.to_string()),
                    mask: mask![$($mask )+]
                }
                .matches($next))
            }
        }

        #[test]
        fn matches() {
            check!("abcde" + [C C C C C] allows "abcde");
            check!("abcdf" + [C C C C C] disallows "abcde");
            check!("abcde" + [W W W W W] allows "fghij");
            check!("abcde" + [M M M M M] allows "eabcd");
            check!("aaabb" + [C M W W W] disallows "accaa");
            check!("baaaa" + [W C M W W] allows "aaccc");
            check!("baaaa" + [W C M W W] disallows "caacc");
            check!("abcde" + [W W W W W] disallows "bcdea");
        }

        #[test]
        fn debug() {
            check!("baaaa" + [W C M W W] allows "aaccc");
        }
    }

    mod game {
        use crate::{Guess, Wordle};

        #[test]
        fn genius() {
            let w = Wordle::new();
            let guesser = guesser!(|_history| { "moved".to_string() });
            assert_eq!(w.play("moved", guesser), Some(1))
        }

        #[test]
        fn magnificent() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 1 {
                    return "right".to_string();
                }
                "wrong".to_string()
            });
            assert_eq!(w.play("right", guesser), Some(2))
        }

        #[test]
        fn impressive() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 2 {
                    return "right".to_string();
                }
                "wrong".to_string()
            });
            assert_eq!(w.play("right", guesser), Some(3))
        }

        #[test]
        fn splendid() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 3 {
                    return "right".to_string();
                }
                "wrong".to_string()
            });
            assert_eq!(w.play("right", guesser), Some(4))
        }

        #[test]
        fn great() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 4 {
                    return "right".to_string();
                }
                "wrong".to_string()
            });
            assert_eq!(w.play("right", guesser), Some(5))
        }

        #[test]
        fn phew() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 5 {
                    return "right".to_string();
                }
                "wrong".to_string()
            });
            assert_eq!(w.play("right", guesser), Some(6))
        }

        #[test]
        fn oops() {
            let w = Wordle::new();
            let guesser = guesser!(|history| { "wrong".to_string() });
            assert_eq!(w.play("right", guesser), None)
        }
    }

    mod correctness {
        use crate::Correctness;

        #[test]
        fn all_green() {
            assert_eq!(Correctness::compute("abcde", "abcde"), mask![C C C C C])
        }

        #[test]
        fn all_gray() {
            assert_eq!(Correctness::compute("abcde", "fghij"), mask![W W W W W]);
        }

        #[test]
        fn all_yellow() {
            assert_eq!(Correctness::compute("abcde", "bcdea"), mask![M M M M M]);
        }

        #[test]
        fn repeat_green() {
            assert_eq!(Correctness::compute("aabbb", "aaccc"), mask![C C W W W])
        }

        #[test]
        fn repeat_yellow() {
            assert_eq!(Correctness::compute("aabbb", "ccaac"), mask![W W M M W])
        }

        #[test]
        fn repeat_some_green() {
            assert_eq!(Correctness::compute("aabbb", "caacc"), mask![W C M W W])
        }

        #[test]
        fn mark_only_one() {
            assert_eq!(Correctness::compute("azzaz", "aaabb"), mask![C M W W W])
        }

        #[test]
        fn right_char_wrong_place_dup() {
            assert_eq!(Correctness::compute("baccc", "aaddd"), mask![W C W W W])
        }

        #[test]
        fn duplicate_char() {
            assert_eq!(Correctness::compute("abcde", "aacde"), mask![C W C C C])
        }
    }
}
