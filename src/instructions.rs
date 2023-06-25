use std::{borrow::BorrowMut, cell::RefCell, io::Read, iter::Peekable};

use crate::{
    ijvm::Frame,
    ijvm_core::{ConstantKind, InstructionRef, Runtime},
    inlining::Inliner,
};

#[derive(Clone, PartialEq, Debug)]
pub enum MemoryBlock {
    BIPUSH(i8),
    DUP,
    ERR,
    HALT,
    IADD,
    IAND,
    IINC(u8, u8),
    ILOAD(u8),
    ISTORE(u8),
    IN,
    IOR,
    IRETURN,
    ISUB,
    NOP,
    OUT,
    POP,
    SWAP,
    WIDE(WideMemoryBlock),

    METHODHEADER { n_args: u16, n_vars: u16 },
    RESOLVED_INVOKEVIRTUAL(InstructionRef),
    RESOLVED_GOTO(InstructionRef),
    RESOLVED_IFEQ(InstructionRef),
    RESOLVED_IFLT(InstructionRef),
    RESOLVED_IF_ICMPEQ(InstructionRef),
    RESOLVED_LDC_W(i32),

    InlinedInvokeVirtual(u16, u16),
    InlinedIreturn(u16),

    Delayed(ResolveLater, Option<InstructionRef>),
    // WIDE(),
    // NEWARRAY(),
    // IALOAD(),
    // IASTORE(),
    // GC(),
    // NETBIND(),
    // NETCONNECT(),
    // NETIN(),
    // NETOUT(),
    // NETCLOSE(),
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum WideMemoryBlock {
    ILOAD(u16),
    ISTORE(u16),
    IIINC(u8, u16),
}

#[derive(Clone, Debug)]
struct IJVMIter<I>
where
    I: Iterator<Item = u8>,
{
    _data: Peekable<I>,
    bytes_read: u64,
    total_bytes_read: u64,
}
impl<I> IJVMIter<I>
where
    I: Iterator<Item = u8>,
{
    fn fetch_bytes_read(&mut self) -> u64 {
        let response = self.bytes_read;
        self.bytes_read = 0;
        response
    }

    fn total_bytes_read(&self) -> u64 {
        self.total_bytes_read
    }

    fn is_end(&mut self) -> bool {
        self._data.peek().is_none()
    }

    fn get_byte(&mut self) -> u8 {
        self.next().unwrap()
    }

    fn get_byte_pair(&mut self) -> (u8, u8) {
        (self.get_byte(), self.get_byte())
    }

    fn get_short(&mut self) -> i16 {
        let pair = self.get_byte_pair();
        (pair.0 as i16) << 8 | pair.1 as i16
    }

    fn get_ushort(&mut self) -> u16 {
        self.get_short() as u16
    }
}
impl<I> Iterator for IJVMIter<I>
where
    I: Iterator<Item = u8>,
{
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self._data.next();
        if let Some(_next) = next {
            self.bytes_read += 1;
            self.total_bytes_read += 1;
        }
        next
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ResolveLater {
    GOTO(i16),
    INVOKEVIRTUAL(u16),
    // comparison branching
    IFEQ(i16),
    IFLT(i16),
    IF_ICMPEQ(i16),
}

thread_local! {
    pub static REFID: RefCell<InstructionRef> = RefCell::new(0);
}
#[derive(Clone, Debug)]
pub struct ReferredMemoryBlock {
    pub block: MemoryBlock,
    pub reference: InstructionRef,
}
impl ReferredMemoryBlock {
    pub fn new(block: MemoryBlock) -> ReferredMemoryBlock {
        let mut resp: ReferredMemoryBlock = ReferredMemoryBlock {
            block,
            reference: 0,
        };
        REFID.with(|id: &RefCell<usize>| {
            let mut id = id.borrow_mut();
            resp.reference = *id;
            *id = id.wrapping_add(1);
        });
        resp
    }
}

pub struct IJVMParser<I>
where
    I: Iterator<Item = u8>,
{
    blocks: Vec<ReferredMemoryBlock>,
    mappings: Vec<usize>,
    data: IJVMIter<I>,
    constants: Vec<ConstantKind>,
}

impl<I> IJVMParser<I>
where
    I: Iterator<Item = u8>,
{
    fn get_target(&self, current: InstructionRef, offset: i16) -> InstructionRef {
        // find value of current within mappings
        let mut ind = 0;
        while self.mappings[ind] != current as usize {
            ind += 1;
        }

        ind = ind.wrapping_add_signed(offset as isize);
        (self.mappings[ind] - 1) as InstructionRef
    }

    fn find_reference(&self, reference: InstructionRef) -> Option<InstructionRef> {
        for i in 0..self.blocks.len() {
            if self.blocks[i].reference == reference {
                return Some(i as InstructionRef);
            }
        }
        None
    }

    pub fn parse_iter(iterator: I, constants: Vec<ConstantKind>) -> Vec<MemoryBlock> {
        let mut parser = IJVMParser {
            blocks: Vec::new(),
            mappings: Vec::new(),
            data: IJVMIter {
                _data: iterator.peekable(),
                bytes_read: 0,
                total_bytes_read: 0,
            },
            constants,
        };
        while !parser.data.is_end() {
            let block = parser.parse_memory_block();

            parser.blocks.push(ReferredMemoryBlock::new(block));

            for _ in 0..parser.data.fetch_bytes_read() {
                parser.mappings.push(parser.blocks.len() - 1);
            }
        }

        // assign references to delayed instructions
        parser.blocks = parser
            .blocks
            .iter()
            .enumerate()
            .map(|(i, block)| {
                if let MemoryBlock::Delayed(instruction, None) = &block.block {
                    let offset = match instruction {
                        ResolveLater::GOTO(offset) => {
                            parser.get_target(i as InstructionRef, *offset)
                        }
                        ResolveLater::INVOKEVIRTUAL(const_ind) => {
                            let constant_val =
                                parser.constants[*const_ind as usize].unwrap_method_ref();
                            let mapped = parser.mappings[constant_val as usize];
                            mapped
                        }
                        ResolveLater::IFEQ(offset) => {
                            parser.get_target(i as InstructionRef, *offset)
                        }
                        ResolveLater::IFLT(offset) => {
                            parser.get_target(i as InstructionRef, *offset)
                        }
                        ResolveLater::IF_ICMPEQ(offset) => {
                            parser.get_target(i as InstructionRef, *offset)
                        }
                    };

                    ReferredMemoryBlock {
                        block: MemoryBlock::Delayed(
                            instruction.clone(),
                            Some(parser.blocks[offset as usize].reference),
                        ),
                        reference: block.reference,
                    }

                    // dbg!(&parser.blocks[i]);
                } else {
                    block.clone()
                }
            })
            .collect();

        // here, we can do optimizations, because references are saved and resolved later
        parser.optimize();

        parser.blocks = parser
            .blocks
            .iter()
            .enumerate()
            .map(|(i, block)| {
                if let MemoryBlock::Delayed(instruction, Some(referencing)) = &block.block {
                    let new_block = match instruction {
                        ResolveLater::GOTO(offset) => {
                            MemoryBlock::RESOLVED_GOTO(parser.find_reference(*referencing).unwrap())
                        }
                        ResolveLater::INVOKEVIRTUAL(offset) => MemoryBlock::RESOLVED_INVOKEVIRTUAL(
                            parser.find_reference(*referencing).unwrap(),
                        ),
                        ResolveLater::IFEQ(offset) => {
                            MemoryBlock::RESOLVED_IFEQ(parser.find_reference(*referencing).unwrap())
                        }
                        ResolveLater::IFLT(offset) => {
                            MemoryBlock::RESOLVED_IFLT(parser.find_reference(*referencing).unwrap())
                        }
                        ResolveLater::IF_ICMPEQ(offset) => MemoryBlock::RESOLVED_IF_ICMPEQ(
                            parser.find_reference(*referencing).unwrap(),
                        ),
                    };
                    ReferredMemoryBlock {
                        block: new_block,
                        reference: block.reference,
                    }
                } else {
                    block.clone()
                }
            })
            .collect();

        parser.blocks.into_iter().map(|x| x.block).collect()
    }

    fn optimize(&mut self) {
        // for example, we can do simple inlining of method refs, if there is no branching code between header and ireturn
        let mut inliner = Inliner::new(&mut self.blocks);
        inliner.optimize();
    }

    fn parse_wide(&mut self) -> WideMemoryBlock {
        match self.data.next().unwrap() {
            0x15 => WideMemoryBlock::ILOAD(self.data.get_ushort()),
            0x36 => WideMemoryBlock::ISTORE(self.data.get_ushort()),
            0x84 => WideMemoryBlock::IIINC(self.data.get_byte(), self.data.get_ushort()),
            c => panic!("Invalid instruction after WIDE: {}", c),
        }
    }

    // instructions parsed in first pass
    pub fn parse_memory_block(&mut self) -> MemoryBlock {
        // check if this is a method ref, from constants
        let ind = self.data.total_bytes_read();
        if self
            .constants
            .contains(&ConstantKind::MethodRef(ind as i32))
        {
            return MemoryBlock::METHODHEADER {
                n_args: self.data.get_ushort(),
                n_vars: self.data.get_ushort(),
            };
        }

        match self.data.next().unwrap() {
            0x10 => MemoryBlock::BIPUSH(self.data.get_byte() as i8),
            0x59 => MemoryBlock::DUP,
            0xFE => MemoryBlock::ERR,
            0xFF => MemoryBlock::HALT,
            0x60 => MemoryBlock::IADD,
            0x7E => MemoryBlock::IAND,
            0x84 => {
                let pair = self.data.get_byte_pair();
                MemoryBlock::IINC(pair.0, pair.1)
            }
            0x15 => MemoryBlock::ILOAD(self.data.get_byte()),
            0xFC => MemoryBlock::IN,
            0xB0 => MemoryBlock::IOR,
            0xAC => MemoryBlock::IRETURN,
            0x36 => MemoryBlock::ISTORE(self.data.get_byte()),
            0x64 => MemoryBlock::ISUB,
            0x13 => MemoryBlock::RESOLVED_LDC_W(
                self.constants[self.data.get_short() as usize].unwrap_stack_value(),
            ),
            0x00 => MemoryBlock::NOP,
            0xFD => MemoryBlock::OUT,
            0x57 => MemoryBlock::POP,
            0x5F => MemoryBlock::SWAP,
            0xC4 => MemoryBlock::WIDE(self.parse_wide()),

            // resolve later
            0x99 => MemoryBlock::Delayed(ResolveLater::IFEQ(self.data.get_short()), None),
            0x9B => MemoryBlock::Delayed(ResolveLater::IFLT(self.data.get_short()), None),
            0x9F => MemoryBlock::Delayed(ResolveLater::IF_ICMPEQ(self.data.get_short()), None),
            0xA7 => MemoryBlock::Delayed(ResolveLater::GOTO(self.data.get_short()), None),
            0xB6 => MemoryBlock::Delayed(ResolveLater::INVOKEVIRTUAL(self.data.get_ushort()), None),
            // 0xD1 => MemoryBlock::NEWARRAY),
            // 0xD2 => MemoryBlock::IALOAD),
            // 0xD3 => MemoryBlock::IASTORE),
            // 0xD4 => MemoryBlock::GC),
            // 0xE1 => MemoryBlock::NETBIND),
            // 0xE2 => MemoryBlock::NETCONNECT),
            // 0xE3 => MemoryBlock::NETIN),
            // 0xE4 => MemoryBlock::NETOUT),
            // 0xE5 => MemoryBlock::NETCLOSE),
            c => {
                // this is likely a method signature
                // verify this by checking that current iter index is among constants
                // this can theoretically parse into incorrect code, but it's unlikely
                // assert!(self
                //     .constants
                //     .contains(&ConstantKind::MethodRef(ind as i32)));

                // combine the byte we read with the next byte to get n_args
                let n_args = (c as u16) << 8 | (self.data.get_byte() as u16);

                MemoryBlock::METHODHEADER {
                    n_args,
                    n_vars: self.data.get_ushort(),
                }
            }
        }
    }
}

impl MemoryBlock {
    #[inline]
    pub fn execute(&self, runtime: &mut Runtime) {
        match &self {
            MemoryBlock::IADD => {
                let top = runtime.stack_pop();
                let second_top = runtime.stack_pop();
                runtime.stack_push(top.wrapping_add(second_top));
            }
            MemoryBlock::ISUB => {
                let top = runtime.stack_pop();
                let second_top = runtime.stack_pop();
                runtime.stack_push(second_top - top);
            }
            MemoryBlock::IAND => {
                let top = runtime.stack_pop();
                let second_top = runtime.stack_pop();
                runtime.stack_push(top & second_top);
            }
            MemoryBlock::IOR => {
                let top = runtime.stack_pop();
                let second_top = runtime.stack_pop();
                runtime.stack_push(top | second_top);
            }
            MemoryBlock::BIPUSH(val) => {
                runtime.stack_push(*val as i32);
            }
            MemoryBlock::OUT => {
                let _popped = runtime.stack_pop();
                // runtime.out_stream().write_all(&[popped as u8]).unwrap();
            }
            MemoryBlock::IN => {
                let mut loaded = [0u8; 1];
                let n_loaded = runtime.in_stream().read(&mut loaded).unwrap();
                if n_loaded > 0 {
                    runtime.stack_push(loaded[0] as i32);
                } else {
                    runtime.stack_push(0);
                }
            }
            MemoryBlock::SWAP => runtime.stack_swap(),
            MemoryBlock::POP => {
                runtime.stack_pop();
            }
            MemoryBlock::HALT => {
                runtime.halt();
            }
            MemoryBlock::DUP => {
                let top = runtime.stack_pop();
                runtime.stack_push(top);
                runtime.stack_push(top);
            }
            MemoryBlock::RESOLVED_GOTO(pc) => runtime.set_pc(*pc),
            MemoryBlock::RESOLVED_IFEQ(pc) => {
                let top = runtime.stack_pop();
                if top == 0 {
                    runtime.set_pc(*pc)
                }
            }
            MemoryBlock::RESOLVED_IFLT(pc) => {
                let top = runtime.stack_pop();
                if top < 0 {
                    runtime.set_pc(*pc)
                }
            }
            MemoryBlock::RESOLVED_IF_ICMPEQ(pc) => {
                let top = runtime.stack_pop();
                let top2 = runtime.stack_pop();
                if top == top2 {
                    runtime.set_pc(*pc)
                }
            }
            MemoryBlock::RESOLVED_LDC_W(constant) => {
                runtime.stack_push(*constant);
            }
            MemoryBlock::ILOAD(ident) => {
                let value = runtime.frame().load_var(*ident as u16);
                runtime.stack_push(value);
            }
            MemoryBlock::ISTORE(ident) => {
                let value = runtime.stack_pop();
                runtime.frame().store_var(*ident as u16, value);
            }
            MemoryBlock::IINC(ident, to_add) => {
                let current_value = runtime.frame().load_var(*ident as u16);
                runtime
                    .frame()
                    .store_var(*ident as u16, current_value + *to_add as i32);
            }
            MemoryBlock::WIDE(block) => match block {
                WideMemoryBlock::ILOAD(ident) => {
                    let value = runtime.frame().load_var(*ident as u16);
                    runtime.stack_push(value);
                }
                WideMemoryBlock::ISTORE(ident) => {
                    let value = runtime.stack_pop();
                    runtime.frame().store_var(*ident as u16, value);
                }
                WideMemoryBlock::IIINC(ident, to_add) => {
                    let current_value = runtime.frame().load_var(*ident as u16);
                    runtime
                        .frame()
                        .store_var(*ident as u16, current_value + *to_add as i32);
                }
            },
            MemoryBlock::RESOLVED_INVOKEVIRTUAL(ind) => {
                let instruction;
                #[cfg(feature = "unsafe")]
                unsafe {
                    instruction = runtime.visit_instructions().get_unchecked(*ind as usize)
                }
                #[cfg(not(feature = "unsafe"))]
                {
                    instruction = &runtime.visit_instructions()[*ind as usize];
                }

                // this should be a method ref
                let (n_args, n_vars) = match *instruction {
                    MemoryBlock::METHODHEADER { n_args, n_vars } => (n_args, n_vars),
                    ref m => panic!(
                        "INVOKEVIRTUAL points at something thats not a METHODHEADER, its a {:?}",
                        m
                    ),
                };

                let mut frame = Frame::new(
                    runtime.stack_len() as u32,
                    n_vars as u32,
                    runtime.program_counter() as InstructionRef,
                );
                frame.store_var(0, 0);

                for i in 0..n_args {
                    frame.store_var(n_args - i - 1, runtime.stack_pop());
                }

                runtime.push_frame(frame);

                runtime.set_pc(*ind);
            }
            MemoryBlock::IRETURN => {
                /*
                    int32_t return_value = stack_pop();
                    frame *previous_frame = current_frame;
                    current_frame = previous_frame->previous_frame;
                    program_counter = previous_frame->restore_pc;
                    stack_top = previous_frame->starting_stack_length;
                    stack_push(return_value);

                    destroy_frame(previous_frame);
                    // stack[stack_top-1] = ret;
                    // stack_push(ret);

                    print_stack();
                    break;
                */

                let return_value = runtime.stack_pop();
                let previous_frame = runtime.pop_frame();
                runtime.set_pc(previous_frame.restore_pc());
                while runtime.stack_len() > previous_frame.starting_stack_length() as usize {
                    runtime.stack_pop();
                }
                runtime.stack_push(return_value);
            }
            MemoryBlock::InlinedIreturn(stack_diff) => {
                let return_value = runtime.stack_pop();
                for _ in 0..*stack_diff {
                    runtime.stack_pop();
                }
                runtime.stack_push(return_value);
            }

            MemoryBlock::InlinedInvokeVirtual(n_args, n_vars) => {
                let mut args = vec![];
                for _ in 0..*n_args {
                    args.push(runtime.stack_pop());
                }
                let frame = runtime.frame();
                for i in 0..*n_vars {
                    frame.store_var(*n_vars - i - 1, args[i as usize]);
                }
            }

            MemoryBlock::ERR => {
                panic!("Encountered ERR instruction");
            }
            MemoryBlock::NOP => {}

            i => todo!("{:?}", i),
        }
    }

    pub fn into(self) -> ReferredMemoryBlock {
        ReferredMemoryBlock::new(self)
    }
}

/*
       case ILOAD: {
           int32_t value = load_val(current_frame, to_exec->arg.p_byte);
           stack_push(value);
           break;
       }
       case ISTORE: {
           uint8_t ident = to_exec->arg.p_byte;
           int32_t value = stack_pop();
           store_var(current_frame, ident, value);
           break;
       }
       case IINC: {
           // First load the variable, then add and store again
           uint8_t ident = to_exec->arg.p_bytes[0];
           int32_t to_add = signed_from_byte(to_exec->arg.p_bytes[1]);
           int32_t current_value = load_val(current_frame, ident);
           store_var(current_frame, ident, current_value + to_add);
           break;
       }

*/
