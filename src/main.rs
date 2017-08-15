extern crate rusqlite;
extern crate serde_json;

use serde_json::Value;

use std::fs::File;
use std::io;
use std::io::Read;

fn main() {
    let mut file = File::open("word_data.json")
        .expect("Cannot read word_data.json");

    let mut input = String::new();
    let mut content = String::new();

    file.read_to_string(&mut content)
        .expect("Cannot read file.");

    let data = serde_json::from_str::<Value>(&content)
        .expect("Cannot deserialize data");

    for (k, _) in data.as_object().unwrap().iter() {
        println!("{:?}", k);

        io::stdin().read_line(&mut input)
            .expect("Cannot read stdin.");
    }
}
