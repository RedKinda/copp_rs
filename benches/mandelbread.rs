use copp_rs::ijvm_core::init_ijvm;
use criterion::{criterion_group, criterion_main, Criterion};

pub fn mandelbread_benchmark(c: &mut Criterion) {
    let mut runtime = init_ijvm("files/advanced/mandelbread.ijvm");

    #[cfg(not(feature = "unsafe"))]
    {
        panic!("Unsafe feature should be enabled for benchmarking");
    }
    let mut g = c.benchmark_group("mandelbread");
    g.sample_size(50);

    g.bench_function("mandelbread", |b| {
        b.iter(|| {
            runtime.run();
            runtime.reset();
        })
    });
}

criterion_group!(benches, mandelbread_benchmark);
criterion_main!(benches);
