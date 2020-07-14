use criterion::{black_box, BatchSize, BenchmarkId, Criterion, Throughput};
use generic_array::typenum::{U12, U16, U4, U6, U8};
use rand::RngCore;
use std::collections::BTreeMap;

use crate::util::Item;

pub fn benches(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("Get");

    let mut rng = rand::thread_rng();

    for i in &[2u32, 4, 6, 8, 10, 25, 50, 75, 100] {
        let size = *i as u64;

        group.throughput(Throughput::Elements(size as u64));

        group.bench_function(
            BenchmarkId::new("LinkedSlabMap<u64, [u64; 3], U4>", size),
            |b| {
                b.iter_batched_ref(
                    || {
                        let keys = (0..size).map(|_| rng.next_u64()).collect::<Vec<_>>();
                        let mut map = linkedslab::SlabMap::<_, _, U4>::new();
                        for _ in 0..size {
                            map.insert(rng.next_u64(), Item::new(&mut rng));
                        }
                        black_box((keys, map))
                    },
                    |(keys, map)| {
                        for k in keys.iter() {
                            map.get(k);
                        }
                        black_box(map);
                    },
                    BatchSize::SmallInput,
                )
            },
        );

        group.bench_function(
            BenchmarkId::new("LinkedSlabMap<u64, [u64; 3], U8>", size),
            |b| {
                b.iter_batched_ref(
                    || {
                        let keys = (0..size).map(|_| rng.next_u64()).collect::<Vec<_>>();
                        let mut map = linkedslab::SlabMap::<_, _, U8>::new();
                        for _ in 0..size {
                            map.insert(rng.next_u64(), Item::new(&mut rng));
                        }
                        black_box((keys, map))
                    },
                    |(keys, map)| {
                        for k in keys.iter() {
                            map.get(k);
                        }
                        black_box(map);
                    },
                    BatchSize::SmallInput,
                )
            },
        );

        group.bench_function(
            BenchmarkId::new("LinkedSlabMap<u64, [u64; 3], U12>", size),
            |b| {
                b.iter_batched_ref(
                    || {
                        let keys = (0..size).map(|_| rng.next_u64()).collect::<Vec<_>>();
                        let mut map = linkedslab::SlabMap::<_, _, U12>::new();
                        for _ in 0..size {
                            map.insert(rng.next_u64(), Item::new(&mut rng));
                        }
                        black_box((keys, map))
                    },
                    |(keys, map)| {
                        for k in keys.iter() {
                            map.get(k);
                        }
                        black_box(map);
                    },
                    BatchSize::SmallInput,
                )
            },
        );

        group.bench_function(
            BenchmarkId::new("LinkedSlabMap<u64, [u64; 3], U16>", size),
            |b| {
                b.iter_batched_ref(
                    || {
                        let keys = (0..size).map(|_| rng.next_u64()).collect::<Vec<_>>();
                        let mut map = linkedslab::SlabMap::<_, _, U16>::new();
                        for _ in 0..size {
                            map.insert(rng.next_u64(), Item::new(&mut rng));
                        }
                        black_box((keys, map))
                    },
                    |(keys, map)| {
                        for k in keys.iter() {
                            map.get(k);
                        }
                        black_box(map);
                    },
                    BatchSize::SmallInput,
                )
            },
        );

        // group.bench_function(BenchmarkId::new("BTreeMap<u64, [u64; 3]>", size), |b| {
        //     b.iter_batched_ref(
        //         || {
        //             let keys = (0..size).map(|_| rng.next_u64()).collect::<Vec<_>>();
        //             let mut map = BTreeMap::default();
        //             for _ in 0..size {
        //                 map.insert(rng.next_u64(), Item::new(&mut rng));
        //             }
        //             black_box((keys, map))
        //         },
        //         |(keys, map)| {
        //             for k in keys.iter() {
        //                 map.get(k);
        //             }
        //             black_box(map);
        //         },
        //         BatchSize::SmallInput,
        //     )
        // });

        // group.bench_function(BenchmarkId::new("Vec<(u64, [u64; 3])>", size), |b| {
        //     b.iter_batched_ref(
        //         || {
        //             let keys = (0..size).map(|_| rng.next_u64()).collect::<Vec<_>>();
        //             let map = (0..size).map(|_| (rng.next_u64(), Item::new(&mut rng))).collect::<Vec<_>>();
        //             black_box((keys, map))
        //         },
        //         |(keys, map)| {
        //             for k in keys.iter() {
        //                 let _wat = map.iter().find(|x| x.0 == *k);
        //             }
        //             black_box(map);
        //         },
        //         BatchSize::SmallInput,
        //     )
        // });
    }

    group.finish();
}
