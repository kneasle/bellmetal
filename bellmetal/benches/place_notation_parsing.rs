use bellmetal::*;
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_pn_parser(c: &mut Criterion) {
    let line_vec = include_str!("../../CC_library.txt")
        .lines()
        .map(|l| {
            let mut iter = l.split("|");

            // First item is the stage
            let stage = Stage::from(iter.next().unwrap().parse::<usize>().unwrap());
            // Next item is the name, which we can ignore
            iter.next();
            // Last part is the place notation
            let pns = iter.next().unwrap();

            assert!(iter.next().is_none());

            (stage, pns)
        })
        .collect::<Vec<(Stage, &str)>>();

    let mut lines = line_vec.iter().cycle();

    c.bench_function("Parse place notations", |b| {
        let (stage, string) = lines.next().unwrap();

        b.iter(|| PlaceNotation::from_multiple_string(string, *stage))
    });
}

criterion_group!(benches, bench_pn_parser);
criterion_main!(benches);
