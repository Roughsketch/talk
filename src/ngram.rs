use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use ngrams::Ngrams;

#[derive(Debug, Deserialize, Serialize)]
pub struct NgramData<'a> {
    p_prev: &'a str,
    prev: &'a str,
    current: &'a str,
    total: u32,
}

impl<'a> NgramData<'a> {
    fn total(&self) -> u32 {
        self.total
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BookNgram<'a> {
    book: &'a str,
    //  TODO: Add counter to get rid of duplicates and double as RNG weight.
    data: HashSet<NgramData<'a>>,
    content: &'a str,
}

impl<'a> Hash for NgramData<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.p_prev.hash(state);
        self.prev.hash(state);
        self.current.hash(state);
    }
}

impl<'a> PartialEq for NgramData<'a> {
  fn eq(&self, other: &NgramData<'a>) -> bool {
        (self.p_prev == other.p_prev) &&
        (self.prev == other.prev) &&
        (self.current == other.current)
  }
}

impl<'a> Eq for NgramData<'a> {}

impl <'a> BookNgram<'a> {
    pub fn new(content: &'a str, book: &'a str) -> BookNgram<'a> {
        let mut data = HashSet::new();

        let lines = content
            .split('\n')
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>();

        for line in lines {
            let iter = line.split_whitespace();
            let ngs = Ngrams::new(iter, 3)
                .pad()
                .collect::<Vec<Vec<&str>>>();

            for ng in ngs {
                if ng[1] != "\u{2060}" {
                    let mut entry = NgramData {
                        current: ng[2],
                        prev: ng[1],
                        p_prev: ng[0],
                        total: 1,
                    };
                    
                    {
                        let opt = data.get(&entry);

                        if opt.is_some() {
                            let en: &NgramData<'a> = opt.unwrap();
                            entry.total += en.total();
                        }
                    }

                    data.replace(entry);
                }
            }
        }

        BookNgram {
            book: book,
            content: content,
            data: data,
        }
    }
}

#[derive(Debug)]
pub struct BookNgrams<'a>(Vec<BookNgram<'a>>);

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

    pub fn start(self) -> String {
        "test".into()
    }

    fn add(&mut self, item: BookNgram<'a>) {
        self.0.push(item);
    }
}