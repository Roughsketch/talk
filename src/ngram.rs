use std::iter::FromIterator;
use ngrams::Ngrams;

#[derive(Debug, Deserialize, Serialize)]
pub struct NgramData {
    p_prev: u16,
    p_prev_len: u8,
    prev: u16,
    prev_len: u8,
    current: u16,
    current_len: u8,
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
                        current: start.offset_to(ng[2].as_ptr()).unwrap_or(0) as u16,
                        current_len: ng[2].len() as u8,
                        prev: start.offset_to(ng[1].as_ptr()).unwrap_or(0) as u16,
                        prev_len: ng[1].len() as u8,
                        p_prev: start.offset_to(ng[0].as_ptr()).unwrap_or(0) as u16,
                        p_prev_len: ng[0].len() as u8,
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