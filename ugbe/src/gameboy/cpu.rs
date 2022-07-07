use super::hardware::Hardware;

mod instructions;
mod registers;

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
            instructions::ExecuteOperation::StoreInR8 { dst, src } => {
                let value = match src {
                    instructions::In8::DataBus => self.data_bus,
                    instructions::In8::R8(reg) => self.registers.read_byte(reg),
                    instructions::In8::Xor(a, b) => {
                        // TODO: Set the flags
                        self.registers.read_byte(a) ^ self.registers.read_byte(b)
                    }
                };

                self.registers.write_byte(dst, value);
            }
        };

        match machine_cycle.memory_operation {
            instructions::MemoryOperation::None => {}
            instructions::MemoryOperation::Read(address_bus_src) => {
                // TODO: Avoid duplicate code
                let address = match address_bus_src {
                    instructions::AddressBusSource::IncrementR16(reg) => {
                        let address = self.registers.read_word(reg);
                        self.registers.write_word(reg, address.wrapping_add(1));
                        address
                    }
                };

                self.data_bus = hardware.read_byte(address);
            }
            instructions::MemoryOperation::Write(address_bus_src, data_bus_src) => {
                self.data_bus = match data_bus_src {
                    instructions::DataBusSource::R8(reg) => self.registers.read_byte(reg),
                };

                // TODO: Avoid duplicate code
                let address = match address_bus_src {
                    instructions::AddressBusSource::IncrementR16(reg) => {
                        let address = self.registers.read_word(reg);
                        self.registers.write_word(reg, address.wrapping_add(1));
                        address
                    }
                };

                hardware.write_byte(address, self.data_bus)
            }
            instructions::MemoryOperation::CBPrefix => todo!("CB prefix"),
            instructions::MemoryOperation::PrefetchNext => {
                // TODO: Check for interrupt during the fetch?
                let pc = self.registers.pc;
                let opcode = hardware.read_byte(pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);

                let instruction = instructions::Instruction::decode(opcode);
                match instruction {
                    Some(instruction) => {
                        self.instruction_state = match instruction.machine_cycles_operations {
                            instructions::MachineCycleOperations::Conditional {
                                condition, ..
                            } => todo!(),
                            instructions::MachineCycleOperations::NotConditional(_) => {
                                InstructionState {
                                    pc: pc,
                                    condition: true,
                                    idx_mcycle: 0,
                                    instruction,
                                }
                            }
                        };
                    }
                    None => panic!(
                        "Encountered invalid instruction (0x{:x}) at 0x{:x}",
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
