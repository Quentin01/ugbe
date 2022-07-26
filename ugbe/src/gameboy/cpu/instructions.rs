use std::{borrow::Cow, fmt::Debug};

mod alu;
mod condition;
mod operands;

use super::super::mmu::Mmu;
use super::registers::Registers;
use super::MemoryOperation;

pub trait Instruction {
    fn raw_desc(&self) -> Cow<'static, str>;

    fn desc(&self, pc: u16, mmu: &dyn Mmu) -> Cow<'static, str> {
        let raw_desc = self.raw_desc();

        if raw_desc.contains("u8") {
            str::replace(
                &raw_desc,
                "u8",
                &format!("${:02x}", mmu.read_byte(pc.wrapping_add(1))),
            )
            .into()
        } else if raw_desc.contains("u16") {
            str::replace(
                &raw_desc,
                "u16",
                &format!("${:04x}", mmu.read_word(pc.wrapping_add(1))),
            )
            .into()
        } else if raw_desc.contains("i8") {
            let offset = mmu.read_byte(pc.wrapping_add(1));
            let dst_pc = (pc.wrapping_add(2)) as i32 + ((offset as i8) as i32);

            str::replace(
                &raw_desc,
                "i8",
                // TODO: Display as a signed hexadecimal integer
                &format!(
                    "${:02x} (=> ${:04x})",
                    mmu.read_byte(pc.wrapping_add(1)),
                    dst_pc
                ),
            )
            .into()
        } else {
            raw_desc
        }
    }

    fn create_execution(&self) -> Box<dyn InstructionExecution + 'static>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InstructionExecutionState {
    Yield(MemoryOperation),
    Complete,
}

pub trait InstructionExecution {
    fn next(&mut self, registers: &mut Registers, data_bus: u8) -> InstructionExecutionState;
}

mod implementation;

include!(concat!(env!("OUT_DIR"), "/instructions_gen.rs"));
