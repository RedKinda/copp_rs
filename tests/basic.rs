#[cfg(test)]
mod tests {
    use copp_rs::*;
    use ijvm_core::init_ijvm;

    // #[test]
    // fn test_task1_1() {
    //     /*
    //     .constant
    //     .end-constant

    //     .main
    //         BIPUSH 0x30
    //         BIPUSH 0x31
    //         IADD
    //         OUT
    //     HALT
    //     .end-main
    //      */
    //     let runtime = init_ijvm("files/task1/program1.ijvm");
    //     assert_eq!(runtime.constants().len(), 0);
    //     assert_eq!(runtime.visit_instructions().len(), 5);
    //     assert_eq!(
    //         runtime.visit_instructions()[0],
    //         instructions::MemoryBlock::BIPUSH(0x30)
    //     );
    //     assert_eq!(
    //         runtime.visit_instructions()[1],
    //         instructions::MemoryBlock::BIPUSH(0x31)
    //     );
    //     assert_eq!(
    //         runtime.visit_instructions()[2],
    //         instructions::MemoryBlock::IADD
    //     );
    //     assert_eq!(
    //         runtime.visit_instructions()[3],
    //         instructions::MemoryBlock::OUT
    //     );
    //     assert_eq!(
    //         runtime.visit_instructions()[4],
    //         instructions::MemoryBlock::HALT
    //     );
    // }

    // #[test]
    // fn test_task1_2() {
    //     /*
    //     .constant
    //         piet 1
    //         koos 2
    //         jan 3
    //     .end-constant

    //     .main
    //         NOP
    //         LDC_W piet
    //         DUP
    //         LDC_W koos
    //         IADD
    //         LDC_W jan
    //         IADD
    //         OUT
    //         NOP
    //     HALT
    //     .end-main
    //      */
    //     let runtime = init_ijvm("files/task1/program2.ijvm");
    //     assert_eq!(runtime.constants().len(), 3);
    //     assert_eq!(runtime.visit_instructions().len(), 10);

    //     // only test for constants
    //     assert_eq!(runtime.constants()[0], 1);
    //     assert_eq!(runtime.constants()[1], 2);
    //     assert_eq!(runtime.constants()[2], 3);
    // }

    // fn test_all(dir: &str) {
    //     // test all .ijvm files in dir
    //     let files = std::fs::read_dir(dir).unwrap();
    //     for file in files {
    //         let file = file.unwrap();
    //         let path = file.path();
    //         let path = path.to_str().unwrap();
    //         if (!path.ends_with(".ijvm")) || path.split('/').last().unwrap().starts_with('_') {
    //             continue;
    //         }
    //         println!("Testing file {}", path);
    //         let mut runtime = init_ijvm(path);

    //         while !runtime.is_finished() {
    //             runtime.step();
    //             println!(
    //                 "{:?}  Stack: {:?}",
    //                 runtime.inner.program_counter(),
    //                 runtime.inner.visit_stack()
    //             );
    //         }
    //     }
    // }
    // #[test]
    // fn test_task2_all() {
    //     test_all("files/task2");
    // }

    // #[test]
    // fn test_task3_all() {
    //     test_all("files/task3");
    // }

    // #[test]
    // fn test_task4_all() {
    //     test_all("files/task4");
    // }

    // #[test]
    // fn test_task5_all() {
    //     test_all("files/task5");
    // }

    // // advanced folder
    // #[test]
    // fn test_advanced_all() {
    //     test_all("files/advanced");
    // }

    // run mandelbread.ijvm in advanced
    #[test]
    fn test_mandelbread() {
        let mut runtime = init_ijvm("files/advanced/mandelbread.ijvm");
        runtime.run();
    }
}
