#![feature(int_roundings)]

use std::time::Duration;
use criterion::*;

#[path = "../src/packer_tight.rs"]
mod packer_tight;

fn benchmark_tight_packing(c: &mut Criterion) {
    // Try to pack a pretty full transaction
    let data: Vec<u8> = (0..packer_tight::MAX_TIGHT_USEFUL_BYTES_PER_TX - 5).map(|_| { rand::random::<u8>() }).collect();

    c.bench_function("tight_packing", |b| b.iter(|| {
        let _blobs = packer_tight::get_tight_blobs_from_data(&data);
    }));
}

criterion_group!{name = tight;
                 config = Criterion::default().measurement_time(Duration::from_secs(60)).sample_size(10000);
                 targets = benchmark_tight_packing
}

criterion_main!(tight);
