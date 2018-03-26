#![feature(offset_to)]
#![feature(test)]
extern crate test;

#[macro_use] extern crate clap;
#[macro_use] extern crate log;
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

mod bench;
mod ngram;

fn main() {
    pretty_env_logger::init().expect("Could not initialize env logger");
    info!("Generating ngrams...");

    let book_data = read_books(Path::new("data/sentences"));
    let books = ngram::BookNgrams::from_books(&book_data);

    info!("Loaded.");

    let matches = App::new("Ngram Sentence Generator")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::with_name("SERVER")
            .short("s")
            .long("server")
            .help("Runs an IPC server that can send generated sentences"))
        .get_matches();

    if matches.is_present("SERVER") {
        ipc_server(&books);
    } else {
        let results = loop {
            let r = books.generate();
            if r.books.len() >= 4 {
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

fn ipc_server(books: &ngram::BookNgrams) {
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
                if r.books.len() >= 4 {
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