use criterion::{criterion_group, criterion_main, Criterion, BatchSize, Throughput};
use std::hint::black_box;
use direct_ring_buffer::create_ring_buffer;

fn bench_write<T: Copy + Default + 'static>(c: &mut Criterion) {
    const N: usize = 1_000_000;
    let mut g = c.benchmark_group(format!("write_element::<{}>", std::any::type_name::<T>()));
    g.throughput(Throughput::Bytes((N * size_of::<T>()) as u64));

    g.bench_function("cold_empty_buffer", |b| {
        b.iter_batched(
            || create_ring_buffer::<T>(N),
            |(mut p, _c)| {
                for _ in 0..N {
                    let ok = p.write_element(T::default());
                    debug_assert!(ok);
                }
            },
            BatchSize::LargeInput,
        );
    });

    g.finish();
}


fn bench_read<T: Copy + Default + 'static>(c: &mut Criterion) {
    const N: usize = 1_000_000;
    let mut g = c.benchmark_group(format!("read_element::<{}>", std::any::type_name::<T>()));
    g.throughput(Throughput::Bytes((N * size_of::<T>()) as u64));

    g.bench_function("full_buffer", |b| {
        b.iter_batched(
            || {
                let (mut p, c) = create_ring_buffer::<T>(N);
                for _ in 0..N {
                    let _ = p.write_element(T::default());
                }
                c
            },
            |mut c| {
                for _ in 0..N {
                    black_box(c.read_element().unwrap());
                }
            },
            BatchSize::LargeInput,
        );
    });

    g.finish();
}

fn bench_elements<T: Copy + Default + 'static>(c: &mut Criterion) {
    bench_write::<T>(c);
    bench_read::<T>(c);
}

criterion_group!(
    benches,
    bench_elements::<u8>,
    bench_elements::<u16>,
    bench_elements::<u32>,
    bench_elements::<usize>
);
criterion_main!(benches);
