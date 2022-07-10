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
    pub fn read_byte(&self, cpu_context: &CpuContext) -> u8 {
        match self {
            Self::DataBus => cpu_context.data_bus,
            Self::R8(reg) => cpu_context.registers.read_byte(*reg),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Out8 {
    None,
    R8(registers::R8),
}

impl Out8 {
    pub fn write_byte(&self, cpu_context: &mut CpuContext, value: u8) {
        match self {
            Self::None => {}
            Self::R8(reg) => cpu_context.registers.write_byte(*reg, value),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Out16 {
    R16(registers::R16),
}

impl Out16 {
    pub fn write_word(&self, cpu_context: &mut CpuContext, value: u16) {
        match self {
            Self::R16(reg) => cpu_context.registers.write_word(*reg, value),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum In16 {
    R16(registers::R16),
}

impl In16 {
    pub fn read_word(&self, cpu_context: &CpuContext) -> u16 {
        match self {
            Self::R16(reg) => cpu_context.registers.read_word(*reg),
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
pub struct CpuContext {
    registers: registers::Registers,
    data_bus: u8,
}

#[derive(Debug, Copy, Clone)]
pub struct Cpu {
    context: CpuContext,
    instruction_state: InstructionState,
}

impl Cpu {
    pub fn new() -> Self {
        let nop_instruction = instructions::Instruction::decode(0x0)
            .expect("Expected NOP instruction with opcode 0x0");

        Self {
            context: CpuContext {
                // TODO: Put the right default value for registers
                registers: registers::Registers::default(),
                data_bus: 0,
            },
            // TODO: Instead of faking an instruction, we should use a state in the CPU to detect that we don't have started yet and to load the first instruction
            instruction_state: InstructionState {
                pc: 0xFFFF,
                condition: true,
                instruction: nop_instruction,
                idx_mcycle: 0,
            },
        }
    }

    fn prefetch_next(&mut self, hardware: &mut Hardware, cb_prefixed: bool) {
        // TODO: Do interrupt fetch too
        let pc = self.context.registers.pc;
        let opcode = hardware.read_byte(pc);
        self.context.registers.pc = self.context.registers.pc.wrapping_add(1);

        let instruction = match cb_prefixed {
            true => instructions::Instruction::decode_cb_prefixed(opcode),
            false => instructions::Instruction::decode(opcode),
        };

        match instruction {
            Some(instruction) => {
                self.instruction_state = match instruction.machine_cycles_operations {
                    instructions::MachineCycleOperations::Conditional { condition, .. } => {
                        InstructionState {
                            pc,
                            condition: condition.check(&self.context),
                            idx_mcycle: 0,
                            instruction,
                        }
                    }
                    instructions::MachineCycleOperations::NotConditional(_) => InstructionState {
                        pc,
                        condition: true,
                        idx_mcycle: 0,
                        instruction,
                    },
                };
            }
            None => panic!(
                "Encountered invalid instruction ({}0x{:02x}) at ${:04x}",
                if cb_prefixed { "0xCB " } else { "" },
                opcode,
                pc
            ),
        }
    }

    pub fn tick(&mut self, hardware: &mut Hardware) {
        if self.instruction_state.idx_mcycle == 0 {
            // println!(
            //     "{} ; ${:04x} ; {:?}",
            //     self.instruction_state
            //         .instruction
            //         .concrete_desc(self.instruction_state.pc, hardware),
            //     self.instruction_state.pc,
            //     self.context,
            // );
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
                let value = src.read_byte(&self.context);
                dst.write_byte(&mut self.context, value);
            }
            instructions::ExecuteOperation::Store16 { dst, src } => {
                let value = src.read_word(&self.context);
                dst.write_word(&mut self.context, value);
            }
            instructions::ExecuteOperation::Alu8 { dst, operation } => {
                let value = operation.execute(&mut self.context);
                dst.write_byte(&mut self.context, value);
            }
            instructions::ExecuteOperation::Alu16 { dst, operation } => {
                let value = operation.execute(&self.context);
                dst.write_word(&mut self.context, value);
            }
        };

        match machine_cycle.memory_operation {
            instructions::MemoryOperation::None => {}
            instructions::MemoryOperation::ChangeAddress(address_bus_src) => {
                address_bus_src.read_word(&mut self.context);
            }
            instructions::MemoryOperation::Read(address_bus_src) => {
                let address = address_bus_src.read_word(&mut self.context);
                self.context.data_bus = hardware.read_byte(address);
            }
            instructions::MemoryOperation::Write(address_bus_src, data_bus_src) => {
                self.context.data_bus = data_bus_src.read_byte(&self.context);
                let address = address_bus_src.read_word(&mut self.context);
                hardware.write_byte(address, self.context.data_bus)
            }
            instructions::MemoryOperation::CBPrefix => {
                self.prefetch_next(hardware, true);
            }
            instructions::MemoryOperation::PrefetchNext => {
                self.prefetch_next(hardware, false);
            }
        }

        if self.instruction_state.idx_mcycle > machine_cycles.len() {
            panic!(
                "The instruction {} didn't fetch the next one",
                self.instruction_state.instruction
            );
        }
    }
}
