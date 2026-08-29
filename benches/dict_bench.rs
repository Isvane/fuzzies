use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

use std::hint::black_box;
use std::io::Write;
use tempfile::NamedTempFile;

use fuzzies::Dictionary;

fn generate_dictionary_keys(count: usize) -> Vec<String> {
    let mut keys = Vec::with_capacity(count);
    for i in 0..count {
        let word = format!("word_{:08x}_{:x}", i, i * 37);
        keys.push(word);
    }
    keys.sort();
    keys.dedup();
    keys
}

fn generate_query_batch(keys: &[String], count: usize, edit_type: EditType) -> Vec<String> {
    let mut queries = Vec::with_capacity(count);
    for (i, key) in keys.iter().take(count).enumerate() {
        let mut q = key.clone();
        match edit_type {
            EditType::Exact => {}
            EditType::SingleDeletion => {
                if q.len() > 1 {
                    q.pop();
                }
            }
            EditType::SingleSubstitution => {
                if !q.is_empty() {
                    let last_idx = q.len() - 1;
                    q.replace_range(last_idx.., "z");
                }
            }
            EditType::Transposition => {
                if q.len() >= 2 {
                    let mut bytes = q.into_bytes();
                    bytes.swap(0, 1);
                    q = String::from_utf8(bytes).unwrap();
                }
            }
            EditType::Miss => {
                q = format!("nonexistent_prefix_{:x}", i);
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
    let mut group = c.benchmark_group("Dictionary Construction");

    for &size in &[1_000, 10_000, 50_000] {
        let keys = generate_dictionary_keys(size);
        group.throughput(Throughput::Elements(keys.len() as u64));

        group.bench_with_input(BenchmarkId::new("from_iterator", size), &keys, |b, keys| {
            b.iter_batched(
                || keys.clone(),
                |k| black_box(Dictionary::from_iterator(black_box(k)).unwrap()),
                BatchSize::SmallInput,
            );
        });
    }

    let keys = generate_dictionary_keys(10_000);
    group.throughput(Throughput::Elements(keys.len() as u64));
    group.bench_function("build_from_file", |b| {
        b.iter_batched(
            || {
                let input_file = NamedTempFile::new().unwrap();
                let output_file = NamedTempFile::new().unwrap();
                {
                    let mut writer = std::io::BufWriter::new(&input_file);
                    for key in &keys {
                        writeln!(writer, "{}", key).unwrap();
                    }
                }
                (input_file, output_file)
            },
            |(input, output)| {
                black_box(
                    Dictionary::build(black_box(input.path()), black_box(output.path())).unwrap(),
                )
            },
            BatchSize::PerIteration,
        );
    });

    group.finish();
}

fn bench_exact_contains(c: &mut Criterion) {
    let mut group = c.benchmark_group("Exact Lookup (contains)");
    let keys = generate_dictionary_keys(50_000);
    let dict = Dictionary::from_iterator(keys.clone()).unwrap();

    let hit_key = keys[keys.len() / 2].clone();
    let miss_key = "nonexistent_key_xyz_99999".to_string();

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
    let keys = generate_dictionary_keys(50_000);
    let dict = Dictionary::from_iterator(keys.clone()).unwrap();

    let target_query = &keys[keys.len() / 2];

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

    for prefix in [false, true] {
        let prefix_query = &target_query[..target_query.len() / 2];
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
    let keys = generate_dictionary_keys(50_000);
    let dict = Dictionary::from_iterator(keys.clone()).unwrap();

    let lower_bound = &keys[10_000];
    let upper_bound = &keys[15_000];
    let query = &keys[12_000];

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
    let keys = generate_dictionary_keys(50_000);
    let dict = Dictionary::from_iterator(keys.clone()).unwrap();

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
