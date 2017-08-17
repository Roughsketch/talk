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
use std::io::Read;
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
    let books = read_books(Path::new("test"));
    let ngrams = ngram::generate_tuples(&books);
    println!("Test:\n{:?}", ngrams);

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

fn read_books(path: &Path) -> HashMap<String, String> {
    let books = HashMap::new();
    let dir = path.read_dir().unwrap();

    for entry in dir.filter_map(|e| e.ok()) {
        let meta = match entry.metadata() {
            Ok(meta) => meta,
            Err(_) => continue,
        };

        if meta.is_file() {
            let path = entry.path();
            let os = path.as_path().file_stem().unwrap();
            let book = os.to_str().expect("Could not convert OsStr");

            let mut file = File::open(entry.path()).unwrap();
            let mut contents = String::new();

            let _ = file.read_to_string(&mut contents);
            contents = contents.replace("\r", "");

            books.insert(book.into(), contents);
        }
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