use copp_rs::ijvm_core::init_ijvm;

fn main() {
    let mut runtime = init_ijvm("files/advanced/mandelbread.ijvm");

    println!("Starting execution");

    #[cfg(feature = "metrics")]
    {
        runtime.run();
        runtime.inner.metrics.print();
    }

    #[cfg(not(feature = "metrics"))]
    {
        for _ in 0..10 {
            runtime.run();
            // reset
            runtime.reset();
        }
    }

    // print metrics
    println!("Finished execution");
    println!("Stack contents:");
    for _ in 0..runtime.inner.visit_stack().len() {
        // println!("{}", runtime.inner.visit_stack()[i]);
    }
}
