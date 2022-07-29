use super::bus::{Bus, MemoryOperation};

mod instructions;
mod registers;

use registers::Registers;

enum State {
    NotStarted,
    WaitingPrefetchRead(bool),
    ExecutingInstruction(
        &'static dyn instructions::Instruction,
        Box<dyn instructions::InstructionExecution + 'static>,
    ),
}

pub struct Cpu {
    registers: Registers,
    state: State,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            // TODO: Put the right default value for registers
            registers: Registers::default(),
            state: State::NotStarted,
        }
    }

    fn prefetch_next(&mut self, cb_prefixed: bool) -> MemoryOperation {
        // TODO: Do interrupt fetch too if cb_prefixed is false

        let pc = self.registers.pc();
        self.registers.set_pc(pc.wrapping_add(1));

        self.state = State::WaitingPrefetchRead(cb_prefixed);
        MemoryOperation::Read { address: pc }
    }

    pub fn tick(&mut self, bus: &Bus) -> MemoryOperation {
        match &mut self.state {
            State::NotStarted => self.prefetch_next(false),
            State::WaitingPrefetchRead(cb_prefixed) => {
                if !*cb_prefixed && bus.data() == 0xCB {
                    return self.prefetch_next(true);
                }

                let instruction = match cb_prefixed {
                    true => instructions::CB_PREFIXED_INSTRUCTIONS_TABLE[bus.data() as usize],
                    false => instructions::INSTRUCTIONS_TABLE[bus.data() as usize],
                };

                self.state =
                    State::ExecutingInstruction(instruction, instruction.create_execution());
                self.tick(bus)
            }
            State::ExecutingInstruction(_instruction, instruction_execution) => {
                match instruction_execution.next(&mut self.registers, bus.data()) {
                    instructions::InstructionExecutionState::Yield(memory_op) => memory_op,
                    instructions::InstructionExecutionState::Complete => self.prefetch_next(false),
                }
            }
        }
    }
}
