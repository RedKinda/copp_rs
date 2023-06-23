use std::collections::HashMap;

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
    var_values: Vec<i32>,
    restore_pc: u32,
}

impl Frame {
    pub fn new(starting_stack_length: u32, var_count: u32, restore_pc: u32) -> Frame {
        Frame {
            starting_stack_length,
            var_values: Vec::with_capacity(var_count as usize),
            restore_pc,
        }
    }

    pub fn load_var(&self, var: u16) -> i32 {
        self.var_values[var as usize]
    }

    pub fn store_var(&mut self, var: u16, value: i32) {
        if self.var_values.len() <= var as usize {
            self.var_values.resize(var as usize + 1, 0);
        }
        self.var_values[var as usize] = value;
    }

    pub fn restore_pc(&self) -> u32 {
        self.restore_pc
    }

    pub fn starting_stack_length(&self) -> u32 {
        self.starting_stack_length
    }
}
