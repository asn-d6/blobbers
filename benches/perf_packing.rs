#![feature(int_roundings)]

use std::time::Duration;
use criterion::*;

#[path = "../src/packer_naive.rs"]
mod packer_naive;
#[path = "../src/packer_tight.rs"]
mod packer_tight;

fn benchmark_packing(c: &mut Criterion) {
    // Pack a pretty full transaction
    let data: Vec<u8> = (0..packer_naive::MAX_USEFUL_BYTES_PER_TX - 5).map(|_| { rand::random::<u8>() }).collect();

    // Pack naively
    c.bench_function("naive_packing", |b| b.iter(|| {
        let _blobs = packer_naive::get_blobs_from_data(&data);
    }));

    // Pack tightly
    c.bench_function("tight_packing", |b| b.iter(|| {
        let _blobs = packer_tight::get_tight_blobs_from_data(&data);
    }));
}

criterion_group!{name = packing;
                 config = Criterion::default().sample_size(10000);
                 targets = benchmark_packing
}

criterion_main!(packing);



