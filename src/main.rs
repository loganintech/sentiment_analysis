#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::env::args;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

static mut PROGRAM_START: SystemTime = UNIX_EPOCH;

fn main() {
    unsafe { PROGRAM_START = SystemTime::now() };
    let vocab_file = args().nth(1).unwrap_or_else(|| {
        eprintln!("You need to include a source file for the classifier.");
        std::process::exit(1)
    });

    let test_file = args().nth(2).unwrap_or_else(|| {
        eprintln!("You need to include a text file to test against.");
        std::process::exit(2)
    });

    log_to_results(
        &format!("Loading classifier data from {}", vocab_file),
        true,
    )
    .unwrap();
    let (list_of_word_info, word_header) = get_vocab(vocab_file).unwrap();
    log_to_results("Outputting preprocessed data.", true).unwrap();
    output("preprocessed.txt", &word_header, &list_of_word_info).unwrap();

    // log_to_results("Loading preprocessed data from preprocessed.txt").unwrap();
    // let formatted = load_preprocessed_from_file("preprocessed.txt").unwrap();

    let formatted = format_preprocessed(word_header, list_of_word_info);

    log_to_results(&format!("Loading test sentences from {}", test_file), true).unwrap();
    let test_sentences = get_sentences(test_file).expect("Couldn't load the sentences file.");
    let sentences = test_sentences.len() as f64;
    let good_count = test_sentences
        .iter()
        .filter(|(_, is_good)| *is_good)
        .count();
    let bad_count = test_sentences
        .iter()
        .filter(|(_, is_good)| !*is_good) // Strictly speaking the * is not needed here (! will dereference a &bool) but it is more consistent to include it
        .count();
    let mut correct_guesses = 0.0;

    log_to_results("Classifying!", true).unwrap();
    for (line, sentence_is_good) in test_sentences {
        let sentiment = classify(
            &formatted,
            &line,
            SentimentCount {
                good: good_count,
                bad: bad_count,
            },
        );
        if (sentiment == Sentiment::Good && sentence_is_good)
            || (sentiment == Sentiment::Bad && !sentence_is_good)
        {
            correct_guesses += 1.0;
        }
    }

    log_to_results(
        &format!(
            "Correctly Classified: {:2.2}%",
            (correct_guesses / sentences) * 100.0
        ),
        true,
    )
    .unwrap();
}

#[derive(PartialEq, Copy, Clone)]
enum Sentiment {
    Good,
    Bad,
}

impl Sentiment {
    fn into_storage(&self) -> &'static str {
        use Sentiment::*;
        match self {
            Good => "1",
            Bad => "0",
        }
    }
}

fn classify(data: &DataSet, evidence: &[String], quality_count: SentimentCount) -> Sentiment {
    let sentence = evidence;
    let (map, _) = data;

    let (mut probability_good, mut probability_bad) = (1.0, 1.0);

    for word in sentence {
        probability_bad *= map
            .get(word)
            .unwrap_or(&SentimentCount { good: 0, bad: 1 })
            .bad as f64;
        probability_good *= map
            .get(word)
            .unwrap_or(&SentimentCount { good: 1, bad: 0 })
            .good as f64;
    }

    probability_good /= quality_count.good as f64;
    probability_bad /= quality_count.bad as f64;

    let guess = probability_good > probability_bad;

    // assert_eq!(guess, is_good);
    if guess {
        Sentiment::Good
    } else {
        Sentiment::Bad
    }
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

struct SentimentCount {
    good: usize,
    bad: usize,
}

type DataSet = (HashMap<String, SentimentCount>, usize);

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
                .and_modify(|SentimentCount { bad, good }| {
                    if sentence[word_idx] == "1" {
                        if sentence.last().unwrap() == "1" {
                            *good += 1;
                        } else {
                            *bad += 1;
                        }
                    }
                })
                .or_insert_with(|| {
                    if sentence.last().unwrap() == "1" {
                        SentimentCount { good: 1, bad: 0 }
                    } else {
                        SentimentCount { good: 0, bad: 1 }
                    }
                });
        }
    }

    (map, length)
}

fn log_to_results<T: std::fmt::Display>(data: T, also_print: bool) -> io::Result<()> {
    let mut out = BufWriter::new(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open("results.txt")?,
    );

    let wr = format!(
        "[{:<3}ms] {}\n",
        std::time::SystemTime::now()
            .duration_since(unsafe { PROGRAM_START })
            .unwrap()
            .as_millis(),
        data
    );

    if also_print {
        print!("{}", wr);
    }

    out.write_all(wr.as_bytes())?;

    Ok(())
}

fn get_vocab<T: AsRef<Path>>(path: T) -> io::Result<(Vec<Vec<String>>, Vec<String>)> {
    let file_data = BufReader::new(File::open(path)?);
    let mut header: HashSet<String> = HashSet::new();
    let mut lines: Vec<Vec<String>> = vec![];

    for line in file_data.lines().filter_map(Result::ok) {
        let line = line
            .split_ascii_whitespace()
            .map(|s| {
                s.chars()
                    .filter(|y| char::is_alphanumeric(*y))
                    .collect::<String>()
            })
            .map(|s| s.to_lowercase())
            .collect::<Vec<String>>();

        for word in &line {
            if !header.contains(word) {
                header.insert(word.clone());
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
