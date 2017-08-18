use std::collections::HashSet;

#[derive(Debug)]
pub struct BookNgram<'a> {
    ngrams: Vec<Vec<Vec<&'a String>>>,
    dict: &'a HashSet<String>,
}

impl <'a> BookNgram<'a> {
    pub fn new(dict: &'a HashSet<String>, content: &String) -> BookNgram<'a> {
        let mut data = Vec::new();
        let map = map_tokens(&dict, content);

        for line in map {
            if line.len() < 4 {
                continue;
            }

            let mut windows = line.windows(3);
            let mut ngs: Vec<Vec<&String>> = Vec::new();

            for window in windows {
                ngs.push(window.to_vec());
            }

            let len = line.len();
            let padding = dict.get("").unwrap();

            ngs.push(vec![line[len - 1], padding, padding]);
            ngs.push(vec![line[len - 2], line[len - 1], padding]);

            ngs.push(vec![padding, padding, line[0]]);
            ngs.push(vec![padding, line[0], line[1]]);

            data.push(ngs);
        }

        BookNgram {
            dict: dict,
            ngrams: data,
        }
    }
}

fn map_tokens<'a>(dict: &'a HashSet<String>, content: &String) -> Vec<Vec<&'a String>> {
    content
        .split("\n")
        .filter(|line| !line.is_empty())
        .map(|line| {
            line.split_whitespace()
                .map(|word| {
                    dict.get(word).unwrap()
                })
                .collect()
        })
        .collect()
}