// fixed capacity stack

use crate::{ijvm::Frame, ijvm_core::InstructionRef};

const STACK_SIZE: usize = 1 << 16;

#[derive(Debug)]
pub struct Stack {
    pub stack: [i32; STACK_SIZE],
    top_value: i32,
    sp: usize,
}

impl Stack {
    pub fn new() -> Stack {
        Stack {
            stack: [0; STACK_SIZE],
            top_value: 0,
            sp: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, value: i32) {
        self.sp += 1;
        self.top_value = value;

        #[cfg(feature = "unsafe")]
        unsafe {
            *self.stack.get_unchecked_mut(self.sp) = value;
        }

        #[cfg(not(feature = "unsafe"))]
        {
            self.stack[self.sp] = value;
        }

    }

    #[inline]
    pub fn pop(&mut self) -> i32 {
        self.sp -= 1;
        let to_ret = self.top_value;

        #[cfg(feature = "unsafe")]
        unsafe {
            self.top_value = *self.stack.get_unchecked(self.sp );
        }

        #[cfg(not(feature = "unsafe"))]
        {
            self.top_value = self.stack[self.sp ];
        }

        to_ret
    }

    #[inline]
    pub fn pop_until_size(&mut self, size: usize) {
        #[cfg(not(feature = "unsafe"))]
        {
            while self.sp > size  {
                self.sp -= 1;
            }
            self.top_value = self.stack[self.sp ];
        }

        #[cfg(feature = "unsafe")]
        {
            if self.sp > size {
                self.sp = size ;
                self.top_value = self.stack[size ];
            }
        }
    }

    #[inline]
    pub fn get_ref_top_n(&mut self, n: usize) -> &[i32] {
        if n == 0 {
            return &[];
        }
        // we dont modify top value because this function is called on INVOKEVIRTUAL which always has empty stack

        self.sp -= n;

        #[cfg(feature = "unsafe")]
        unsafe {
            // unchecked slice
            self.stack.get_unchecked(self.sp+1..self.sp + n+1)
        }

        #[cfg(not(feature = "unsafe"))]
        {
            &self.stack[self.sp..self.sp + n]
        }
    }

    #[inline]
    pub fn peek_top(&self) -> i32 {
        self.top_value
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.sp 
    }

    pub fn clear(&mut self) {
        self.sp = 0;
    }
}

pub type TinyVars = TinyVarsVec;

// fixed size var stack, size u16 max, contains i32s
pub struct TinyVarsVec {
    vars: Vec<i32>,
}

impl TinyVarsVec {
    pub fn new(size: u32) -> Self {
        Self {
            vars: vec![0; size as usize],
        }
    }

    #[inline]
    pub fn load_var(&self, var: u16) -> i32 {
        #[cfg(feature = "unsafe")]
        unsafe {
            *self.vars.get_unchecked(var as usize)
        }

        #[cfg(not(feature = "unsafe"))]
        self.vars[var as usize]
    }

    #[inline]
    pub fn store_var(&mut self, var: u16, value: i32) {
        // possibly extend
        if self.vars.len() <= var as usize {
            self.vars.resize(var as usize + 1, 0);
        }

        #[cfg(feature = "unsafe")]
        unsafe {
            *self.vars.get_unchecked_mut(var as usize) = value;
        }

        #[cfg(not(feature = "unsafe"))]
        {
            self.vars[var as usize] = value;
        }
    }

    pub fn reset(&mut self) {
        self.vars.clear();
    }
}

pub struct TinyVarsDict {
    vars: Vec<(u16, i32)>,
}

impl TinyVarsDict {
    pub fn new(size: u32) -> TinyVarsDict {
        TinyVarsDict {
            vars: Vec::with_capacity(size as usize),
        }
    }

    #[inline]
    pub fn load_var(&self, var: u16) -> i32 {
        for (v, value) in self.vars.iter() {
            if *v == var {
                return *value;
            }
        }
        0
    }

    #[inline]
    pub fn store_var(&mut self, var: u16, value: i32) {
        for (v, v_value) in &mut self.vars {
            if *v == var {
                *v_value = value;
                return;
            }
        }
        self.vars.push((var, value));
    }
}

pub struct FrameStack {
    frames: Vec<Frame>,
    // count: usize,
}

impl FrameStack {
    pub fn new() -> FrameStack {
        FrameStack {
            frames: vec![Frame::new(0, 0, 0)],
            // count: 1,
        }
    }

    #[inline]
    pub fn push_frame(
        &mut self,
        starting_stack_length: u32,
        var_count: u32,
        restore_pc: InstructionRef,
        args: &[i32],
    ) -> &mut Frame {
        // if self.frames.len() < self.count {
        //     self.frames[self.count].reset(starting_stack_length, var_count, restore_pc)
        // } else {
        self.frames
            .push(Frame::new(starting_stack_length, var_count, restore_pc));
        // }

        // self.count += 1;

        let frame = self.current_frame();

        frame.store_var(0, 0);

        for i in 0..args.len() {
            frame.store_var(i as u16, args[i]);
        }

        frame
    }

    #[inline]
    pub fn pop_frame(&mut self) -> Frame {
        // self.count -= 1;

        #[cfg(feature = "unsafe")]
        unsafe {
            // self.frames.get_unchecked_mut(self.count)
            self.frames.pop().unwrap_unchecked()
        }

        #[cfg(not(feature = "unsafe"))]
        self.frames.pop().unwrap()
        // &mut self.frames[self.count]
    }

    #[inline]
    pub fn current_frame(&mut self) -> &mut Frame {
        #[cfg(feature = "unsafe")]
        unsafe {
            // self.frames.get_unchecked_mut(self.count - 1)
            self.frames.last_mut().unwrap_unchecked()
        }

        #[cfg(not(feature = "unsafe"))]
        self.frames.last_mut().unwrap()
        // &mut self.frames[self.count - 1]
    }

    #[inline]
    pub fn clear(&mut self) {
        // self.count = 1;
    }
}
