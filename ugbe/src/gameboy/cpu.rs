use super::hardware::Hardware;

mod alu;
mod instructions;
mod registers;

#[derive(Debug, Copy, Clone)]
pub enum In8 {
    DataBus,
    R8(registers::R8),
}

impl In8 {
    pub fn read_byte(&self, cpu: &Cpu) -> u8 {
        match self {
            Self::DataBus => cpu.data_bus,
            Self::R8(reg) => cpu.registers.read_byte(*reg),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Out8 {
    None,
    R8(registers::R8),
}

impl Out8 {
    pub fn write_byte(&self, cpu: &mut Cpu, value: u8) {
        match self {
            Self::None => {}
            Self::R8(reg) => cpu.registers.write_byte(*reg, value),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Out16 {
    R16(registers::R16),
}

impl Out16 {
    pub fn write_word(&self, cpu: &mut Cpu, value: u16) {
        match self {
            Self::R16(reg) => cpu.registers.write_word(*reg, value),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum In16 {
    R16(registers::R16),
}

impl In16 {
    pub fn read_word(&self, cpu: &Cpu) -> u16 {
        match self {
            Self::R16(reg) => cpu.registers.read_word(*reg),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct InstructionState {
    pc: u16,
    condition: bool,
    idx_mcycle: usize,
    instruction: &'static instructions::Instruction,
}

#[derive(Debug, Copy, Clone)]
pub struct Cpu {
    registers: registers::Registers,
    instruction_state: InstructionState,
    data_bus: u8,
}

impl Cpu {
    pub fn new() -> Self {
        let nop_instruction = instructions::Instruction::decode(0x0)
            .expect("Expected NOP instruction with opcode 0x0");

        Self {
            // TODO: Put the right default value for registers
            registers: registers::Registers::default(),
            instruction_state: InstructionState {
                pc: 0xFFFF,
                condition: true,
                instruction: nop_instruction,
                idx_mcycle: 0,
            },
            data_bus: 0,
        }
    }

    pub fn tick(&mut self, hardware: &mut Hardware) {
        if self.instruction_state.idx_mcycle == 0 {
            println!(
                "Instruction: {}",
                self.instruction_state
                    .instruction
                    .concrete_desc(self.instruction_state.pc, hardware)
            );
        }

        let machine_cycles = match self.instruction_state.instruction.machine_cycles_operations {
            instructions::MachineCycleOperations::Conditional { ok, not_ok, .. } => {
                if self.instruction_state.condition {
                    ok
                } else {
                    not_ok
                }
            }
            instructions::MachineCycleOperations::NotConditional(machine_cycles) => machine_cycles,
        };

        let machine_cycle = machine_cycles[self.instruction_state.idx_mcycle];
        self.instruction_state.idx_mcycle += 1;

        match machine_cycle.execute_operation {
            instructions::ExecuteOperation::None => {}
            instructions::ExecuteOperation::Store8 { dst, src } => {
                let value = src.read_byte(self);
                dst.write_byte(self, value);
            }
            instructions::ExecuteOperation::Store16 { dst, src } => {
                let value = src.read_word(self);
                dst.write_word(self, value);
            }
            instructions::ExecuteOperation::Alu8 { dst, operation } => {
                let value = operation.execute(self);
                dst.write_byte(self, value);
            }
            instructions::ExecuteOperation::Alu16 { dst, operation } => {
                let value = operation.execute(self);
                dst.write_word(self, value);
            }
        };

        match machine_cycle.memory_operation {
            instructions::MemoryOperation::None => {}
            instructions::MemoryOperation::Read(address_bus_src) => {
                let address = address_bus_src.read_word(self);
                self.data_bus = hardware.read_byte(address);
            }
            instructions::MemoryOperation::Write(address_bus_src, data_bus_src) => {
                self.data_bus = data_bus_src.read_byte(self);
                let address = address_bus_src.read_word(self);
                hardware.write_byte(address, self.data_bus)
            }
            instructions::MemoryOperation::CBPrefix => {
                // TODO: Avoid duplication of code
                let pc = self.registers.pc;
                let opcode = hardware.read_byte(pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);

                let instruction = instructions::Instruction::decode_cb_prefixed(opcode);
                match instruction {
                    Some(instruction) => {
                        self.instruction_state = match instruction.machine_cycles_operations {
                            instructions::MachineCycleOperations::Conditional {
                                condition, ..
                            } => InstructionState {
                                pc,
                                condition: condition.check(self),
                                idx_mcycle: 0,
                                instruction,
                            },
                            instructions::MachineCycleOperations::NotConditional(_) => {
                                InstructionState {
                                    pc,
                                    condition: true,
                                    idx_mcycle: 0,
                                    instruction,
                                }
                            }
                        };
                    }
                    None => panic!(
                        "Encountered invalid instruction (0xCB 0x{:x}) at ${:04x}",
                        opcode, pc
                    ),
                }
            }
            instructions::MemoryOperation::PrefetchNext => {
                // TODO: Check for interrupt during the fetch?
                // TODO: Avoid duplication of code
                let pc = self.registers.pc;
                let opcode = hardware.read_byte(pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);

                let instruction = instructions::Instruction::decode(opcode);
                match instruction {
                    Some(instruction) => {
                        self.instruction_state = match instruction.machine_cycles_operations {
                            instructions::MachineCycleOperations::Conditional {
                                condition, ..
                            } => InstructionState {
                                pc,
                                condition: condition.check(self),
                                idx_mcycle: 0,
                                instruction,
                            },
                            instructions::MachineCycleOperations::NotConditional(_) => {
                                InstructionState {
                                    pc,
                                    condition: true,
                                    idx_mcycle: 0,
                                    instruction,
                                }
                            }
                        };
                    }
                    None => panic!(
                        "Encountered invalid instruction (0x{:x}) at ${:04x}",
                        opcode, pc
                    ),
                }
            }
        }

        println!("\tAfter M-cycle: {:?}", self.registers);

        if self.instruction_state.idx_mcycle > machine_cycles.len() {
            panic!(
                "The instruction {} didn't fetch the next one",
                self.instruction_state.instruction
            );
        }
    }
}
