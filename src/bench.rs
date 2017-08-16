use test;

use *;

#[bench]
fn bench_generate(b: &mut test::Bencher) {
    let data = match read_data("word_data.json") {
        Ok(data) => data,
        Err(why) => {
            println!("Could not read data: {:?}", why);
            return
        }
    };

    b.iter(|| {
        generate(&data);
    })
}