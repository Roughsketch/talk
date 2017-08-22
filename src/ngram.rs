use std::collections::HashSet;
use std::iter::FromIterator;
use ngrams::Ngrams;
use rand::{self, Rng};

const WORD_SEP: &'static str = "\u{2060}";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NgramData {
    p_prev: u32,
    p_prev_len: u8,
    prev: u32,
    prev_len: u8,
    current: u32,
    current_len: u8,
}

impl NgramData {
    pub fn p_prev<'a>(&self, content: &'a str) -> &'a str {
        if self.p_prev_len == 0 {
            return WORD_SEP
        }

        let index = self.p_prev as usize;
        let end = index + self.p_prev_len as usize;
        &content[index..end]
    }

    pub fn prev<'a>(&self, content: &'a str) -> &'a str {
        if self.prev_len == 0 {
            return WORD_SEP
        }
        
        let index = self.prev as usize;
        let end = index + self.prev_len as usize;
        &content[index..end]
    }

    pub fn current<'a>(&self, content: &'a str) -> &'a str {
        if self.current_len == 0 {
            return WORD_SEP
        }
        
        let index = self.current as usize;
        let end = index + self.current_len as usize;
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
            let ngs = Ngrams::new(line.split_whitespace(), 3)
                .pad()
                .collect::<Vec<Vec<&str>>>();

            for ng in ngs {
                if !(ng[1] == "\u{2060}" && ng[2] == "\u{2060}") {
                    data.push(NgramData {
                        current: start.offset_to(ng[2].as_ptr()).unwrap_or(0) as u32,
                        current_len: word_length(ng[2]),
                        prev: start.offset_to(ng[1].as_ptr()).unwrap_or(0) as u32,
                        prev_len: word_length(ng[1]),
                        p_prev: start.offset_to(ng[0].as_ptr()).unwrap_or(0) as u32,
                        p_prev_len: word_length(ng[0]),
                    });
                }
            }
        }

        BookNgram {
            book: book,
            content: content,
            data: data,
        }
    }

    fn search(&self, p_prev: &str, prev: &str) -> Vec<NgramEntry<'a>> {
        let mut res = Vec::new();

        for entry in &self.data {
            if entry.p_prev(self.content) == p_prev && entry.prev(self.content) == prev {
                res.push(NgramEntry {
                    book: self.book,
                    ngram: entry.clone(),
                });
            }
        }

        res
    }
}

fn word_length<'a>(slice: &'a str) -> u8 {
    if slice == "\u{2060}" {
        0
    } else {
        if slice.len() > 255 {
            panic!("Length is over 255: {}", slice);
        }
        slice.len() as u8
    }
}

#[derive(Debug)]
pub struct BookNgrams<'a>(Vec<BookNgram<'a>>);

#[derive(Clone, Debug)]
pub struct NgramEntry<'a> {
    book: &'a str,
    ngram: NgramData,
}

#[derive(Debug)]
pub struct Output<'a> {
    books: HashSet<&'a str>,
    string: String,
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
            self.string += " ".into();
        }

        self.string += word.into();
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

    pub fn generate(&self) -> Output<'a> {
        let mut output = Output::new();
        let mut current = self.random("\u{2060}", "\u{2060}");

        loop {
            if let Some(choice) = current {
                let book_index = self.0.iter().position(|ref b| b.book == choice.book).unwrap();
                let content = self.0[book_index].content;

                if choice.ngram.current(content) == WORD_SEP {
                    break;
                }

                output.append_entry(choice.book, choice.ngram.current(content));

                current = self.random(choice.ngram.prev(content), choice.ngram.current(content));
            } else {
                break;
            }
        }
        output
    }

    fn random(&self, p_prev: &str, prev: &str) -> Option<NgramEntry<'a>> {
        let mut rng = rand::thread_rng();
        let choices = self.search(p_prev, prev);
        
        match rng.choose(&choices) {
            Some(choice) => Some(choice.clone()),
            None => None,
        }
    }

    fn search(&self, p_prev: &str, prev: &str) -> Vec<NgramEntry<'a>> {
        let mut res = Vec::new();

        for bg in &self.0 {
            res.append(&mut bg.search(p_prev, prev));
        }

        res
    }

    fn add(&mut self, item: BookNgram<'a>) {
        self.0.push(item);
    }
}