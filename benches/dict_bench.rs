use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use fuzzies::{Dictionary, SearchResult};
use std::fs;
use std::hint::black_box;
use std::io::Write;
use std::time::Duration;
use tempfile::NamedTempFile;

fn generate_words() -> Vec<String> {
    let mut words = vec![
        "apple".to_string(),
        "banana".to_string(),
        "cherry".to_string(),
        "date".to_string(),
        "fig".to_string(),
        "grape".to_string(),
    ];
    for i in 0..1000 {
        words.push(format!("word{:04}", i));
    }
    words
}

fn setup_bench_dictionary() -> (Dictionary, NamedTempFile) {
    use fst::SetBuilder;

    let mut temp_file = NamedTempFile::new().unwrap();
    let mut words = generate_words();
    words.sort_unstable();

    let mut build = SetBuilder::new(&mut temp_file).unwrap();
    for word in words {
        build.insert(word).unwrap();
    }
    build.finish().unwrap();

    let dict = Dictionary::open(temp_file.path()).unwrap();
    (dict, temp_file)
}

fn bench_build_and_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("1. Prep, Build, and Load");
    let unsorted_words = generate_words(); // Purposely unsorted

    group.bench_function("Dictionary::sort (in-place)", |b| {
        b.iter_batched(
            || {
                let mut raw_file = NamedTempFile::new().unwrap();
                for word in &unsorted_words {
                    writeln!(raw_file, "{}", word).unwrap();
                }
                raw_file
            },
            |raw_file| {
                Dictionary::sort(raw_file.path()).unwrap();
            },
            BatchSize::SmallInput,
        );
    });

    let sorted_file = {
        let mut f = NamedTempFile::new().unwrap();
        let mut w = unsorted_words.clone();
        w.sort_unstable();
        for word in w {
            writeln!(f, "{}", word).unwrap();
        }
        f
    };

    group.bench_function("Dictionary::build", |b| {
        b.iter_batched(
            || NamedTempFile::new().unwrap(),
            |output_file| {
                Dictionary::build(sorted_file.path(), output_file.path()).unwrap();
            },
            BatchSize::SmallInput,
        );
    });

    let fst_file = NamedTempFile::new().unwrap();
    Dictionary::build(sorted_file.path(), fst_file.path()).unwrap();

    group.bench_function("Dictionary::open (Mmap)", |b| {
        b.iter(|| {
            let _dict = black_box(Dictionary::open(fst_file.path()).unwrap());
        });
    });

    let fst_bytes: &'static [u8] = Box::leak(fs::read(fst_file.path()).unwrap().into_boxed_slice());
    group.bench_function("Dictionary::from_embedded", |b| {
        b.iter(|| {
            let _dict = black_box(Dictionary::from_embedded(black_box(fst_bytes)).unwrap());
        });
    });

    group.finish();
}

fn bench_metadata(c: &mut Criterion) {
    let (dict, _temp) = setup_bench_dictionary();
    let mut group = c.benchmark_group("2. Metadata & Membership");

    group.bench_function("len", |b| b.iter(|| black_box(dict.len())));
    group.bench_function("is_empty", |b| b.iter(|| black_box(dict.is_empty())));

    group.bench_function("contains (Hit)", |b| {
        b.iter(|| black_box(dict.contains(black_box("apple"))))
    });
    group.bench_function("contains (Miss)", |b| {
        b.iter(|| black_box(dict.contains(black_box("pineapple"))))
    });

    group.finish();
}

fn bench_searches(c: &mut Criterion) {
    let (dict, _temp) = setup_bench_dictionary();
    let mut group = c.benchmark_group("3. Search Queries");

    group.bench_function("Exact (distance=0)", |b| {
        b.iter(|| {
            let _res: Vec<SearchResult> = black_box(
                dict.search(black_box("apple"))
                    .distance(0)
                    .execute()
                    .unwrap(),
            );
        });
    });

    group.bench_function("Fuzzy + Transposition (distance=1)", |b| {
        b.iter(|| {
            let _res: Vec<SearchResult> = black_box(
                dict.search(black_box("aple"))
                    .distance(1)
                    .transposition(true)
                    .execute()
                    .unwrap(),
            );
        });
    });

    group.bench_function("Prefix Search", |b| {
        b.iter(|| {
            let _res: Vec<SearchResult> = black_box(
                dict.search(black_box("app"))
                    .prefix(true)
                    .limit(5)
                    .execute()
                    .unwrap(),
            );
        });
    });

    group.bench_function("Range Bounded Search (ge='b', le='c')", |b| {
        b.iter(|| {
            let _res: Vec<SearchResult> = black_box(
                dict.search(black_box("b"))
                    .prefix(true)
                    .ge(black_box("b"))
                    .le(black_box("c"))
                    .limit(5)
                    .execute()
                    .unwrap(),
            );
        });
    });

    group.finish();
}

fn bench_batch(c: &mut Criterion) {
    let (dict, _temp) = setup_bench_dictionary();
    let mut group = c.benchmark_group("4. Batch Search (Rayon)");

    let batch_sizes = vec![100, 500, 1000];

    for size in batch_sizes {
        let batch_queries: Vec<&str> = (0..size)
            .map(|i| if i % 2 == 0 { "word0050" } else { "baxana" })
            .collect();

        group.bench_with_input(
            BenchmarkId::new("Parallel Execution", size),
            &batch_queries,
            |b, queries| {
                b.iter(|| {
                    let _res = black_box(dict.batch_search(black_box(queries)).execute());
                });
            },
        );
    }
    group.finish();
}

fn configured_criterion() -> Criterion {
    Criterion::default().measurement_time(Duration::from_secs(5))
}

criterion_group!(
    name = benches;
    config = configured_criterion();
    targets = bench_build_and_load, bench_metadata, bench_searches, bench_batch
);
criterion_main!(benches);
