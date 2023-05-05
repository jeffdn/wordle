use std::borrow::Cow;

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

    #[inline]
    fn is_correct(&self) -> bool {
        self.mask == mask![C C C C C]
    }
}

pub(crate) struct Guesser<'a> {
    answer: &'a str,
    dictionary: Cow<'a, Vec<&'a str>>,
    history: [Option<Guess<'a>>; 6],
    guesses: usize,
}

fn _word_filter(guess: &Guess, word: &str) -> bool {
    let mut used = [false; 5];

    for (i, ((g, &m), w)) in guess
        .word
        .bytes()
        .zip(&guess.mask)
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

        let mut plausible = true;

        if guess
            .word
            .bytes()
            .zip(&guess.mask)
            .enumerate()
            .any(|(j, (g_i, &m_i))| {
                if g_i != w || used[j] {
                    return false;
                }

                match m_i {
                    Correctness::Correct => false,
                    Correctness::Misplaced if j != i => {
                        used[j] = true;
                        true
                    },
                    _ => {
                        plausible = false;
                        false
                    },
                }
            })
        {
        } else if !plausible {
            return false;
        }
    }

    for (&m, u) in guess.mask.iter().zip(&used) {
        if m == Correctness::Misplaced && !u {
            return false;
        }
    }

    true
}

impl<'a> Guesser<'a> {
    pub(crate) fn new(answer: &'a str, dictionary: &'a Vec<&'a str>) -> Self {
        Self {
            answer,
            dictionary: Cow::Borrowed(dictionary),
            history: [None; 6],
            guesses: 0,
        }
    }

    pub(crate) fn solve(&mut self) -> Option<(&'a str, usize)> {
        let mut current_word = "tares";

        loop {
            let guess = Guess::check(self.answer, current_word);
            self.guesses += 1;

            if self.guesses > 6 {
                break None;
            }

            if guess.is_correct() {
                break Some((guess.word, self.guesses));
            }

            match &mut self.dictionary {
                Cow::Borrowed(_) => {
                    self.dictionary = Cow::Owned(
                        self.dictionary
                            .iter()
                            .filter(|word| _word_filter(&guess, word))
                            .map(|word| *word)
                            .collect(),
                    );
                },
                Cow::Owned(dict) => dict.retain(|word| _word_filter(&guess, word)),
            };

            self.history[self.guesses - 1] = Some(guess);

            if self.dictionary.is_empty() {
                break None;
            }

            current_word = self.dictionary[0];
        }
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

        assert!(!_word_filter(&guess, "nymph"));
        assert!(_word_filter(&guess, "amply"));
    }

    #[test]
    fn plausibility_close() {
        let answer = "ccccc";
        let guess_word = "ccccg";
        let guess = Guess::check(answer, guess_word);

        assert!(_word_filter(&guess, "ccccc"));
        assert!(_word_filter(&guess, "ccccz"));
    }

    #[test]
    fn plausibility_requires_misplaced() {
        let answer = "islet";
        let guess_word = "tares";
        let guess = Guess::check(answer, guess_word);

        // As we have the 's', but misplaced, all subsequent guesses should have
        // an 's', and in a different position.
        assert!(!_word_filter(&guess, "given"));
        assert!(!_word_filter(&guess, "model"));
        assert!(!_word_filter(&guess, "chief"));
        assert!(_word_filter(&guess, "islet"));
    }
}
