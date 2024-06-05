use crate::{ijvm_core::InstructionRef, tiny::TinyVars};

pub struct IJVMBlock {
    pub origin: u32,
    pub pool_size: u32,
    pub contents: Vec<u8>,
}

impl IJVMBlock {
    pub fn read_block(stream: impl Iterator<Item = u8>) -> IJVMBlock {
        let mut stream = stream;
        // read 4 u8s into a u32

        let origin = stream
            .by_ref()
            .take(4)
            .fold(0u32, |acc, x| (acc << 8) | x as u32);
        let pool_size = stream
            .by_ref()
            .take(4)
            .fold(0u32, |acc, x| (acc << 8) | x as u32);

        let contents = stream.take(pool_size as usize).collect();
        IJVMBlock {
            origin,
            pool_size,
            contents,
        }
    }
}

/*
struct frame {
    struct frame *previous_frame;
    uint32_t starting_stack_length;
    uint32_t var_count;
    uint32_t var_arr_size;
    // uint16_t *var_idents;
    int32_t *var_values;

    uint32_t restore_pc;
} typedef frame;
 */

pub struct Frame {
    starting_stack_length: u32,
    vars: TinyVars,
    restore_pc: InstructionRef,
}

impl Frame {
    pub fn new(starting_stack_length: u32, var_count: u32, restore_pc: InstructionRef) -> Frame {
        Frame {
            starting_stack_length,
            vars: TinyVars::new(var_count),
            restore_pc,
        }
    }

    #[inline]
    pub fn load_var(&self, var: u16) -> i32 {
        self.vars.load_var(var)
    }

    #[inline]
    pub fn store_var(&mut self, var: u16, value: i32) {
        self.vars.store_var(var, value);
    }

    #[inline]
    pub fn restore_pc(&self) -> InstructionRef {
        self.restore_pc
    }

    #[inline]
    pub fn starting_stack_length(&self) -> u32 {
        self.starting_stack_length
    }

    #[inline]
    pub fn reset(
        &mut self,
        new_stack_length: u32,
        new_var_count: u32,
        new_restore_pc: InstructionRef,
    ) {
        self.vars.reset();
        self.starting_stack_length = new_stack_length;
        self.restore_pc = new_restore_pc;
    }
}
