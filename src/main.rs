#![feature(test)]
extern crate test;

extern crate ngrams;
extern crate rand;
extern crate rusqlite;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;

use rand::Rng;
use rand::distributions::{Weighted, WeightedChoice, IndependentSample};

use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::Path;

mod bench;
mod ngram;

#[derive(Serialize, Deserialize, Debug)]
pub struct Data{
    start: Option<u32>,
    end: Option<u32>,
    next: Option<HashMap<String, u32>>,
}

type WordData = HashMap<String, Data>;

#[derive(Debug)]
enum Error {
    JsonError(serde_json::Error),
    IoError(io::Error),
}

type Result<T> = std::result::Result<T, Error>;

fn main() {
    let book_data = read_books(Path::new("data/sentences"));
    let books = book_data
        .iter()
        .map(|(ref book, ref content)| ngram::BookNgram::new(&content, book))
        .collect::<Vec<ngram::BookNgram>>();

    let mut input = String::new();
    io::stdin().read_line(&mut input);

    let data = match read_data("word_data.json") {
        Ok(data) => data,
        Err(why) => {
            println!("Could not read data: {:?}", why);
            return
        }
    };

    loop {
        println!("{}", generate(&data));
    }
}

fn write_file(path: &Path, content: String) {
    let mut file = File::create(path).expect(&format!("Could not make file: {:?}", path));

    file.write(content.as_bytes());
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

fn generate(data: &WordData) -> String {
    let starts = data.iter()
        .filter(|&(_, ref v)| v.start.is_some())
        .map(|(k, _)| k.clone())
        .collect::<Vec<String>>();

    let mut rng = rand::thread_rng();
    let mut word = rng.choose(&starts).unwrap();
    let mut output = Vec::new();
    
    loop {
        output.push(word.clone());

        let entry = match data.get(word) {
            Some(v) => v,
            None => {
                println!("{} does not have an entry.", word);
                return output.join(" ")
            }
        };

        let next = match entry.next {
            Some(ref next) => next,
            None => return output.join(" "),
        };

        let mut weights = next.iter()
            .filter_map(|(&ref w, &count)| Some(Weighted { weight: count, item: w }))
            .collect::<Vec<_>>();

        let wc = WeightedChoice::new(&mut weights);
        word = wc.ind_sample(&mut rng);
    }
}

fn read_data(filename: &str) -> Result<WordData> {
    let mut content = String::new();

    let mut file = File::open(filename)
        .map_err(Error::IoError)?;

    file.read_to_string(&mut content)
        .map_err(Error::IoError)?;

    serde_json::from_str::<WordData>(&content)
        .map_err(Error::JsonError)
}