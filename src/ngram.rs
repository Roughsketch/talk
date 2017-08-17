use std::collections::HashMap;

use ngrams::Ngrams;

#[derive(Debug)]
pub struct NgramData<'a> {
    p_prev: &'a str,
    prev: &'a str,
    current: &'a str,
}

pub struct BookNgram<'a> {
    book: String,
    data: Vec<NgramData<'a>>,
    content: &'a str,
}

impl BookNgram<'a> {
    pub fn new
}

pub fn generate_tuples<'a>(books: &HashMap<String, String>) -> HashMap<String, Vec<NgramData<'a>>> {
    let mut ngrams = HashMap::new();

    for (book, content) in books {
        let res = parse_book(content);
        ngrams.insert(book.clone(), res);
    }

    ngrams
}

fn parse_book<'a>(content: &String) -> Vec<NgramData<'a>> {
    let mut ngrams = Vec::new();

    let lines = content
        .split('\n')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>();

    for line in lines {
        let iter = line.split_whitespace();
        let ngs = Ngrams::new(iter, 3)
            .pad()
            //.map(|v| v.iter().map(|s| s.to_string()).collect())
            .collect::<Vec<Vec<&str>>>();

        for ng in ngs {
            ngrams.push(NgramData {
                current: ng[2],
                prev: ng[1],
                p_prev: ng[0],
            });
        }
    }

    ngrams
}