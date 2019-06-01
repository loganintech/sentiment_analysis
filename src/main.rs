#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::env::args;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;

fn main() {
    // let (list_of_word_info, word_header) = get_vocab("training_set.txt").unwrap();
    // output("preprocessed.txt", &word_header, &list_of_word_info).unwrap();
    // let formatted = dbg!(format_preprocessed(word_header, list_of_word_info));

    let input_file = args()
        .nth(1)
        .expect("You need to include a text file to test against.");

    let formatted = load_preprocessed_from_file("preprocessed.txt").unwrap();
    let test_sentences = get_sentences(input_file).expect("Couldn't load the sentences file.");
    let sentences = test_sentences.len() as f64;
    let good_count = test_sentences.iter().filter(|(_, y)| *y).count();
    let bad_count = test_sentences.iter().filter(|(_, y)| !*y).count();
    let mut correct_guesses = 0.0;

    println!("Classifying!");
    for (line, is_good) in test_sentences {
        if classify(&formatted, (&line, is_good), (good_count, bad_count)) {
            correct_guesses += 1.0;
        };
    }

    println!(
        "Correct Guesses Percent: {:2.2}%",
        (correct_guesses / sentences) * 100.0
    );
}

fn classify(data: &DataSet, evidence: (&[String], bool), quality_count: (usize, usize)) -> bool {
    let (sentence, is_good) = evidence;
    let (map, _) = data;

    let (mut probability_good, mut probability_bad) = (1.0, 1.0);

    for word in sentence {
        probability_bad *= map.get(word).unwrap_or(&(1, 0)).0 as f64;
        probability_good *= map.get(word).unwrap_or(&(0, 1)).1 as f64;
    }

    probability_good /= quality_count.0 as f64;
    probability_bad /= quality_count.1 as f64;

    let guess = probability_good > probability_bad;

    // assert_eq!(guess, is_good);
    guess == is_good
}

fn get_sentences<T: AsRef<Path>>(path: T) -> io::Result<Vec<(Vec<String>, bool)>> {
    let file_data = BufReader::new(File::open(path)?);
    let mut lines: Vec<(Vec<String>, bool)> = vec![];

    for line in file_data.lines().filter_map(Result::ok) {
        let line = line
            .split_ascii_whitespace()
            .map(|x| x.chars().filter(|y| char::is_alphanumeric(*y)).collect())
            .map(|x: String| x.to_lowercase())
            .collect::<Vec<String>>();

        let (last, line) = line.split_last().unwrap();

        lines.push((line.to_owned(), last.parse::<usize>().unwrap() == 1));
    }

    Ok(lines)
}

fn output<T: AsRef<Path>>(path: T, words: &[String], data: &[Vec<String>]) -> io::Result<()> {
    let mut outfile = BufWriter::new(File::create(path)?);

    outfile.write_all(format!("{}\n", words.join(", ")).as_bytes())?;
    for values in data {
        outfile.write_all(format!("{}\n", values.join(", ")).as_bytes())?;
    }

    Ok(())
}

type DataSet = (HashMap<String, (usize, usize)>, usize);

fn load_preprocessed_from_file<T: AsRef<Path>>(path: T) -> io::Result<DataSet> {
    let file_data = BufReader::new(File::open(path)?);
    let mut lines = file_data.lines().filter_map(Result::ok);
    let header = lines
        .next()
        .unwrap()
        .split(", ")
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();
    let data = lines
        .map(|x: String| {
            x.split(", ")
                .map(std::string::ToString::to_string)
                .collect::<Vec<String>>()
        })
        .collect::<Vec<Vec<String>>>();

    Ok(format_preprocessed(header, data))
}

fn format_preprocessed(words: Vec<String>, data: Vec<Vec<String>>) -> DataSet {
    let (mut map, length): DataSet = (HashMap::new(), words.len());

    for (word_idx, word) in words.iter().enumerate() {
        for sentence in &data {
            map.entry(word.clone())
                .and_modify(|(bad, good)| {
                    if sentence[word_idx].parse::<usize>().unwrap() == 1 {
                        if sentence.last().unwrap().parse::<usize>().unwrap() == 1 {
                            *good += 1;
                        } else {
                            *bad += 1;
                        }
                    }
                })
                .or_insert_with(|| {
                    if sentence.last().unwrap().parse::<usize>().unwrap() == 1 {
                        (0, 1)
                    } else {
                        (1, 0)
                    }
                });
        }
    }

    (map, length)
}

fn get_vocab<T: AsRef<Path>>(path: T) -> io::Result<(Vec<Vec<String>>, Vec<String>)> {
    let file_data = BufReader::new(File::open(path)?);
    let mut header: HashSet<String> = HashSet::new();
    let mut lines: Vec<Vec<String>> = vec![];

    for line in file_data.lines().filter_map(Result::ok) {
        let line = line
            .split_ascii_whitespace()
            .map(|x| x.chars().filter(|y| char::is_alphanumeric(*y)).collect())
            .map(|x: String| x.to_lowercase())
            .collect::<Vec<String>>();

        for word in &line {
            if !header.contains(word) {
                header.insert(word.to_string());
            }
        }

        lines.push(line);
    }

    let mut words: Vec<String> = header.into_iter().collect::<Vec<String>>();
    words.sort();
    words.remove(0);

    let mut parsed: Vec<Vec<String>> = vec![vec![]; lines.len()];
    for (idx, line) in lines.iter().enumerate() {
        let (sentiment, line_words) = line.split_last().unwrap();

        for word in &words {
            parsed[idx].push((if line_words.contains(&word) { "1" } else { "0" }).to_string());
        }

        parsed[idx].push(sentiment.to_owned());
    }
    words.push("classlabel".to_string());

    Ok((parsed, words))
}
