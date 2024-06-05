use copp_rs::ijvm_core::init_ijvm;
use criterion::{criterion_group, criterion_main, Criterion};

pub fn mandelbread_benchmark(c: &mut Criterion) {
    let mut runtime = init_ijvm("files/advanced/mandelbread.ijvm");

    // #[cfg(not(feature = "unsafe"))]
    // {
    //     panic!("Unsafe feature should be enabled for benchmarking");
    // }
    let mut g = c.benchmark_group("mandelbread");
    g.sample_size(50);

    g.bench_function("mandelbread", |b| {
        b.iter(|| {
            runtime.run();
            runtime.reset();
        })
    });
    // .bench_function("mandelbread-full", |b| {
    //     b.iter(|| {
    //         let mut runtime = init_ijvm("files/advanced/mandelbread.ijvm");
    //         runtime.run();
    //     })
    // });
}

pub fn mandelbread_bench_full(c: &mut Criterion) {
    let mut g = c.benchmark_group("mandelbread-full");
    g.sample_size(50);

    g.bench_function("mandelbread-full", |b| {
        b.iter(|| {
            let mut runtime = init_ijvm("files/advanced/mandelbread.ijvm");
            runtime.run();
        })
    });
}

criterion_group!(benches, mandelbread_benchmark);
// criterion_group!(benches_full, mandelbread_bench_full);
criterion_main!(benches);
