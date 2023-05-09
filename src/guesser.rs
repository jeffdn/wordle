use std::{borrow::Cow, collections::HashSet};

macro_rules! mask {
    (C) => {Correctness::Correct};
    (M) => {Correctness::Misplaced};
    (W) => {Correctness::Wrong};
    ($($c:tt)+) => {[
         $(mask!($c)),+
    ]};
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Correctness {
    Correct,
    Misplaced,
    Wrong,
}

impl Correctness {
    fn compute(answer: &str, word: &str) -> [Self; 5] {
        let mut c = [Correctness::Wrong; 5];
        let mut used = [false; 5];

        for (i, (a, g)) in answer.bytes().zip(word.bytes()).enumerate() {
            if a == g {
                c[i] = Correctness::Correct;
                used[i] = true;
            }
        }

        for (i, g) in word.bytes().enumerate() {
            if c[i] == Correctness::Correct {
                continue;
            }

            if answer.bytes().enumerate().any(|(i, a)| {
                if a == g && !used[i] {
                    used[i] = true;
                    return true;
                }

                false
            }) {
                c[i] = Correctness::Misplaced;
            }
        }

        c
    }
}

#[derive(Clone, Copy)]
struct Guess<'a> {
    word: &'a str,
    mask: [Correctness; 5],
}

impl<'a> Guess<'a> {
    fn check(answer: &'a str, word: &'a str) -> Self {
        Self {
            word,
            mask: Correctness::compute(answer, word),
        }
    }

    fn matches(&self, word: &str) -> bool {
        let mut used = [false; 5];

        'outer: for (i, ((g, &m), w)) in self
            .word
            .bytes()
            .zip(&self.mask)
            .zip(word.bytes())
            .enumerate()
        {
            if m == Correctness::Correct {
                if w != g {
                    return false;
                } else {
                    used[i] = true;
                    continue;
                }
            }

            for (j, (g_i, &m_i)) in self.word.bytes().zip(&self.mask).enumerate() {
                if g_i != w || used[j] {
                    continue;
                }

                match m_i {
                    Correctness::Correct => continue,
                    Correctness::Misplaced if j != i => {
                        used[j] = true;
                        continue 'outer;
                    },
                    _ => {
                        return false;
                    },
                }
            }
        }

        for (&m, u) in self.mask.iter().zip(&used) {
            if m == Correctness::Misplaced && !u {
                return false;
            }
        }

        true
    }

    #[inline]
    fn is_correct(&self) -> bool {
        self.mask == mask![C C C C C]
    }
}

pub(crate) struct Guesser<'a> {
    answer: &'a str,
    dictionary: Cow<'a, Vec<&'a str>>,
    exclusions: &'a HashSet<&'a str>,
    history: [Option<Guess<'a>>; 6],
}

impl<'a> Guesser<'a> {
    pub(crate) fn new(
        answer: &'a str,
        dictionary: &'a Vec<&'a str>,
        exclusions: &'a HashSet<&'a str>,
    ) -> Self {
        Self {
            answer,
            dictionary: Cow::Borrowed(dictionary),
            exclusions,
            history: [None; 6],
        }
    }

    pub(crate) fn solve(&mut self) -> Option<usize> {
        let mut current_word = "salet";

        for i in 0..6 {
            let guess = Guess::check(self.answer, current_word);

            if guess.is_correct() {
                return Some(i + 1);
            }

            match &mut self.dictionary {
                Cow::Borrowed(_) => {
                    self.dictionary = Cow::Owned(
                        self.dictionary
                            .iter()
                            .filter_map(|word| {
                                (guess.matches(word) && !self.exclusions.contains(word))
                                    .then(|| *word)
                            })
                            .collect(),
                    );
                },
                Cow::Owned(dict) => {
                    dict.retain(|word| guess.matches(word) && !self.exclusions.contains(word))
                },
            };

            self.history[i] = Some(guess);

            if self.dictionary.is_empty() {
                break;
            }

            current_word = self.dictionary[0];
        }

        None
    }

    pub(crate) fn guessed_words(&self) -> Vec<&str> {
        self.history.iter().map(|og| og.unwrap().word).collect()
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn all_correct() {
        assert_eq!(Correctness::compute("stare", "stare"), mask![C C C C C]);
    }

    #[test]
    fn all_misplaced() {
        assert_eq!(Correctness::compute("stare", "tares"), mask![M M M M M]);
    }

    #[test]
    fn all_wrong() {
        assert_eq!(Correctness::compute("stare", "chomp"), mask![W W W W W]);
    }

    #[test]
    fn mixed() {
        assert_eq!(Correctness::compute("tares", "tardy"), mask![C C C W W]);
        assert_eq!(Correctness::compute("party", "tardy"), mask![M C C W C]);
    }

    #[test]
    fn plausibility_imply() {
        let answer = "imply";
        let guess_word = "gypsy";
        let guess = Guess::check(answer, guess_word);

        assert!(!guess.matches("nymph"));
        assert!(guess.matches("amply"));
    }

    #[test]
    fn plausibility_close() {
        let answer = "ccccc";
        let guess_word = "ccccg";
        let guess = Guess::check(answer, guess_word);

        assert!(guess.matches("ccccc"));
        assert!(guess.matches("ccccz"));
    }

    #[test]
    fn plausibility_racer() {
        let answer = "racer";
        let guess_word = "tares";
        let guess = Guess::check(answer, guess_word);

        assert!(guess.matches("pacer"));
        assert!(guess.matches("raced"));
        assert!(!guess.matches("races"));
    }

    #[test]
    fn plausibility_requires_misplaced() {
        let answer = "islet";
        let guess_word = "tares";
        let guess = Guess::check(answer, guess_word);

        // As we have the 's', but misplaced, all subsequent guesses should have
        // an 's', and in a different position.
        assert!(!guess.matches("given"));
        assert!(!guess.matches("model"));
        assert!(!guess.matches("chief"));
        assert!(guess.matches("islet"));
    }
}
