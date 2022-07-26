use super::hardware::Hardware;

mod instructions;
mod registers;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum MemoryOperation {
    None,
    Read { address: u16 },
    Write { address: u16, value: u8 },
}
enum State {
    NotStarted,
    ExecutingInstruction(Box<dyn instructions::InstructionExecution + 'static>),
    PrefetchingCb,
}

pub struct Cpu {
    registers: registers::Registers,
    data_bus: u8,
    state: State,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            // TODO: Put the right default value for registers
            registers: registers::Registers::default(),
            data_bus: 0,
            state: State::NotStarted,
        }
    }

    fn prefetch_next(&mut self, hardware: &mut Hardware, cb_prefixed: bool) {
        // TODO: Do interrupt fetch too
        let pc = self.registers.pc();
        let opcode = hardware.read_byte(pc) as usize;
        self.registers.set_pc(pc.wrapping_add(1));

        if !cb_prefixed && opcode == 0xCB {
            self.state = State::PrefetchingCb;
            return;
        }

        let instruction = match cb_prefixed {
            true => &instructions::CB_PREFIXED_INSTRUCTIONS_TABLE[opcode],
            false => &instructions::INSTRUCTIONS_TABLE[opcode],
        };

        println!("{:?}", self.registers);
        println!("${pc:04x} {}", instruction.desc(pc, hardware));

        self.state = State::ExecutingInstruction(instruction.create_execution());
    }

    pub fn tick(&mut self, hardware: &mut Hardware) {
        println!("CPU TICK");

        match &mut self.state {
            State::NotStarted => {
                self.prefetch_next(hardware, false);
            }
            State::PrefetchingCb => {
                self.prefetch_next(hardware, true);
            }
            State::ExecutingInstruction(instruction_execution) => {
                match instruction_execution.next(&mut self.registers, self.data_bus) {
                    instructions::InstructionExecutionState::Yield(memory_op) => match memory_op {
                        MemoryOperation::None => {}
                        MemoryOperation::Read { address } => {
                            self.data_bus = hardware.read_byte(address)
                        }
                        MemoryOperation::Write { address, value } => {
                            hardware.write_byte(address, value)
                        }
                    },
                    instructions::InstructionExecutionState::Complete => {
                        self.prefetch_next(hardware, false);
                    }
                }
            }
        }
    }
}
