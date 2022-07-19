use std::{borrow::Cow, fmt::Debug};

#[derive(Clone, Copy)]
pub struct Instruction {
    desc: &'static str,
    operations: super::machine_cycle::Operations,
}

impl Instruction {
    pub const fn new(desc: &'static str, operations: super::machine_cycle::Operations) -> Self {
        Self { desc, operations }
    }

    pub fn desc(&self, pc: u16, hardware: &super::super::Hardware) -> Cow<'static, str> {
        if self.desc.contains("{u8}") {
            str::replace(
                self.desc,
                "{u8}",
                &format!("${:02x}", hardware.read_byte(pc.wrapping_add(1))),
            )
            .into()
        } else if self.desc.contains("{u16}") {
            str::replace(
                self.desc,
                "{u16}",
                &format!("${:04x}", hardware.read_word(pc.wrapping_add(1))),
            )
            .into()
        } else if self.desc.contains("{i8}") {
            let offset = hardware.read_byte(pc.wrapping_add(1));
            let dst_pc = (pc.wrapping_add(2)) as i32 + ((offset as i8) as i32);

            str::replace(
                self.desc,
                "{i8}",
                // TODO: Display as a signed hexadecimal integer
                &format!(
                    "${:02x} (=> ${:04x})",
                    hardware.read_byte(pc.wrapping_add(1)),
                    dst_pc
                ),
            )
            .into()
        } else {
            self.desc.into()
        }
    }

    pub fn create_state(&'static self, start_pc: u16, cpu: &super::super::Cpu) -> InstructionState {
        match self.operations {
            super::machine_cycle::Operations::NotConditional(machine_cycles) => {
                InstructionState::new(start_pc, self, machine_cycles)
            }
            super::machine_cycle::Operations::Conditional { cond, ok, not_ok } => {
                if cond.check(cpu) {
                    InstructionState::new(start_pc, self, ok)
                } else {
                    InstructionState::new(start_pc, self, not_ok)
                }
            }
        }
    }
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Instruction")
            .field("desc", &self.desc)
            .finish()
    }
}

#[derive(Clone, Copy)]
pub struct InstructionState {
    start_pc: u16,
    instr: &'static Instruction,
    idx: usize,
    machine_cycles: &'static [super::machine_cycle::MachineCycle],
}

impl InstructionState {
    fn new(
        start_pc: u16,
        instr: &'static Instruction,
        machine_cycles: &'static [super::machine_cycle::MachineCycle],
    ) -> Self {
        assert!(!machine_cycles.is_empty());

        Self {
            start_pc,
            instr,
            idx: 0,
            machine_cycles,
        }
    }

    pub const fn start_pc(&self) -> u16 {
        self.start_pc
    }

    pub fn execute(mut self, cpu: &mut super::super::Cpu, hardware: &mut super::super::Hardware) {
        self.machine_cycles[self.idx].execute(cpu, hardware);

        self.idx += 1;
        if self.idx < self.machine_cycles.len() {
            cpu.state = super::super::State::ExecutingInstruction(self);
        }
    }
}
