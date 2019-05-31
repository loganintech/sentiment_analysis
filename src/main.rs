#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;
fn main() {
    let (list_of_word_info, word_header) = get_vocab("training_set.txt").unwrap();
    output("preprocessed.txt", word_header, list_of_word_info).unwrap();
}

fn output<T: AsRef<Path>>(path: T, words: Vec<String>, data: Vec<Vec<String>>) -> io::Result<()> {
    let mut outfile = BufWriter::new(File::create(path)?);

    outfile.write_all(format!("{}\n", words.join(", ")).as_bytes())?;
    for values in data {
        outfile.write_all(format!("{}\n", values.join(", ")).as_bytes())?;
    }

    Ok(())
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
        println!("{:?}", line);
        let (sentiment, line_words) = line.split_last().unwrap();

        for word in &words {
            parsed[idx].push((if line_words.contains(&word) { "1" } else { "0" }).to_string());
        }

        parsed[idx].push(sentiment.to_owned());
    }
    words.push("classlabel".to_string());

    Ok((parsed, words))
}
