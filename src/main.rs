#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
fn main() {
    let data = get_vocab("training_set.txt").unwrap();
}

fn get_vocab<T: AsRef<Path>>(path: T) -> io::Result<HashMap<String, (f64, usize)>> {
    let file_data = BufReader::new(File::open(path)?);
    let mut feature_set: HashMap<String, (f64, usize)> = HashMap::new();
    for line in file_data.lines().filter_map(|x| x.ok()) {
        let line: String = line
            .chars()
            .filter(|x| x.is_alphanumeric() || x.is_whitespace())
            .map(|x| x.to_lowercase().to_string())
            .collect::<String>();
        let line = line
            .split_ascii_whitespace()
            .map(std::string::ToString::to_string)
            .collect::<Vec<String>>();

        let (good_or_bad, sentence) = line.split_last().unwrap();
        let good_or_bad = good_or_bad.parse::<f64>().unwrap();
        let sentence = sentence
            .iter()
            .map(std::clone::Clone::clone)
            .collect::<Vec<_>>();

        for word in sentence {
            feature_set
                .entry(word)
                .and_modify(|x| {
                    x.0 += good_or_bad;
                    x.1 += 1
                })
                .or_insert((good_or_bad, 1));
        }
    }

    Ok(feature_set)
}

fn get_features<T: AsRef<Path>>(
    path: T,
    trained_set: HashMap<String, (f64, usize)>,
) -> io::Result<(Vec<String>, Vec<Vec<usize>>)> {
    let file_data = BufReader::new(File::open(path)?);
    let lines = file_data.lines().filter_map(std::result::Result::ok);
    let mut sentences: Vec<Vec<usize>> = vec![];

    let mut keys = trained_set
        .keys()
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();
    keys.sort();
    keys.push("classlabel".to_string());

    for line in lines {
        let line: String = line
            .chars()
            .filter(|x| x.is_alphanumeric() || x.is_whitespace())
            .map(|x| x.to_lowercase().to_string())
            .collect::<String>();
        let line = line
            .split_ascii_whitespace()
            .map(std::string::ToString::to_string)
            .collect::<Vec<String>>();

        let (_, sentence) = line.split_last().unwrap();
        let sentence = sentence
            .into_iter()
            .map(std::clone::Clone::clone)
            .collect::<HashSet<String>>();

        let mut this_sentence: Vec<usize> = Vec::with_capacity(keys.len());
        for word in keys.iter() {
            this_sentence.push(if sentence.contains(word) { 1 } else { 0 });
        }

    }

    Ok((keys, sentences))
}
