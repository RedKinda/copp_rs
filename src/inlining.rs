use std::collections::HashSet;

use crate::{
    ijvm_core::InstructionRef,
    instructions::{IJVMParser, MemoryBlock, ReferredMemoryBlock, ResolveLater},
};

pub struct Inliner<'a> {
    blocks: &'a mut Vec<ReferredMemoryBlock>,
}

impl Inliner<'_> {
    pub fn new(blocks: &mut Vec<ReferredMemoryBlock>) -> Inliner {
        Inliner { blocks }
    }

    pub fn optimize(&mut self) {
        // find references to all method headers
        let mut method_headers = Vec::new();

        for refblock in self.blocks.iter_mut() {
            if let MemoryBlock::METHODHEADER { .. } = refblock.block {
                method_headers.push(refblock.reference);
            }
        }

        // for each header, iterate until ireturn is found, but filter none if there is a branching instruction in the middle
        // we also count stack increases/decreases so we can adjust at the end accordingly
        for header in method_headers {
            self.optimize_header(header);
        }
    }

    fn optimize_header(&mut self, header_ref: InstructionRef) {
        let instruction_ref = self.find_reference(header_ref).unwrap();
        let mut current_ref = instruction_ref + 1;

        let mut stack_tracker: u16 = {
            if let MemoryBlock::METHODHEADER { n_args, n_vars } = self.blocks[instruction_ref].block
            {
                n_args
            } else {
                panic!("Not a method header");
            }
        };

        let mut used_vars = HashSet::new();

        let mut instructions = Vec::new();

        while self.blocks[current_ref].block != MemoryBlock::IRETURN {
            instructions.push(self.blocks[current_ref].clone());
            match self.blocks[current_ref].block {
                MemoryBlock::Delayed(_, _) => return,
                MemoryBlock::BIPUSH(_) => {
                    stack_tracker += 1;
                }
                MemoryBlock::DUP | MemoryBlock::IN => {
                    stack_tracker += 1;
                }
                MemoryBlock::IADD
                | MemoryBlock::ISUB
                | MemoryBlock::IOR
                | MemoryBlock::IAND
                | MemoryBlock::OUT
                | MemoryBlock::POP => {
                    stack_tracker -= 1;
                }
                MemoryBlock::ISTORE(ident) => {
                    stack_tracker -= 1;
                    used_vars.insert(ident);
                }
                MemoryBlock::ILOAD(ident) => {
                    stack_tracker += 1;
                    used_vars.insert(ident);
                }
                _ => {}
            }

            current_ref += 1;
        }

        // now inline the ireturn. This means
        // 1. istore to variable 0
        // 2. adjust stack size with POPs
        // 3. iload variable 0

        instructions.push(MemoryBlock::InlinedIreturn(stack_tracker).into());

        // now find all the references to this method header
        let mut i = 0;
        while i < self.blocks.len() {
            let block = &mut self.blocks[i];
            if let MemoryBlock::Delayed(ResolveLater::INVOKEVIRTUAL(_), referencing) =
                &mut block.block
            {
                if referencing.unwrap() == header_ref {
                    // found a reference, now replace it with the instructions
                    let self.blocks.remove(i);

                    for instruction in instructions.iter().rev() {
                        self.blocks.insert(i, instruction.clone());
                    }
                }
            }

            i += 1;
        }
    }

    fn find_reference(&self, reference: InstructionRef) -> Option<InstructionRef> {
        for i in 0..self.blocks.len() {
            if self.blocks[i].reference == reference {
                return Some(i as InstructionRef);
            }
        }
        None
    }
}
