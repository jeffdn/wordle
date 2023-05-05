mod guesser;

static ANSWERS: &str = include_str!("../answers.txt");
static DICTIONARY: &str = include_str!("../corpus/word-counts.txt");

fn main() {
    let answers: Vec<&str> = ANSWERS.split_ascii_whitespace().collect();
    let dictionary: Vec<&str> = {
        let mut pairs: Vec<(&str, usize)> = DICTIONARY
            .split('\n')
            .filter_map(|pair| {
                let mut split = pair.splitn(2, ' ');

                match (split.next(), split.next()) {
                    (Some(word), Some(count_str)) => count_str.parse().map(|c| (word, c)).ok(),
                    _ => None,
                }
            })
            .collect();
        pairs.sort_by_key(|&(_, count)| std::cmp::Reverse(count));
        pairs.into_iter().map(|(word, _)| word).collect()
    };

    let mut count = 0;
    let mut score = 0;

    for answer in answers.iter().take(100) {
        let mut guesser = crate::guesser::Guesser::new(answer, &dictionary);
        count += 1;

        match guesser.solve() {
            Some((word, guess_count)) => {
                println!("{answer}: {word} in {guess_count}");
                score += guess_count;
            },
            _ => println!("{answer}: unsolved"),
        };
    }

    println!("average score: {}", score as f32 / count as f32);
}
