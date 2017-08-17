use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use ngrams::Ngrams;

#[derive(Debug)]
pub struct NgramData {
    p_prev: String,
    prev: String,
    current: String,
}

pub fn generate_tuples(path: &Path) -> HashMap<String, Vec<NgramData>> {
    let mut ngrams = HashMap::new();
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
            let mut res = parse_file(entry.path().as_path());
            ngrams.insert(book.into(), res);
        }
    }

    ngrams
}

fn parse_file(entry: &Path) -> Vec<NgramData> {
    let mut ngrams = Vec::new();

    let mut file = File::open(entry).unwrap();
    let mut contents = String::new();

    let _ = file.read_to_string(&mut contents);
    contents = contents.replace("\r", "");

    let lines = contents
        .split('\n')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>();

    for line in lines {
        let iter = line.split_whitespace();
        let ngs = Ngrams::new(iter, 3)
            .pad()
            .map(|v| v.iter().map(|s| s.to_string()).collect())
            .collect::<Vec<Vec<String>>>();

        for ng in ngs {
            ngrams.push(NgramData {
                current: ng[2].clone(),
                prev: ng[1].clone(),
                p_prev: ng[0].clone(),
            });
        }
    }

    ngrams
}