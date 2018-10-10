use std::collections::{HashMap, HashSet};
use std::fmt;
use std::iter::FromIterator;

use ngrams::Ngrams;
use rand::{self, Rng};
use rayon::prelude::*;
use serde_derive::{Deserialize, Serialize};

const WORD_SEP: &'static str = "";

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Offset(u32);

impl Offset {
    fn new(offset: u32, length: u8) -> Self {
        Offset((offset & 0x00FFFFFF) | ((length as u32) << 24))
    }

    fn len(&self) -> usize {
        (self.0 >> 24) as usize
    }

    fn ptr(&self) -> usize {
        (self.0 & 0x00FFFFFF) as usize
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NgramData {
    pp_prev: Offset,
    p_prev: Offset,
    prev: Offset,
    current: Offset,
}

impl NgramData {
    pub fn pp_prev<'a>(&self, content: &'a str) -> &'a str {
        if self.pp_prev.len() == 0 {
            return WORD_SEP
        }

        let index = self.pp_prev.ptr();
        let end = index + self.pp_prev.len();
        &content[index..end]
    }

    pub fn p_prev<'a>(&self, content: &'a str) -> &'a str {
        if self.p_prev.len() == 0 {
            return WORD_SEP
        }

        let index = self.p_prev.ptr();
        let end = index + self.p_prev.len();
        &content[index..end]
    }

    pub fn prev<'a>(&self, content: &'a str) -> &'a str {
        if self.prev.len() == 0 {
            return WORD_SEP
        }
        
        let index = self.prev.ptr();
        let end = index + self.prev.len();
        &content[index..end]
    }

    pub fn current<'a>(&self, content: &'a str) -> &'a str {
        if self.current.len() == 0 {
            return WORD_SEP
        }
        
        let index = self.current.ptr();
        let end = index + self.current.len();
        &content[index..end]
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BookNgram<'a> {
    book: &'a str,
    pub data: Vec<NgramData>,
    content: &'a str,
}

impl <'a> BookNgram<'a> {
    pub fn new(content: &'a str, book: &'a str) -> BookNgram<'a> {
        let mut data = Vec::new();
        let start = content.as_ptr();

        let lines = content
            .split('\n')
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>();

        for line in lines {
            let ngs = Ngrams::new(line.split_whitespace(), 4)
                .pad()
                .collect::<Vec<Vec<&str>>>();

            for ng in ngs {
                if !(ng[2] == WORD_SEP && ng[3] == WORD_SEP) {
                    unsafe {
                        data.push(NgramData {
                            current: ng[3].as_ptr().offset_from(start) as u32,
                            current_len: word_length(ng[3]),
                            prev: ng[2].as_ptr().offset_from(start) as u32,
                            prev_len: word_length(ng[2]),
                            p_prev: ng[1].as_ptr().offset_from(start) as u32,
                            p_prev_len: word_length(ng[1]),
                            pp_prev: ng[0].as_ptr().offset_from(start) as u32,
                            pp_prev_len: word_length(ng[0]),
                        });
                    }
                }
            }
        }

        BookNgram { book, content, data }
    }

    fn search(&self, pp_prev: &str, p_prev: &str, prev: &str) -> Vec<NgramEntry<'a>> {
        self.data
            .par_iter()
            .filter(|entry| {
                entry.pp_prev(self.content) == pp_prev &&
                entry.p_prev(self.content) == p_prev && 
                entry.prev(self.content) == prev
            })
            .map(|entry| {
                NgramEntry {
                    book: self.book,
                    ngram: entry.clone(),
                }
            })
            .collect()
    }
}

fn word_length(slice: &str) -> u8 {
    if slice == WORD_SEP {
        0
    } else {
        if slice.len() > 255 {
            panic!("Length is over 255: {}", slice);
        }
        slice.len() as u8
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BookNgrams<'a>(#[serde(borrow)] Vec<BookNgram<'a>>);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NgramEntry<'a> {
    book: &'a str,
    ngram: NgramData,
}

#[derive(Debug, Serialize)]
pub struct Output<'a> {
    pub books: HashSet<&'a str>,
    pub string: String,
}

impl<'a> fmt::Display for Output<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}\n\nBooks:\n", self.string)?;

        for (index, book) in self.books.iter().enumerate() {
            write!(f, "\t{}: {}\n", index + 1, book)?;
        }

        Ok(())
    }
}

impl<'a> Output<'a> {
    pub fn new() -> Output<'a> {
        Output {
            books: HashSet::new(),
            string: String::new(),
        }
    }

    pub fn append_entry(&mut self, book: &'a str, word: &'a str) {
        self.add_book(book);

        if self.string.len() > 0 {
            self.string.push(' ');
        }

        self.string.push_str(word);
    }

    fn add_book(&mut self, book: &'a str) {
        self.books.insert(book);
    }
}

impl<'a> FromIterator<BookNgram<'a>> for BookNgrams<'a> {
    fn from_iter<I: IntoIterator<Item=BookNgram<'a>>>(iter: I) -> Self {
        let mut c = BookNgrams::new();

        for i in iter {
            c.add(i);
        }

        c
    }
}

impl<'a> BookNgrams<'a> {
    pub fn new() -> BookNgrams<'a> {
        BookNgrams(Vec::new())
    }

    pub fn from_books(books: &'a HashMap<String, String>) -> Self {
        let ngrams = books
            .par_iter()
            .map(|(ref book, ref content)| 
                BookNgram::new(&content, book))
            .collect::<Vec<BookNgram>>();
        
        BookNgrams(ngrams)
    }

    pub fn generate(&self) -> Output<'a> {
        let mut output = Output::new();
        let mut current = self.random(WORD_SEP, WORD_SEP, WORD_SEP);

        loop {
            if let Some(choice) = current {
                let data = self.0
                    .par_iter()
                    .find_any(|ref b| b.book == choice.book)
                    .unwrap();

                if choice.ngram.current(data.content) == WORD_SEP {
                    break;
                }

                output.append_entry(choice.book, choice.ngram.current(data.content));

                current = self.random(
                    choice.ngram.p_prev(data.content),
                    choice.ngram.prev(data.content),
                    choice.ngram.current(data.content));
            } else {
                break;
            }
        }
        output
    }

    fn random(&self, pp_prev: &str, p_prev: &str, prev: &str) -> Option<NgramEntry<'a>> {
        let mut rng = rand::thread_rng();
        let choices = self.search(pp_prev, p_prev, prev);
        
        match rng.choose(&choices) {
            Some(choice) => Some(choice.clone()),
            None => None,
        }
    }

    fn search(&self, pp_prev: &str, p_prev: &str, prev: &str) -> Vec<NgramEntry<'a>> {
        self.0.par_iter().flat_map(|bg| {
            bg.search(pp_prev, p_prev, prev)
        }).collect()
    }

    fn add(&mut self, item: BookNgram<'a>) {
        self.0.push(item);
    }
}