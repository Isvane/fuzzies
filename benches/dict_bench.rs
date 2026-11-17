use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use fst::{IntoStreamer, Set, Streamer};
use std::collections::HashSet;
use std::hint::black_box;

use fuzzies::Dictionary;

const FST_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/words.fst");

fn load_dictionary() -> Dictionary {
    Dictionary::open(FST_PATH).expect("failed to load assets/words.fst")
}

fn load_sample_keys(limit: usize) -> Vec<String> {
    let bytes = std::fs::read(FST_PATH).expect("failed to read assets/words.fst");
    let set = Set::new(bytes).expect("invalid FST file");
    let mut stream = set.into_stream();
    let mut keys = Vec::with_capacity(limit);

    while let Some(key) = stream.next() {
        if keys.len() >= limit {
            break;
        }
        if let Ok(s) = std::str::from_utf8(key) {
            keys.push(s.to_owned());
        }
    }

    assert!(
        !keys.is_empty(),
        "assets/words.fst contains no valid UTF-8 keys"
    );
    keys
}

fn make_english_miss(word: &str, is_known: impl Fn(&str) -> bool) -> String {
    let char_count = word.chars().count();
    let prefix: String = if char_count <= 2 {
        word.to_owned()
    } else {
        word.chars().take(char_count - 2).collect()
    };

    for suffix in ["qx", "zx", "jz", "vq"] {
        let candidate = format!("{prefix}{suffix}");
        if !is_known(&candidate) {
            return candidate;
        }
    }

    let mut fallback = format!("{prefix}qx");
    while is_known(&fallback) {
        fallback.push('z');
    }
    fallback
}

fn generate_query_batch(keys: &[String], count: usize, edit_type: EditType) -> Vec<String> {
    let known: HashSet<&str> = keys.iter().map(String::as_str).collect();
    let mut queries = Vec::with_capacity(count);

    for key in keys.iter().cycle().take(count) {
        let mut q = key.clone();
        match edit_type {
            EditType::Exact => {}
            EditType::SingleDeletion => {
                if q.chars().count() > 1 {
                    q.pop();
                }
            }
            EditType::SingleSubstitution => {
                if let Some((last_idx, _)) = q.char_indices().next_back() {
                    q.replace_range(last_idx.., "z");
                }
            }
            EditType::Transposition => {
                if q.len() >= 2 && q.is_ascii() {
                    let mut bytes = q.into_bytes();
                    bytes.swap(0, 1);
                    q = String::from_utf8(bytes)
                        .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned());
                }
            }
            EditType::Miss => {
                q = make_english_miss(key, |candidate| known.contains(candidate));
            }
        }
        queries.push(q);
    }
    queries
}

#[derive(Clone, Copy)]
enum EditType {
    Exact,
    SingleDeletion,
    SingleSubstitution,
    Transposition,
    Miss,
}

fn bench_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("Dictionary Load & Construction");

    group.bench_function("load_from_fst", |b| {
        b.iter(|| black_box(Dictionary::open(black_box(FST_PATH)).unwrap()));
    });

    let keys = load_sample_keys(50_000);
    group.throughput(Throughput::Elements(keys.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("from_iterator", keys.len()),
        &keys,
        |b, keys| {
            b.iter_batched(
                || keys.clone(),
                |k| black_box(Dictionary::from_iterator(black_box(k)).unwrap()),
                BatchSize::SmallInput,
            );
        },
    );

    group.finish();
}

fn bench_exact_contains(c: &mut Criterion) {
    let mut group = c.benchmark_group("Exact Lookup (contains)");
    let dict = load_dictionary();
    let keys = load_sample_keys(50_000);

    let hit_key = ["algorithm", "example", "benchmark", "dictionary", "hit"]
        .into_iter()
        .find(|&w| dict.contains(w))
        .map(str::to_owned)
        .unwrap_or_else(|| keys[keys.len() / 2].clone());

    let miss_key = make_english_miss(&hit_key, |candidate| dict.contains(candidate));

    group.throughput(Throughput::Elements(1));

    group.bench_function("hit", |b| {
        b.iter(|| black_box(dict.contains(black_box(&hit_key))));
    });

    group.bench_function("miss", |b| {
        b.iter(|| black_box(dict.contains(black_box(&miss_key))));
    });

    group.finish();
}

fn bench_single_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("Single Search");
    let dict = load_dictionary();
    let keys = load_sample_keys(50_000);

    let target_query = keys
        .iter()
        .skip(keys.len() / 2)
        .find(|k| k.chars().count() >= 6)
        .unwrap_or(&keys[keys.len() / 2]);

    for distance in [1, 2] {
        group.bench_with_input(
            BenchmarkId::new("distance_scaling", distance),
            &distance,
            |b, &dist| {
                b.iter(|| {
                    black_box(
                        dict.search(black_box(target_query))
                            .distance(dist)
                            .execute()
                            .unwrap(),
                    )
                });
            },
        );
    }

    for transposition in [false, true] {
        group.bench_with_input(
            BenchmarkId::new("transposition", transposition),
            &transposition,
            |b, &trans| {
                b.iter(|| {
                    black_box(
                        dict.search(black_box(target_query))
                            .distance(1)
                            .transposition(trans)
                            .execute()
                            .unwrap(),
                    )
                });
            },
        );
    }

    let half_chars = target_query.chars().count() / 2;
    let prefix_end = target_query
        .char_indices()
        .nth(half_chars)
        .map_or(target_query.len(), |(idx, _)| idx);
    let prefix_query = &target_query[..prefix_end];

    for prefix in [false, true] {
        group.bench_with_input(
            BenchmarkId::new("prefix_matching", prefix),
            &prefix,
            |b, &pref| {
                b.iter(|| {
                    black_box(
                        dict.search(black_box(prefix_query))
                            .distance(1)
                            .prefix(pref)
                            .execute()
                            .unwrap(),
                    )
                });
            },
        );
    }

    for limit in [1, 5, 50] {
        group.bench_with_input(BenchmarkId::new("limit_size", limit), &limit, |b, &lim| {
            b.iter(|| {
                black_box(
                    dict.search(black_box(target_query))
                        .limit(lim)
                        .distance(1)
                        .execute()
                        .unwrap(),
                )
            });
        });
    }

    group.finish();
}

fn bench_range_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("Range Bounded Search");
    let dict = load_dictionary();
    let keys = load_sample_keys(50_000);

    let len = keys.len();
    let lower_bound = &keys[len / 5];
    let query = &keys[len / 4];
    let upper_bound = &keys[(len * 3) / 10];

    group.bench_function("ge_and_le_bounds", |b| {
        b.iter(|| {
            black_box(
                dict.search(black_box(query))
                    .ge(black_box(lower_bound))
                    .le(black_box(upper_bound))
                    .distance(1)
                    .execute()
                    .unwrap(),
            )
        });
    });

    group.finish();
}

fn bench_batch_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("Rayon Parallel Batch Search");
    let dict = load_dictionary();
    let keys = load_sample_keys(50_000);

    let edit_types = [
        ("exact", EditType::Exact),
        ("deletion", EditType::SingleDeletion),
        ("substitution", EditType::SingleSubstitution),
        ("transposition", EditType::Transposition),
        ("miss", EditType::Miss),
    ];

    for (edit_name, edit_type) in edit_types {
        for &batch_size in &[10, 100, 1_000] {
            let raw_queries = generate_query_batch(&keys, batch_size, edit_type);
            let query_refs: Vec<&str> = raw_queries.iter().map(|s| s.as_str()).collect();

            group.throughput(Throughput::Elements(batch_size as u64));

            group.bench_with_input(
                BenchmarkId::new(format!("{edit_name}/batch"), batch_size),
                &query_refs,
                |b, queries| {
                    b.iter(|| {
                        black_box(
                            dict.batch_search(black_box(queries))
                                .distance(1)
                                .limit(5)
                                .execute(),
                        )
                    });
                },
            );
        }
    }

    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .with_plots();
    targets =
        bench_construction,
        bench_exact_contains,
        bench_single_search,
        bench_range_search,
        bench_batch_search
);

criterion_main!(benches);
