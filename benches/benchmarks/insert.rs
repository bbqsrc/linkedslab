use criterion::{black_box, BatchSize, BenchmarkId, Criterion, Throughput};
use rand::RngCore;
use std::collections::BTreeMap;

use crate::util::Item;

pub fn benches(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("Insert");

    let mut rng = rand::thread_rng();

    for i in 1u32..=9u32 {
        let size = 2u64.pow(i as u32);

        group.throughput(Throughput::Elements(size as u64));

        group.bench_function(
            BenchmarkId::new("LinkedSlabMap<u64, [u64; 3]>", size),
            |b| {
                b.iter_batched_ref(
                    || {
                        let keys = (0..size).map(|_| rng.next_u64()).collect::<Vec<_>>();
                        let values = (0..size).map(|_| Item::new(&mut rng)).collect::<Vec<_>>();
                        black_box((keys, values, linkedslab::SlabMap::default()))
                    },
                    |(keys, values, map)| {
                        for (k, v) in keys.iter().zip(values.iter()) {
                            map.insert(*k, *v);
                        }
                        black_box(map);
                    },
                    BatchSize::SmallInput,
                )
            },
        );

        group.bench_function(BenchmarkId::new("BTreeMap<u64, [u64; 3]>", size), |b| {
            b.iter_batched_ref(
                || {
                    let keys = (0..size).map(|_| rng.next_u64()).collect::<Vec<_>>();
                    let values = (0..size).map(|_| Item::new(&mut rng)).collect::<Vec<_>>();
                    black_box((keys, values, BTreeMap::default()))
                },
                |(keys, values, map)| {
                    for (k, v) in keys.iter().zip(values.iter()) {
                        map.insert(*k, *v);
                    }
                    black_box(map);
                },
                BatchSize::SmallInput,
            )
        });

        group.bench_function(BenchmarkId::new("Vec<(u64, [u64; 3])>", size), |b| {
            b.iter_batched_ref(
                || {
                    let keys = (0..size).map(|_| rng.next_u64()).collect::<Vec<_>>();
                    let values = (0..size).map(|_| Item::new(&mut rng)).collect::<Vec<_>>();
                    black_box((keys, values, Vec::default()))
                },
                |(keys, values, map)| {
                    for (k, v) in keys.iter().zip(values.iter()) {
                        map.push((*k, *v))
                    }
                    black_box(map);
                },
                BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}
