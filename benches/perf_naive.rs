#![feature(int_roundings)]

use std::time::Duration;
use criterion::*;

#[path = "../src/packer.rs"]
mod packer;

fn benchmark_naive_packing(c: &mut Criterion) {
    // Pack a pretty full transaction
    let data: Vec<u8> = (0..packer::MAX_USEFUL_BYTES_PER_TX - 5).map(|_| { rand::random::<u8>() }).collect();

    c.bench_function("naive_packing", |b| b.iter(|| {
        let blobs = packer::get_blobs_from_data(&data);
    }));
}

criterion_group!{name = naive;
                 config = Criterion::default().measurement_time(Duration::from_secs(60)).sample_size(10000);
                 targets = benchmark_naive_packing
}

criterion_main!(naive);



