use copp_rs::ijvm_core::init_ijvm;

fn main() {
    let mut runtime = init_ijvm("files/task3/GOTO1.ijvm");
    runtime.run();
    println!("Finished execution");
    println!("Stack contents:");
    for i in 0..runtime.visit_stack().len() {
        println!("{}", runtime.visit_stack()[i]);
    }
}
