use std::collections::HashSet;

mod guesser;

static ANSWERS: &str = include_str!("../answers.txt");
static DICTIONARY: &str = include_str!("../corpus/word-counts.txt");

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

    let mut count = 0;
    let mut score = 0;
    let mut wrong = 0;
    let mut exclusions: HashSet<&str> = HashSet::new();

    for answer in answers.iter() {
        let mut guesser = crate::guesser::Guesser::new(answer, &dictionary, &exclusions);

        match guesser.solve() {
            Some(guess_count) => {
                count += 1;
                score += guess_count;
                exclusions.insert(answer);
            },
            _ => {
                println!("{answer}: {:?}", guesser.guessed_words());
                wrong += 1;
            },
        };
    }

    println!("average score: {}", score as f32 / count as f32);
    println!("missed words: {}", wrong);
}
