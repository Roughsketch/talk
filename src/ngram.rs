use ngrams::Ngrams;

#[derive(Debug, Deserialize, Serialize)]
pub struct NgramData<'a> {
    p_prev: &'a str,
    prev: &'a str,
    current: &'a str,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BookNgram<'a> {
    book: &'a str,
    data: Vec<NgramData<'a>>,
    content: &'a str,
}

impl <'a> BookNgram<'a> {
    pub fn new(content: &'a str, book: &'a str) -> BookNgram<'a> {
        let mut data = Vec::new();

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
