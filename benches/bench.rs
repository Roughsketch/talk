#[allow(unused_imports)]
use test;
#[allow(unused_imports)]
use std::path::Path;
#[allow(unused_imports)]
use rayon::prelude::*;
#[allow(unused_imports)]
use ngram;
#[allow(unused_imports)]
use read_books;

#[bench]
fn bench_read(b: &mut test::Bencher) {
    let path = Path::new("data/sentences");

    b.iter(|| {
        read_books(path);
    })
}

#[bench]
fn bench_generate(b: &mut test::Bencher) {
    let book_data = read_books(Path::new("data/sentences"));
    let books = book_data
        .iter()
        .map(|(ref book, ref content)| 
            ngram::BookNgram::new(&content, book))
        .collect::<ngram::BookNgrams>();

    b.iter(|| {
        loop {
            let res = books.generate();
            if res.books.len() >= 4 {
                break res;
            }
        }
    })
}

#[bench]
fn bench_build(b: &mut test::Bencher) {
    b.iter(|| {
        let book_data = read_books(Path::new("data/sentences"));
        ngram::BookNgrams::from_books(&book_data);
    })
}