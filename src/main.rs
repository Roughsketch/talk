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
    let set = read_set(Path::new("data/sentences"));
    println!("{:?} - {}", set, set.len());

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

fn read_set(path: &Path) -> HashMap<String, ngram::BookNgram> {
    let mut output = HashMap::new();

    let books = read_books(path);

    for (book, content) in books {
        let unique = unique_tokens(&content);
        let map = map_tokens(&unique, &content);
        output.insert(book, ngram::BookNgram::new(unique, map, book));
    }

    output
}

fn unique_tokens(content: &String) -> HashSet<String> {
    let mut unique = HashSet::new();

    let mut words = content.split_whitespace().collect::<Vec<&str>>();
    words.sort();
    words.dedup_by(|a, b| a == b);

    for word in words {
        unique.insert(word.into());
    }

    unique
}

fn map_tokens<'a>(dict: &'a HashSet<String>, content: &String) -> Vec<Vec<&'a String>> {
    content
        .split("\n")
        .filter(|line| !line.is_empty())
        .map(|line| {
            line.split_whitespace()
                .map(|word| {
                    dict.get(word).unwrap()
                })
                .collect()
        })
        .collect()
}