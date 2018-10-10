#![feature(ptr_offset_from)]
#![feature(test)]
extern crate test;

extern crate bincode;
#[macro_use] extern crate clap;
#[macro_use] extern crate log;
#[cfg(target_os = "linux")]
extern crate nanomsg;
extern crate ngrams;
extern crate pretty_env_logger;
extern crate rand;
extern crate rayon;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use clap::{App, Arg};

mod ngram;

fn main() {
    pretty_env_logger::init();
    info!("Generating ngrams...");

    let matches = App::new("Ngram Sentence Generator")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::with_name("SERVER")
            .short("s")
            .long("server")
            .help("Runs an IPC server that can send generated sentences"))
        .arg(Arg::with_name("COMPILE")
            .short("c")
            .long("compile")
            .takes_value(true)
            .help("Compiles a folder of sen files into something that can be deserialized quickly."))
        .arg(Arg::with_name("LOAD")
            .short("l")
            .long("load")
            .takes_value(true)
            .help("Loads data from a previously compiled set."))
        .arg(Arg::with_name("DIRECTORY")
            .short("d")
            .long("dir")
            .takes_value(true)
            .help("Directory to read sentence data from."))
        .arg(Arg::with_name("UNIQUE")
            .short("u")
            .long("unique")
            .takes_value(true)
            .help("How many unique sources must be taken from to produce a valid output."))
        .get_matches();

    info!("Generating ngrams...");

    let file = matches
        .value_of("LOAD")
        .unwrap_or("")
        .to_owned();

    if !file.is_empty() {
        let mut file = File::open(file).unwrap();
        let mut buffer = Vec::new();

        if let Err(why) = file.read_to_end(&mut buffer) {
            error!("Could not read file: {:?}", why);
            return;
        }

        let books = bincode::deserialize::<ngram::BookNgrams>(&buffer).unwrap();

        handle_matches(matches, &books);
    } else {
        let book_data = read_books(matches.value_of("LOAD").unwrap_or("data/sentences"));
        let books = ngram::BookNgrams::from_books(&book_data);

        handle_matches(matches, &books);
    }
}

fn handle_matches(matches: clap::ArgMatches, books: &ngram::BookNgrams) {
    info!("Loaded.");

    if let Some(file) = matches.value_of("COMPILE") {
        let mut file = File::create(file).unwrap();

        if let Err(why) = file.write(&bincode::serialize(&books).unwrap()) {
            error!("Could not write file: {:?}", why);
            return;
        }
    }

    let unique = {
        let s = matches.value_of("UNIQUE").unwrap_or("1");
        s.parse::<usize>().unwrap_or(1)
    };

    if matches.is_present("SERVER") {
        ipc_server(&books, unique);
    } else {
        let results = loop {
            let r = books.generate();
            if r.books.len() >= unique {
                break r;
            }
        };

        println!("{}", results);
    }
}

fn read_file(path: &Path) -> String {
    let mut file = File::open(path)
        .expect(&format!("No such file: {:?}", path));
    let mut content = String::new();

    let _ = file.read_to_string(&mut content);
    content.replace("\r", "")
}

fn read_books<P: AsRef<Path>>(path: P) -> HashMap<String, String> {
    let mut books = HashMap::new();

    let dir = path.as_ref()
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
#[cfg(not(target_os = "linux"))]
fn ipc_server(_books: &ngram::BookNgrams, unique: usize) {
    println!("Server is not supported on Windows.");
}

#[cfg(target_os = "linux")]
fn ipc_server(books: &ngram::BookNgrams, unique: usize) {
    let mut socket = nanomsg::Socket::new(nanomsg::Protocol::Rep)
        .expect("Could not create IPC socket.");
    
    let _endpoint = socket.bind("ipc:///tmp/talk.ipc")
        .expect("Could not bind to IPC endpoint.");

    let mut msg = String::new();
    loop {
        if let Err(why) = socket.read_to_string(&mut msg) {
            error!("Error reading from socket: {:?}", why);
            continue;
        }

        trace!("Got payload: '{}'", msg);

        if msg == "gen" {
            let results = loop {
                let r = books.generate();
                if r.books.len() >= unique {
                    break r;
                }
            };
            
            let json = match serde_json::to_string(&results) {
                Ok(json) => json,
                Err(why) => {
                    error!("Could not serialize results: {:?}", why);
                    msg.clear();
                    continue;
                }
            };

            if let Err(why) = socket.write(&json.as_bytes()) {
                error!("Could not write to socket: {:?}", why);
            }
        }

        msg.clear();
    }
}