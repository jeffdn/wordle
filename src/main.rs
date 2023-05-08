use std::collections::HashMap;

mod guesser;

static ANSWERS: &str = include_str!("../answers.txt");
static DICTIONARY: &str = include_str!("../corpus/word-counts.txt");
static LETTERS: &str = include_str!("../corpus/letter-frequency.txt");

fn main() {
    let answers: Vec<&str> = ANSWERS.split_ascii_whitespace().collect();
    let dictionary: Vec<&str> = {
        let mut pairs: Vec<(&str, usize)> = DICTIONARY
            .split('\n')
            .filter_map(|pair| match pair.split_once(' ') {
                Some((word, count_str)) => count_str.parse().map(|c| (word, c)).ok(),
                _ => None,
            })
            .collect();
        pairs.sort_by_key(|&(_, count)| std::cmp::Reverse(count));
        pairs.into_iter().map(|(word, _)| word).collect()
    };
    let letters: HashMap<u8, f32> = LETTERS
        .split('\n')
        .filter_map(|pair| match pair.split_once(' ') {
            Some((letter, count_str)) => count_str
                .parse()
                .map(|c| (letter.bytes().nth(0).unwrap(), c))
                .ok(),
            _ => None,
        })
        .collect();

    let mut count = 0;
    let mut score = 0;
    let mut unsolved_count = 0;

    for answer in answers.iter() {
        let mut guesser = crate::guesser::Guesser::new(answer, &dictionary, &letters);

        match guesser.solve() {
            Some((_, guess_count)) => {
                // println!("{answer}: {word} in {guess_count}");
                score += guess_count;
                count += 1;
            },
            _ => {
                unsolved_count += 1;
                println!("{answer}: unsolved -- {:?}", guesser.guessed_words());
            },
        };
    }

    println!("average score: {}", score as f32 / count as f32);
    println!("unsolved count: {}", unsolved_count);
}
