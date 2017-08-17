use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use ngrams::Ngrams;

#[derive(Debug)]
pub struct NgramData<'a> {
    p_prev: &'a str,
    prev: &'a str,
    current: &'a str,
}

#[derive(Debug)]
pub struct BookNgram<'a> {
    book: String,
    data: Vec<NgramData<'a>>,
    content: &'a String,
}

impl <'a> BookNgram<'a> {
    pub fn new(content: &'a String, book: String) -> BookNgram<'a> {
        let mut data = Vec::new();

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
                data.push(NgramData {
                    current: ng[2],
                    prev: ng[1],
                    p_prev: ng[0],
                });
            }
        }

        BookNgram {
            book: book,
            content: content,
            data: data,
        }
    }
}
