use std::io::Read;

#[cfg(feature = "metrics")]
use std::collections::HashMap;

use crate::{
    ijvm,
    instructions::{IJVMParser, MemoryBlock},
    tiny::{FrameStack, Stack},
};
pub type Constant = i32;
pub type InstructionRef = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum ConstantKind {
    None(Constant),
    MethodRef(Constant),
    StackValue(Constant),
    Either(Constant),
}

impl ConstantKind {
    #[inline]
    pub fn value(&self) -> Constant {
        match self {
            ConstantKind::MethodRef(x) => *x,
            ConstantKind::StackValue(x) => *x,
            ConstantKind::Either(x) => *x,
            _ => panic!("None!"),
        }
    }
    #[inline]
    pub fn unchecked_value(&self) -> Constant {
        match self {
            ConstantKind::MethodRef(x) => *x,
            ConstantKind::StackValue(x) => *x,
            ConstantKind::Either(x) => *x,
            ConstantKind::None(x) => *x,
        }
    }
    #[inline]
    pub fn unwrap_method_ref(&self) -> Constant {
        match self {
            ConstantKind::MethodRef(x) => *x,
            ConstantKind::Either(x) => *x,
            #[cfg(feature = "unsafe")]
            _ => unsafe { std::hint::unreachable_unchecked() },
            #[cfg(not(feature = "unsafe"))]
            _ => panic!("Not a method ref"),
        }
    }
    #[inline]
    pub fn unwrap_stack_value(&self) -> Constant {
        match self {
            ConstantKind::StackValue(x) => *x,
            ConstantKind::Either(x) => *x,
            #[cfg(feature = "unsafe")]
            _ => unsafe { std::hint::unreachable_unchecked() },
            #[cfg(not(feature = "unsafe"))]
            _ => panic!("Not a method ref"),
        }
    }
    #[inline]
    pub fn as_method(self) -> ConstantKind {
        match self {
            ConstantKind::StackValue(x) => ConstantKind::Either(x),
            ConstantKind::None(x) => ConstantKind::MethodRef(x),
            _ => self,
        }
    }
    #[inline]
    pub fn as_stack(self) -> ConstantKind {
        match self {
            ConstantKind::MethodRef(x) => ConstantKind::Either(x),
            ConstantKind::None(x) => ConstantKind::StackValue(x),
            _ => self,
        }
    }
    #[inline]
    pub fn is_none(&self) -> bool {
        matches!(self, ConstantKind::None(_))
    }
}

pub struct Runtime {
    instructions: Vec<MemoryBlock>,
    pub inner: RuntimeInner,
}

pub struct RuntimeInner {
    instructions: Vec<MemoryBlock>,
    constants: Vec<Constant>,
    frames: FrameStack,
    program_counter: usize, // counter over instructions, not original bytes
    is_finished: bool,
    stack: Stack,
    pub out_stream: std::io::Stderr,
    in_stream: std::io::Stdin,

    #[cfg(feature = "metrics")]
    pub metrics: Metrics,
}

impl Runtime {
    #[inline]
    pub fn step(&mut self) {
        let instruction;
        #[cfg(feature = "unsafe")]
        unsafe {
            instruction = self.instructions.get_unchecked(self.inner.program_counter);
            // .get(self.program_counter).unwrap()
        }
        #[cfg(not(feature = "unsafe"))]
        {
            instruction = &self.instructions[self.program_counter()];
        }

        #[cfg(feature = "metrics")]
        {
            let counter = self
                .inner
                .metrics
                .metrics
                .entry(instruction.metrics_str())
                .or_insert(0);
            *counter += 1;
        }

        // println!(
        //     "Executing: {:?} - stack {:?}",
        //     instruction,
        //     &self.inner.stack.stack[1..self.inner.stack.len() + 1]
        // );

        instruction.execute(&mut self.inner);
        #[cfg(feature = "unsafe")]
        unsafe {
            self.inner.program_counter = self.inner.program_counter.unchecked_add(1);
        }
        #[cfg(not(feature = "unsafe"))]
        {
            self.inner.program_counter += 1;
        }

        // this check has to be present for tests, as they dont HALT correctly
        #[cfg(not(feature = "unsafe"))]
        if self.program_counter() >= self.instructions.len() {
            self.inner.is_finished = true;
        }
    }
    pub fn steps(&mut self, count: usize) {
        for _ in 0..count {
            self.step();
        }
    }
    #[inline]
    pub fn run(&mut self) {
        while !self.inner.is_finished {
            self.step();
        }
    }

    #[inline]
    pub fn visit_instructions(&self) -> &Vec<MemoryBlock> {
        &self.instructions
    }

    pub fn reset(&mut self) {
        self.inner.program_counter = 0;
        self.inner.stack.clear();
        self.inner.frames.clear();
        self.inner.is_finished = false;

        #[cfg(feature = "metrics")]
        {
            self.inner.metrics.clear();
        }
    }

    #[inline]
    pub fn tos(&self) -> i32 {
        self.inner.stack.peek_top()
    }

    pub fn constants(&self) -> &Vec<Constant> {
        self.inner.constants()
    }
    #[inline]
    pub fn is_finished(&self) -> bool {
        self.inner.is_finished
    }
    #[inline]
    pub fn program_counter(&self) -> usize {
        self.inner.program_counter()
    }
    #[inline]
    pub fn frame(&mut self) -> &ijvm::Frame {
        self.inner.frames.current_frame()
    }
}

impl RuntimeInner {
    #[inline]
    pub fn set_pc(&mut self, pc: InstructionRef) {
        self.program_counter = pc;
    }

    #[inline]
    pub fn stack_pop(&mut self) -> i32 {
        self.stack.pop()
    }

    #[inline]
    pub fn pop_until_size(&mut self, size: usize) {
        self.stack.pop_until_size(size);
    }

    #[inline]
    pub fn stack_push(&mut self, value: i32) {
        self.stack.push(value);
    }

    #[inline]
    pub fn stack_len(&self) -> usize {
        self.stack.len()
    }

    #[inline]
    pub fn stack_swap(&mut self) {
        // #[cfg(feature = "unsafe")]
        // unsafe {
        //     let len = self.stack.len();
        //     // unsafely swaps top two elements on the stack
        //     // SAFETY: IJVM file has to be valid
        //     let a = *self.stack.get_unchecked(len - 1);
        //     let b = *self.stack.get_unchecked(len - 2);
        //     *self.stack.get_unchecked_mut(len - 1) = b;
        //     *self.stack.get_unchecked_mut(len - 2) = a;
        // }
        // #[cfg(not(feature = "unsafe"))]
        {
            let a = self.stack_pop();
            let b = self.stack_pop();
            self.stack_push(a);
            self.stack_push(b);
        }
    }

    #[inline]
    pub fn in_stream(&mut self) -> &mut std::io::Stdin {
        &mut self.in_stream
    }

    #[inline]
    pub fn out_stream(&mut self) -> &mut std::io::Stderr {
        &mut self.out_stream
    }

    #[inline]
    pub fn halt(&mut self) {
        self.is_finished = true;
    }

    #[inline]
    pub fn constants(&self) -> &Vec<Constant> {
        &self.constants
    }

    #[inline]
    pub fn program_counter(&self) -> usize {
        self.program_counter
    }

    #[inline]
    pub fn visit_stack(&self) -> &Stack {
        &self.stack
    }

    #[inline]
    pub fn visit_instructions(&self) -> &Vec<MemoryBlock> {
        &self.instructions
    }

    #[inline]
    pub fn frames(&mut self) -> &mut FrameStack {
        &mut self.frames
    }

    #[inline]
    pub fn frame(&mut self) -> &mut ijvm::Frame {
        self.frames.current_frame()
    }

    #[inline]
    pub fn push_frame(&mut self, var_count: u16, arg_count: u16) -> &mut ijvm::Frame {
        self.frames.push_frame(
            self.stack_len() as u32,
            var_count as u32,
            self.program_counter() as InstructionRef,
            self.stack.get_ref_top_n(arg_count as usize),
        )
    }

    #[inline]
    pub fn pop_frame(&mut self) {
        let frame = self.frames.pop_frame();

        let restore_pc = frame.restore_pc();
        let stack_len = frame.starting_stack_length() as usize;

        self.set_pc(restore_pc);
        self.pop_until_size(stack_len);
    }
}

pub fn init_ijvm(binary_file: &str) -> Runtime {
    let mut fp = std::fs::File::open(binary_file).unwrap();
    let mut header = [0u8; 4];
    fp.read_exact(&mut header).unwrap();
    let header = u32::from_be_bytes(header);
    if header != 0x1DEADFAD {
        panic!("Invalid header");
    }
    let constants = load_constants(ijvm::IJVMBlock::read_block(
        fp.by_ref().bytes().map(|x| x.unwrap()),
    ));
    let text = ijvm::IJVMBlock::read_block(fp.by_ref().bytes().map(|x| x.unwrap()));

    // dbg!(&constants, &text.contents);

    // classify constants
    // find 0x13 in text.contents, the index of next constant is stackvalue
    let mut constants_kinded = constants
        .iter()
        .map(|x| ConstantKind::None(*x))
        .collect::<Vec<_>>();

    for ind in 0..text.contents.len() {
        let byte = &text.contents[ind];
        if *byte == 0x13 {
            if text.contents.len() < ind + 3 {
                continue;
            }
            let constant_ind =
                (text.contents[ind + 1] as usize) << 8 | text.contents[ind + 2] as usize;

            if constants_kinded.len() > constant_ind {
                constants_kinded[constant_ind] = constants_kinded[constant_ind].clone().as_stack();
            }
        }
    }

    // do the same for methods with 0xB6
    for ind in 0..text.contents.len() {
        let byte = &text.contents[ind];
        if text.contents.len() < ind + 3 {
            continue;
        }
        if *byte == 0xB6 {
            let constant_ind =
                (text.contents[ind + 1] as usize) << 8 | text.contents[ind + 2] as usize;
            constants_kinded[constant_ind] = constants_kinded[constant_ind].clone().as_method();
        }
    }

    // check none constant is none
    for (i, x) in constants_kinded.iter().enumerate() {
        if x.is_none() {
            println!("WARNING: Constant {} is none", i);
        }
    }

    let instructions = IJVMParser::parse_iter(text.contents.iter().cloned(), constants_kinded);

    // println!(
    //     "Loaded ijvm file {}, constants pool size: {}, text pool size: {}",
    //     binary_file,
    //     constants.len(),
    //     text.pool_size
    // );
    // let current_frame = ijvm::Frame::new(0, 0, 0);
    let program_counter = 0;
    let is_finished = false;
    let stack = Stack::new();
    let out_stream = std::io::stderr();
    let in_stream = std::io::stdin();
    let inner = RuntimeInner {
        instructions: instructions.clone(),
        constants,
        frames: FrameStack::new(),
        program_counter,
        is_finished,
        stack,
        out_stream,
        in_stream,
        #[cfg(feature = "metrics")]
        metrics: Metrics::default(),
    };
    Runtime {
        inner,
        instructions,
    }
}

/*
int32_t load_constant(uint8_t index) {
    int32_t num = constants.contents[index * 4] << 24;
    num = num | constants.contents[index * 4 + 1] << 16;
    num = num | constants.contents[index * 4 + 2] << 8;
    num = num | constants.contents[index * 4 + 3];
    return num;
}

 */

pub fn load_constants(block: ijvm::IJVMBlock) -> Vec<i32> {
    let mut constants = Vec::new();
    for i in 0..block.pool_size / 4 {
        let mut num = 0i32;
        num |= (block.contents[i as usize * 4] as i32) << 24;
        num |= (block.contents[i as usize * 4 + 1] as i32) << 16;
        num |= (block.contents[i as usize * 4 + 2] as i32) << 8;
        num |= block.contents[i as usize * 4 + 3] as i32;
        constants.push(num);
    }
    constants
}

#[cfg(feature = "metrics")]
#[derive(Default)]
pub struct Metrics {
    metrics: HashMap<&'static str, usize>,
}

#[cfg(feature = "metrics")]
impl Metrics {
    pub fn print(&self) {
        for (k, v) in &self.metrics {
            println!(
                "{}: {} ({:.2}%)",
                k,
                v,
                (*v as f64 / self.total() as f64) * 100.0
            );
        }
    }

    pub fn total(&self) -> usize {
        self.metrics.values().sum()
    }

    pub fn clear(&mut self) {
        self.metrics.clear();
    }
}

#[cfg(feature = "metrics")]
impl Drop for Metrics {
    fn drop(&mut self) {
        self.print();
    }
}
