use super::hardware::Hardware;

mod instructions;
mod registers;

#[derive(Copy, Clone)]
pub enum State {
    NotStarted,
    ExecutingInstruction(instructions::InstructionState),
}

#[derive(Copy, Clone)]
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
        let pc = self.registers.pc;
        let opcode = hardware.read_byte(pc) as usize;
        self.registers.pc = self.registers.pc.wrapping_add(1);

        let instruction = match cb_prefixed {
            true => &instructions::CB_PREFIXED_INSTRUCTIONS_TABLE[opcode],
            false => &instructions::INSTRUCTIONS_TABLE[opcode],
        };

        // println!("{:?}", self.registers);
        println!("${pc:04x} {}", instruction.desc(pc, hardware));

        self.state = State::ExecutingInstruction(instruction.create_state(pc, self));
    }

    pub fn tick(&mut self, hardware: &mut Hardware) {
        if let State::NotStarted = self.state {
            self.prefetch_next(hardware, false);
        }

        match self.state {
            State::NotStarted => {
                panic!("Not possible to be not started after a fetch of the next instruction")
            }
            State::ExecutingInstruction(instruction_state) => {
                instruction_state.execute(self, hardware);
            }
        }
    }
}
