use std::collections::HashSet;

#[derive(Debug, Deserialize, Serialize)]
pub struct NgramData<'a> {
    p_prev: &'a str,
    prev: &'a str,
    current: &'a str,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BookNgram<'a> {
    book: String,
    data: Vec<NgramData<'a>>,
    dict: HashSet<String>,
}

const WORD_SEP: String = String::from("");

impl <'a> BookNgram<'a> {
    pub fn new(dict: HashSet<String>, map: Vec<Vec<&'a String>>, book: String) -> BookNgram<'a> {
        let mut data = Vec::new();

        for line in map {
            if line.len() < 4 {
                continue;
            }

            let mut ngs = generate_ngrams(&line, 3);

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
            dict: dict,
            data: data,
        }
    }
}

fn generate_ngrams<'a>(line: &Vec<&'a String>, count: usize) -> Vec<Vec<&'a String>> {
    let mut windows = line
        .windows(count)
        .map(|s| s.to_vec())
        .collect::<Vec<Vec<&String>>>();

    let len = line.len();
    windows.push(vec![line[len - 1], &WORD_SEP, &WORD_SEP]);
    windows.push(vec![line[len - 2], line[len - 1], &WORD_SEP]);

    windows.push(vec![&WORD_SEP, &WORD_SEP, line[0]]);
    windows.push(vec![&WORD_SEP, line[0], line[1]]);

    windows
}