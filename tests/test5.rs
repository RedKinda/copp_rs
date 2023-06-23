#[cfg(test)]
mod tests_5 {
    use copp_rs::{ijvm_core::init_ijvm, *};

    #[test]
    fn test_invokenoargs() {
        let mut runtime = init_ijvm("files/task5/TestInvokeNoArgs.ijvm");

        runtime.steps(2);
        assert_eq!(runtime.tos(), 0x42);
        runtime.steps(2);
        assert_eq!(runtime.tos(), 0x43);
        runtime.steps(2);
        assert_eq!(runtime.tos(), 0x43);
    }

    #[test]
    fn test_invoke1() {
        let mut runtime = init_ijvm("files/task5/test-invokevirtual1.ijvm");

        runtime.steps(3);
        let pc = runtime.program_counter();
        runtime.step();
        assert_ne!(runtime.program_counter(), pc + 1);
        runtime.step();
        assert_eq!(runtime.tos(), 0x1);
        runtime.steps(2);
    }

    #[test]
    fn test_ireturn1() {
        let mut runtime = init_ijvm("files/task5/test-invokevirtual1.ijvm");

        runtime.steps(6);
        assert_eq!(runtime.tos(), 0x1);
        runtime.step();
        assert_eq!(runtime.tos(), 0x2);
    }

    #[test]
    fn test_invoke2() {
        let mut runtime = init_ijvm("files/task5/test-invokevirtual2.ijvm");

        runtime.steps(5);
        let pc = runtime.program_counter();
        runtime.step();
        assert_ne!(runtime.program_counter(), pc + 1);
        runtime.step();
        assert_eq!(runtime.tos(), 0x2);
        runtime.step();
        assert_eq!(runtime.tos(), 0x3);
        runtime.steps(3);
    }

    #[test]
    fn test_ireturn2() {
        let mut runtime = init_ijvm("files/task5/test-invokevirtual2.ijvm");

        runtime.steps(10);
        assert_eq!(runtime.tos(), 0x5);
        runtime.step();
        assert_eq!(runtime.tos(), 0x2);
    }

    #[test]
    fn test_frame() {
        let mut runtime = init_ijvm("files/task5/testinvoke-frame.ijvm");

        runtime.steps(6);
        assert_eq!(runtime.frame().load_var(0), 0x4);
        runtime.step();
        assert_eq!(runtime.frame().load_var(1), 0x3);

        runtime.steps(5);
        assert_eq!(runtime.frame().load_var(1), 0x2);
        assert_eq!(runtime.frame().load_var(2), 0x3);

        runtime.steps(2);
        assert_eq!(runtime.tos(), 0x5);
        runtime.step();
        assert_eq!(runtime.tos(), 0x2);
        assert_eq!(runtime.frame().load_var(0), 0x4);
        assert_eq!(runtime.frame().load_var(1), 0x3);
    }

    #[test]
    fn test_nested_frame() {
        let mut runtime = init_ijvm("files/task5/test-nestedinvoke-frame.ijvm");

        runtime.steps(7);
        assert_eq!(runtime.tos(), 0x5);
        assert_eq!(runtime.frame().load_var(0), 0x21);
        assert_eq!(runtime.frame().load_var(1), 0x2C);

        runtime.steps(5);
        assert_eq!(runtime.tos(), 0x6);
        assert_eq!(runtime.frame().load_var(1), 0x1);
        assert_eq!(runtime.frame().load_var(2), 0x5);

        runtime.steps(8);
        assert_eq!(runtime.tos(), 0xA);
        assert_eq!(runtime.frame().load_var(1), 9);

        runtime.steps(3);
        assert_eq!(runtime.tos(), 0xA);
        assert_eq!(runtime.frame().load_var(1), 0x1);
        assert_eq!(runtime.frame().load_var(2), 0x5);

        runtime.steps(2);
        assert_eq!(runtime.tos(), 0x10);
        assert_eq!(runtime.frame().load_var(0), 0x21);
        assert_eq!(runtime.frame().load_var(1), 0x2C);
    }
}
