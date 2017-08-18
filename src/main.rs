#![feature(test)]
extern crate test;

extern crate rusqlite;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;

mod bench;
mod ngram;

fn main() {
    let books = read_books(Path::new("data/sentences"));
    let mut dicts = Vec::new();
    let mut data = Vec::new();
    let mut ngrams = HashMap::new();


    for (book, content) in books {
        let unique = unique_tokens(&content);
        data.push((book, content));
        dicts.push(unique);
    }

    for (index, &(ref book, ref content)) in data.iter().enumerate() {
        ngrams.insert(book, ngram::BookNgram::new(&dicts[index], &content));
    }

    println!("{:?} - {}", ngrams, ngrams.len());

    let mut input = String::new();
    io::stdin().read_line(&mut input);
}

fn read_file(path: &Path) -> String {
    let mut file = File::open(path).expect(&format!("No such file: {:?}", path));
    let mut content = String::new();

    let _ = file.read_to_string(&mut content);
    content.replace("\r", "")
}

fn read_books(path: &Path) -> HashMap<String, String> {
    let mut books = HashMap::new();

    let dir = path
        .read_dir().unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.metadata().is_ok())
        .filter(|e| e.metadata().unwrap().is_file());

    for file in dir {
        let path = file.path();
        let os = path.as_path().file_stem().unwrap();
        let book = os.to_str().expect("Could not convert OsStr");
        let content = read_file(&path);

        books.insert(book.into(), content);
    }

    books
}

fn unique_tokens(content: &String) -> HashSet<String> {
    let mut unique = HashSet::new();

    unique.insert(String::from("")); // Empty value

    let mut words = content.split_whitespace().collect::<Vec<&str>>();
    words.sort();
    words.dedup_by(|a, b| a == b);

    for word in words {
        unique.insert(word.into());
    }

    unique
}
